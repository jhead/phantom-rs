import React from 'react';
import {View, Text, FlatList} from 'react-native';
import {Server} from '../services/serverState';
import {styles} from './ServersScreen.styles';
import {ServerRow} from './ServerRow';

interface ServerListProps {
  servers: Server[];
}

export const ServerList: React.FC<ServerListProps> = ({servers}) => {
  const handleServerPress = (server: Server) => {
    console.log(`Server pressed: ${server.name}`);
    // TODO: Implement server selection/connection logic
  };

  return (
    <FlatList
      data={servers}
      renderItem={({item}) => (
        <ServerRow server={item} onPress={() => handleServerPress(item)} />
      )}
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
