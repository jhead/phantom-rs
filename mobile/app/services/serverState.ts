import AsyncStorage from '@react-native-async-storage/async-storage';
import {atom} from 'jotai';
import {atomWithStorage, createJSONStorage} from 'jotai/utils';
import {Phantom, PhantomOpts} from 'react-native-phantom';
import {Client, ClientInterface, Pong} from 'react-native-phantom';

export type ServerStatus = 'online' | 'offline' | 'connecting' | 'starting';

export interface ServerData {
  players: number;
  maxPlayers: number;
  motd: [string, string];
  icon?: string;
  lastPing?: number;
}

export interface Server {
  id: string;
  name: string;
  address: string;
  port: string;
  status: ServerStatus;
  data?: ServerData;
  autoStart: boolean;
}

// Create persistent storage for servers
const storage = createJSONStorage<Server[]>(() => AsyncStorage);

// Persistent servers atom with AsyncStorage
export const serversAtom = atomWithStorage<Server[]>('servers', [], storage);

// Service management state
export const serviceStateAtom = atom({
  isInitialized: false,
  isRunning: false,
});

class ServerStateService {
  private phantomInstances = new Map<string, Phantom>();
  private client: ClientInterface | null = null;
  private pingInterval: NodeJS.Timeout | null = null;
  private pingInProgress = new Set<string>();
  private updateCallbacks: ((servers: Server[]) => void)[] = [];
  private currentServers: Server[] = [];

  private logger = {
    info: (message: string, ...args: any[]) =>
      console.log(`[ServerState] ${message}`, ...args),
    error: (message: string, ...args: any[]) =>
      console.error(`[ServerState] ${message}`, ...args),
    warn: (message: string, ...args: any[]) =>
      console.warn(`[ServerState] ${message}`, ...args),
  };

  constructor() {
    this.logger.info('ServerStateService initialized');
  }

  // Initialize the service and load persisted servers
  async initialize(): Promise<void> {
    this.logger.info('Initializing server state service');

    try {
      // Load persisted servers
      const stored = await AsyncStorage.getItem('servers');
      if (stored) {
        const servers: Server[] = JSON.parse(stored);
        this.currentServers = servers.map(server => ({
          ...server,
          status: 'offline' as ServerStatus,
          data: undefined, // Clear stale data
        }));
        this.notifyCallbacks();
      }

      // Initialize ping client
      this.client = await Client.create();

      // Start ping service
      this.startPingService();

      this.logger.info('Server state service initialized successfully');

      // Start auto-start servers asynchronously (non-blocking)
      if (this.currentServers.length > 0) {
        this.startAutoStartServersAsync();
      }
    } catch (error) {
      this.logger.error('Failed to initialize server state service:', error);
      throw error;
    }
  }

  // Shutdown the service and cleanup resources
  async shutdown(): Promise<void> {
    this.logger.info('Shutting down server state service');

    // Stop ping service
    this.stopPingService();

    // Stop all phantom instances
    await this.stopAllPhantomInstances();

    // Clear callbacks
    this.updateCallbacks.length = 0;

    this.logger.info('Server state service shut down');
  }

  // Get current servers
  getServers(): Server[] {
    return [...this.currentServers];
  }

  // Add update callback
  onUpdate(callback: (servers: Server[]) => void): () => void {
    this.updateCallbacks.push(callback);
    // Return unsubscribe function
    return () => {
      const index = this.updateCallbacks.indexOf(callback);
      if (index > -1) {
        this.updateCallbacks.splice(index, 1);
      }
    };
  }

  // Add a new server
  async addServer(
    serverData: Omit<Server, 'status' | 'data' | 'autoStart'>,
  ): Promise<void> {
    this.logger.info('Adding server:', serverData);

    const newServer: Server = {
      ...serverData,
      status: 'connecting',
      autoStart: true,
    };

    // Add to current servers
    this.currentServers = [...this.currentServers, newServer];

    // Persist to storage
    await this.persistServers();

    // Notify callbacks
    this.notifyCallbacks();

    try {
      // Start phantom instance
      await this.startPhantomInstance(newServer);

      // Update status to starting
      this.updateServerStatus(newServer.id, 'starting');

      // Ping immediately to get initial data
      this.pingServer(newServer);

      this.logger.info('Server added successfully:', newServer.name);
    } catch (error) {
      this.logger.error('Failed to start server:', error);
      this.updateServerStatus(newServer.id, 'offline');
      throw error;
    }
  }

  // Remove a server
  async removeServer(serverId: string): Promise<void> {
    this.logger.info('Removing server:', serverId);

    const server = this.currentServers.find(s => s.id === serverId);
    if (!server) {
      this.logger.warn('Server not found for removal:', serverId);
      return;
    }

    // Stop phantom instance
    await this.stopPhantomInstance(server);

    // Remove from current servers
    this.currentServers = this.currentServers.filter(s => s.id !== serverId);

    // Persist to storage
    await this.persistServers();

    // Notify callbacks
    this.notifyCallbacks();

    this.logger.info('Server removed successfully:', server.name);
  }

  // Update server auto-start setting
  async updateServerAutoStart(
    serverId: string,
    autoStart: boolean,
  ): Promise<void> {
    const serverIndex = this.currentServers.findIndex(s => s.id === serverId);
    if (serverIndex === -1) return;

    this.currentServers[serverIndex] = {
      ...this.currentServers[serverIndex],
      autoStart,
    };

    // Persist to storage
    await this.persistServers();

    // Notify callbacks
    this.notifyCallbacks();
  }

  // Update a server's details
  async updateServer(
    serverId: string,
    updates: Partial<Omit<Server, 'id' | 'status' | 'data'>>,
  ): Promise<void> {
    this.logger.info('Updating server:', serverId, updates);

    const serverIndex = this.currentServers.findIndex(s => s.id === serverId);
    if (serverIndex === -1) {
      this.logger.warn('Server not found for update:', serverId);
      return;
    }

    const server = this.currentServers[serverIndex];
    const updatedServer = {
      ...server,
      ...updates,
    };

    console.log('Updating server:', serverId, updates, server, updatedServer);

    // Stop existing phantom instance
    await this.stopPhantomInstance(server);

    // Update server in current servers
    this.currentServers[serverIndex] = updatedServer;

    // Persist to storage
    await this.persistServers();

    // Notify callbacks
    this.notifyCallbacks();

    try {
      // Start new phantom instance
      await this.startPhantomInstance(updatedServer);

      // Update status to starting
      this.updateServerStatus(updatedServer.id, 'starting');

      // Ping immediately to get initial data
      this.pingServer(updatedServer);

      this.logger.info('Server updated successfully:', updatedServer.name);
    } catch (error) {
      this.logger.error('Failed to start updated server:', error);
      this.updateServerStatus(updatedServer.id, 'offline');
      throw error;
    }
  }

  // Private methods

  private async persistServers(): Promise<void> {
    try {
      await AsyncStorage.setItem(
        'servers',
        JSON.stringify(this.currentServers),
      );
    } catch (error) {
      this.logger.error('Failed to persist servers:', error);
    }
  }

  private notifyCallbacks(): void {
    this.updateCallbacks.forEach(callback => {
      try {
        callback([...this.currentServers]);
      } catch (error) {
        this.logger.error('Error in update callback:', error);
      }
    });
  }

  private async startAutoStartServers(): Promise<void> {
    const autoStartServers = this.currentServers.filter(s => s.autoStart);

    this.logger.info(`Starting ${autoStartServers.length} auto-start servers`);

    for (const server of autoStartServers) {
      try {
        this.updateServerStatus(server.id, 'starting');
        await this.startPhantomInstance(server);
        this.logger.info('Auto-started server:', server.name);
      } catch (error) {
        this.logger.error('Failed to auto-start server:', server.name, error);
        this.updateServerStatus(server.id, 'offline');
      }
    }
  }

  private startAutoStartServersAsync(): void {
    // Start servers asynchronously without blocking initialization
    const autoStartServers = this.currentServers.filter(s => s.autoStart);

    this.logger.info(
      `Starting ${autoStartServers.length} auto-start servers (async)`,
    );

    autoStartServers.forEach(server => {
      this.startServerAsync(server);
    });
  }

  private async startServerAsync(server: Server): Promise<void> {
    try {
      this.updateServerStatus(server.id, 'starting');

      // Start phantom instance with timeout
      await Promise.race([
        this.startPhantomInstance(server),
        new Promise((_, reject) =>
          setTimeout(() => reject(new Error('Phantom start timeout')), 30000),
        ),
      ]);

      this.logger.info('Auto-started server:', server.name);
      // Status will be updated by ping service once it connects
    } catch (error) {
      this.logger.error('Failed to auto-start server:', server.name, error);
      this.updateServerStatus(server.id, 'offline');
    }
  }

  private async startPhantomInstance(server: Server): Promise<void> {
    const existingPhantom = this.phantomInstances.get(server.id);
    if (existingPhantom) {
      this.logger.info(
        'Restarting existing Phantom instance for:',
        server.name,
      );
      await existingPhantom.start();
      return;
    }

    this.logger.info('Creating new Phantom instance for:', server.name);

    const opts: PhantomOpts = {
      server: `${server.address}:${server.port}`,
      bind: '0.0.0.0',
      bindPort: 0,
      debug: true,
      ipv6: false,
      timeout: 10000n,
    };

    const phantom = new Phantom(opts);

    try {
      phantom.setLogger({
        logString: (str: string) => {
          this.logger.info(`Phantom[${server.name}]:`, str);
        },
      });
    } catch (error) {
      // Logger setup is optional
      this.logger.info('Failed to set phantom logger:', error);
    }

    this.logger.info('Starting Phantom instance for:', server.name);

    // Start phantom and store instance immediately
    // The actual startup is logged by the phantom logger
    phantom
      .start()
      .then(() => {
        this.logger.info(
          'Phantom instance started successfully for:',
          server.name,
        );
      })
      .catch(error => {
        this.logger.error(
          'Phantom instance failed to start for:',
          server.name,
          error,
        );
        this.updateServerStatus(server.id, 'offline');
      });

    this.phantomInstances.set(server.id, phantom);
    this.logger.info('Phantom instance created and starting for:', server.name);
  }

  private async stopPhantomInstance(server: Server): Promise<void> {
    const phantom = this.phantomInstances.get(server.id);
    if (phantom) {
      try {
        await phantom.stop();
        this.phantomInstances.delete(server.id);
        this.logger.info('Stopped Phantom instance for:', server.name);
      } catch (error) {
        this.logger.error(
          'Failed to stop Phantom instance:',
          server.name,
          error,
        );
      }
    }
  }

  private async stopAllPhantomInstances(): Promise<void> {
    const stopPromises = Array.from(this.phantomInstances.entries()).map(
      async ([serverId, phantom]) => {
        const server = this.currentServers.find(s => s.id === serverId);
        if (server) {
          await this.stopPhantomInstance(server);
        }
      },
    );

    await Promise.allSettled(stopPromises);
  }

  private startPingService(): void {
    if (this.pingInterval) {
      clearInterval(this.pingInterval);
    }

    // Initial ping for all servers
    this.currentServers.forEach(server => {
      this.pingServer(server);
    });

    // Start periodic pinging
    this.pingInterval = setInterval(() => {
      this.currentServers.forEach(server => {
        if (
          server.status !== 'offline' &&
          !this.pingInProgress.has(server.id)
        ) {
          this.pingServer(server);
        }
      });
    }, 5000);

    this.logger.info('Ping service started');
  }

  private stopPingService(): void {
    if (this.pingInterval) {
      clearInterval(this.pingInterval);
      this.pingInterval = null;
    }

    this.pingInProgress.clear();
    this.logger.info('Ping service stopped');
  }

  private async pingServer(server: Server): Promise<void> {
    if (!this.client || this.pingInProgress.has(server.id)) {
      return;
    }

    this.pingInProgress.add(server.id);

    try {
      const addr = `${server.address}:${server.port}`;
      const pong = await this.client.ping(addr);

      const serverData: ServerData = {
        players: parseInt(pong.players, 10) || 0,
        maxPlayers: parseInt(pong.maxPlayers, 10) || 0,
        motd: [pong.motd, pong.subMotd],
        lastPing: Date.now(),
      };

      this.updateServerData(server.id, serverData);
      this.updateServerStatus(server.id, 'online');
    } catch (error) {
      this.logger.info(`Failed to ping server ${server.name}:`, error);
      this.updateServerStatus(server.id, 'offline');
    } finally {
      this.pingInProgress.delete(server.id);
    }
  }

  private updateServerStatus(serverId: string, status: ServerStatus): void {
    this.currentServers = this.currentServers.map(server =>
      server.id === serverId ? {...server, status} : server,
    );

    this.notifyCallbacks();
  }

  private updateServerData(serverId: string, data: ServerData): void {
    console.log('Updating server data:', serverId, data);

    this.currentServers = this.currentServers.map(server =>
      server.id === serverId ? {...server, data} : server,
    );

    this.notifyCallbacks();
  }
}

// Singleton instance
export const serverStateService = new ServerStateService();
