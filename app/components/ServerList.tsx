import React, {useEffect} from 'react';
import {View, Text, FlatList, TouchableOpacity} from 'react-native';
import {useAtom, useSetAtom} from 'jotai';
import {serversAtom, Server, ServerStatus} from '../atoms/servers';
import {managePhantomInstanceAtom} from '../atoms/phantom';
import {styles} from './ServersScreen.styles';

const StatusIndicator: React.FC<{status: ServerStatus}> = ({status}) => {
  const getStatusColor = () => {
    switch (status) {
      case 'online':
        return '#4CAF50';
      case 'offline':
        return '#F44336';
      case 'connecting':
        return '#FFC107';
    }
  };

  return (
    <View
      style={[
        styles.statusIndicator,
        {
          backgroundColor: getStatusColor(),
        },
      ]}
    />
  );
};

const ServerIcon: React.FC<{icon?: string}> = ({icon}) => {
  if (!icon) {
    return (
      <View style={styles.serverIconPlaceholder}>
        <View style={styles.serverIconInner}>
          <View style={styles.serverIconLine} />
          <View style={[styles.serverIconLine, {width: '60%'}]} />
          <View style={[styles.serverIconLine, {width: '80%'}]} />
        </View>
      </View>
    );
  }

  return (
    <View style={styles.serverIcon}>
      <View style={styles.serverIconInner}>
        <View style={styles.serverIconLine} />
        <View style={[styles.serverIconLine, {width: '60%'}]} />
        <View style={[styles.serverIconLine, {width: '80%'}]} />
      </View>
    </View>
  );
};

const PlayerCount: React.FC<{
  players?: number;
  maxPlayers?: number;
}> = ({players, maxPlayers}) => {
  if (players === undefined || maxPlayers === undefined) {
    return (
      <View style={styles.playerCountPlaceholder}>
        <View style={styles.playerCountSkeleton} />
      </View>
    );
  }

  return (
    <Text style={styles.playerCount}>
      {players}/{maxPlayers} players
    </Text>
  );
};

const MOTD: React.FC<{motd?: [string, string]}> = ({motd}) => {
  if (!motd) {
    return (
      <View style={styles.motdPlaceholder}>
        <View style={styles.motdSkeleton} />
        <View style={[styles.motdSkeleton, {width: '70%'}]} />
      </View>
    );
  }

  return (
    <View style={styles.motdContainer}>
      <Text style={styles.motdLine} numberOfLines={1}>
        {motd[0]}
      </Text>
      <Text style={styles.motdLine} numberOfLines={1}>
        {motd[1]}
      </Text>
    </View>
  );
};

export const ServerList: React.FC = () => {
  const [servers] = useAtom(serversAtom);
  const managePhantomInstance = useSetAtom(managePhantomInstanceAtom);

  // Start Phantom instances for new servers
  useEffect(() => {
    const startPhantomInstances = async () => {
      for (const server of servers) {
        if (server.status === 'connecting') {
          try {
            await managePhantomInstance({server, action: 'start'});
          } catch (error) {
            console.error(
              `Failed to start Phantom instance for server ${server.id}:`,
              error,
            );
          }
        }
      }
    };

    startPhantomInstances();
  }, [servers, managePhantomInstance]);

  // Cleanup Phantom instances when servers are removed
  useEffect(() => {
    return () => {
      const cleanupPhantomInstances = async () => {
        for (const server of servers) {
          try {
            await managePhantomInstance({server, action: 'stop'});
          } catch (error) {
            console.error(
              `Failed to stop Phantom instance for server ${server.id}:`,
              error,
            );
          }
        }
      };

      cleanupPhantomInstances();
    };
  }, [servers, managePhantomInstance]);

  const renderServer = ({item}: {item: Server}) => (
    <TouchableOpacity style={styles.serverItem}>
      <View style={styles.serverHeader}>
        <ServerIcon icon={item.data?.icon} />
        <View style={styles.serverInfo}>
          <Text style={styles.serverName}>{item.name}</Text>
          <Text style={styles.serverAddress}>
            {item.address}:{item.port}
          </Text>
        </View>
        <StatusIndicator status={item.status} />
      </View>
      <View style={styles.serverDetails}>
        <PlayerCount
          players={item.data?.players}
          maxPlayers={item.data?.maxPlayers}
        />
        <MOTD motd={item.data?.motd} />
      </View>
    </TouchableOpacity>
  );

  return (
    <FlatList
      data={servers}
      renderItem={renderServer}
      keyExtractor={item => item.id}
      style={styles.serverList}
      ListEmptyComponent={
        <View style={styles.emptyState}>
          <Text style={styles.emptyStateText}>No servers added yet</Text>
        </View>
      }
    />
  );
};
