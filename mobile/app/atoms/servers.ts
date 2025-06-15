import {atom} from 'jotai';

export type ServerStatus = 'online' | 'offline' | 'connecting';

export interface ServerData {
  players: number;
  maxPlayers: number;
  motd: [string, string];
  icon?: string;
  lastPing?: number; // Unix timestamp in milliseconds
}

export interface Server {
  id: string;
  name: string;
  address: string;
  port: string;
  status: ServerStatus;
  data?: ServerData;
}

export const serversAtom = atom<Server[]>([]);

// Derived atom for adding a new server
export const addServerAtom = atom(
  null,
  (get, set, newServer: Omit<Server, 'status' | 'data'>) => {
    const currentServers = get(serversAtom);
    set(serversAtom, [
      ...currentServers,
      {
        ...newServer,
        status: 'connecting',
      },
    ]);
  },
);

// Derived atom for updating server data
export const updateServerDataAtom = atom(
  null,
  (get, set, {id, data}: {id: string; data: ServerData}) => {
    const currentServers = get(serversAtom);
    set(
      serversAtom,
      currentServers.map(server =>
        server.id === id
          ? {
              ...server,
              data,
              status: 'online' as ServerStatus,
            }
          : server,
      ),
    );
  },
);

// Derived atom for updating server status
export const updateServerStatusAtom = atom(
  null,
  (get, set, {id, status}: {id: string; status: ServerStatus}) => {
    const currentServers = get(serversAtom);
    set(
      serversAtom,
      currentServers.map(server =>
        server.id === id
          ? {
              ...server,
              status,
            }
          : server,
      ),
    );
  },
);
