import {useEffect, useState, useCallback} from 'react';
import {serverStateService, Server} from '../services/serverState';

export const useServers = () => {
  const [servers, setServers] = useState<Server[]>([]);
  const [isInitialized, setIsInitialized] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Initialize the service on first mount
  useEffect(() => {
    let unsubscribe: (() => void) | null = null;

    const initializeService = async () => {
      try {
        console.log('[useServers] Initializing server state service');

        // Subscribe to updates before initializing
        unsubscribe = serverStateService.onUpdate(setServers);

        // Initialize the service
        await serverStateService.initialize();

        // Get initial servers
        const initialServers = serverStateService.getServers();
        setServers(initialServers);
        setIsInitialized(true);
        setError(null);

        console.log('[useServers] Service initialized successfully');
      } catch (err) {
        console.error('[useServers] Failed to initialize service:', err);
        setError(
          err instanceof Error
            ? err.message
            : 'Failed to initialize server service',
        );
        setIsInitialized(false);
      }
    };

    initializeService();

    // Cleanup on unmount
    return () => {
      console.log('[useServers] Cleaning up service');
      if (unsubscribe) {
        unsubscribe();
      }
      // Note: We don't shutdown the service here since it's a singleton
      // that might be used by other components
    };
  }, []);

  const addServer = useCallback(
    async (serverData: Omit<Server, 'status' | 'data' | 'autoStart'>) => {
      try {
        console.log('[useServers] Adding server:', serverData);
        await serverStateService.addServer(serverData);
        setError(null);
      } catch (err) {
        console.error('[useServers] Failed to add server:', err);
        setError(err instanceof Error ? err.message : 'Failed to add server');
        throw err;
      }
    },
    [],
  );

  const removeServer = useCallback(async (serverId: string) => {
    try {
      console.log('[useServers] Removing server:', serverId);
      await serverStateService.removeServer(serverId);
      setError(null);
    } catch (err) {
      console.error('[useServers] Failed to remove server:', err);
      setError(err instanceof Error ? err.message : 'Failed to remove server');
      throw err;
    }
  }, []);

  const updateServerAutoStart = useCallback(
    async (serverId: string, autoStart: boolean) => {
      try {
        console.log(
          '[useServers] Updating server auto-start:',
          serverId,
          autoStart,
        );
        await serverStateService.updateServerAutoStart(serverId, autoStart);
        setError(null);
      } catch (err) {
        console.error('[useServers] Failed to update server auto-start:', err);
        setError(
          err instanceof Error ? err.message : 'Failed to update server',
        );
        throw err;
      }
    },
    [],
  );

  const updateServer = useCallback(
    async (
      serverId: string,
      updates: Partial<Omit<Server, 'id' | 'status' | 'data'>>,
    ) => {
      try {
        console.log('[useServers] Updating server:', serverId, updates);
        await serverStateService.updateServer(serverId, updates);
        setError(null);
      } catch (err) {
        console.error('[useServers] Failed to update server:', err);
        setError(
          err instanceof Error ? err.message : 'Failed to update server',
        );
        throw err;
      }
    },
    [],
  );

  return {
    servers,
    isInitialized,
    error,
    addServer,
    removeServer,
    updateServerAutoStart,
    updateServer,
  };
};
