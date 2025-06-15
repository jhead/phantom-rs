import {Client, ClientInterface, Pong} from 'react-native-phantom';
import {Server, ServerData, ServerStatus} from '../atoms/servers';
import {updateServerDataAtom, updateServerStatusAtom} from '../atoms/servers';
import {useSetAtom} from 'jotai';

let client: ClientInterface | null = null;
const pingInProgress = new Map<string, boolean>();
let pingInterval: NodeJS.Timeout | null = null;
let isRunning = false;
let currentServers: Server[] = [];
let updateServerData:
  | ((params: {id: string; data: ServerData}) => void)
  | null = null;
let updateServerStatus:
  | ((params: {id: string; status: ServerStatus}) => void)
  | null = null;

const getClient = async (): Promise<ClientInterface> => {
  if (!client) {
    client = await Client.create();
  }
  return client;
};

const parsePongToServerData = (pong: Pong): ServerData => {
  return {
    players: parseInt(pong.players, 10) || 0,
    maxPlayers: parseInt(pong.maxPlayers, 10) || 0,
    motd: [pong.motd, pong.subMotd],
  };
};

const pingServer = async (server: Server) => {
  if (!updateServerData || !updateServerStatus) {
    console.error('Ping service not properly initialized');
    return;
  }

  // Skip if already pinging this server
  if (pingInProgress.get(server.id)) {
    console.log(`Skipping ping for server ${server.id} - already in progress`);
    return;
  }

  try {
    pingInProgress.set(server.id, true);
    const client = await getClient();
    const addr = `${server.address}:${server.port}`;
    console.log(`Pinging server ${addr}...`);
    const pong = await client.ping(addr);
    const serverData = parsePongToServerData(pong);
    serverData.lastPing = Date.now();
    updateServerData({id: server.id, data: serverData});
    updateServerStatus({id: server.id, status: 'online'});
    console.log(
      `Server ${addr} is online with ${serverData.players}/${serverData.maxPlayers} players`,
    );
  } catch (error) {
    console.error(
      `Failed to ping server ${server.address}:${server.port}:`,
      error,
    );
    updateServerStatus({id: server.id, status: 'offline'});
  } finally {
    pingInProgress.delete(server.id);
  }
};

const updateServerList = (servers: Server[]) => {
  const newServerIds = new Set(servers.map(s => s.id));
  const currentServerIds = new Set(currentServers.map(s => s.id));

  // Only update if server list actually changed
  if (
    newServerIds.size !== currentServerIds.size ||
    ![...newServerIds].every(id => currentServerIds.has(id))
  ) {
    console.log('Server list changed, updating ping service');
    currentServers = servers;

    // Ping any new servers immediately
    for (const server of servers) {
      if (!currentServerIds.has(server.id) && server.status !== 'offline') {
        pingServer(server);
      }
    }
  }
};

export const startPingService = (
  servers: Server[],
  onUpdateData: (params: {id: string; data: ServerData}) => void,
  onUpdateStatus: (params: {id: string; status: ServerStatus}) => void,
) => {
  if (isRunning) {
    console.log('Ping service already running, updating server list');
    updateServerList(servers);
    return;
  }

  console.log('Starting ping service');
  isRunning = true;
  currentServers = servers;
  updateServerData = onUpdateData;
  updateServerStatus = onUpdateStatus;

  // Initial ping for all servers
  for (const server of servers) {
    if (server.status !== 'offline') {
      pingServer(server);
    }
  }

  // Start periodic pinging
  pingInterval = setInterval(() => {
    for (const server of currentServers) {
      if (server.status !== 'offline') {
        pingServer(server);
      }
    }
  }, 5000);
};

export const stopPingService = () => {
  if (!isRunning) {
    console.log('Ping service not running');
    return;
  }

  console.log('Stopping ping service');
  if (pingInterval) {
    clearInterval(pingInterval);
    pingInterval = null;
  }
  isRunning = false;
  currentServers = [];
  updateServerData = null;
  updateServerStatus = null;
};

export const useServerPing = () => {
  const updateServerData = useSetAtom(updateServerDataAtom);
  const updateServerStatus = useSetAtom(updateServerStatusAtom);

  return {
    pingServer: (server: Server) => pingServer(server),
  };
};
