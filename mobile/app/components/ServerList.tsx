import React, {useCallback} from 'react';
import {View, Text, FlatList} from 'react-native';
import {Server} from '../services/serverState';
import {styles} from './ServersScreen.styles';
import {ServerRow} from './ServerRow';

interface ServerListProps {
  servers: Server[];
  onDeleteServer: (serverId: string) => Promise<void>;
  onEditServer: (server: Server) => void;
}

export const ServerList: React.FC<ServerListProps> = ({
  servers,
  onDeleteServer,
  onEditServer,
}) => {
  const handleServerPress = useCallback((server: Server) => {
    console.log(`Server pressed: ${server.name}`);
    // TODO: Implement server selection/connection logic
  }, []);

  const handleDeleteServer = useCallback(
    async (server: Server) => {
      console.log(`Deleting server: ${server.name}`);
      try {
        await onDeleteServer(server.id);
        console.log(`Server ${server.name} deleted successfully`);
      } catch (error) {
        console.error(`Failed to delete server ${server.name}:`, error);
      }
    },
    [onDeleteServer],
  );

  const renderServerRow = useCallback(
    ({item}: {item: Server}) => (
      <ServerRow
        server={item}
        onPress={() => handleServerPress(item)}
        onDelete={() => handleDeleteServer(item)}
        onEdit={() => onEditServer(item)}
      />
    ),
    [handleServerPress, handleDeleteServer, onEditServer],
  );

  return (
    <View style={styles.serversListContainer}>
      <FlatList
        data={servers}
        renderItem={renderServerRow}
        keyExtractor={item => item.id}
        style={styles.serverList}
        ListEmptyComponent={
          <View style={styles.emptyState}>
            <Text style={styles.emptyStateText}>No servers added yet</Text>
          </View>
        }
      />
    </View>
  );
};
