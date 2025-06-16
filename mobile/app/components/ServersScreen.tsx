import React, {useState, useCallback} from 'react';
import {View, Text, TouchableOpacity} from 'react-native';
import {styles} from './ServersScreen.styles';
import {PhantomLogo} from './PhantomLogo';
import {ServerList} from './ServerList';
import {AddServerModal} from './AddServerModal';
import {EditServerModal} from './EditServerModal';
import {useServers} from '../hooks/useServers';
import {Server} from '../services/serverState';

export const ServersScreen: React.FC = () => {
  const [isAddModalVisible, setIsAddModalVisible] = useState(false);
  const [editingServer, setEditingServer] = useState<Server | null>(null);
  const {servers, addServer, removeServer, updateServer, isInitialized, error} =
    useServers();

  const handleEditServer = useCallback((server: Server) => {
    console.log(`Opening edit modal for server: ${server.name}`);
    setEditingServer(server);
  }, []);

  const handleEditComplete = useCallback(
    async (
      serverId: string,
      updates: Partial<Omit<Server, 'id' | 'status' | 'data'>>,
    ) => {
      try {
        await updateServer(serverId, updates);
        console.log(`Server ${serverId} edited successfully`);
      } catch (error) {
        console.error(`Failed to edit server ${serverId}:`, error);
      } finally {
        setEditingServer(null);
      }
    },
    [updateServer],
  );

  const handleAddModalOpen = useCallback(() => {
    setIsAddModalVisible(true);
  }, []);

  const handleAddModalClose = useCallback(() => {
    setIsAddModalVisible(false);
  }, []);

  const handleEditModalClose = useCallback(() => {
    setEditingServer(null);
  }, []);

  if (!isInitialized) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <PhantomLogo />
          <Text style={styles.headerTitle}>Phantom</Text>
        </View>
        <View style={styles.loadingContainer}>
          <Text style={styles.loadingText}>Initializing...</Text>
        </View>
      </View>
    );
  }

  if (error) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <PhantomLogo />
          <Text style={styles.headerTitle}>Phantom</Text>
        </View>
        <View style={styles.errorContainer}>
          <Text style={styles.errorText}>Error: {error}</Text>
        </View>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <PhantomLogo />
        <Text style={styles.headerTitle}>Phantom</Text>
        <TouchableOpacity style={styles.addButton} onPress={handleAddModalOpen}>
          <Text style={styles.addButtonText}>+</Text>
        </TouchableOpacity>
      </View>

      <Text style={styles.serversTitle}>Servers</Text>
      <View style={styles.serversListContainer}>
        <ServerList
          servers={servers}
          onDeleteServer={removeServer}
          onEditServer={handleEditServer}
        />
      </View>

      <AddServerModal
        visible={isAddModalVisible}
        onClose={handleAddModalClose}
        addServer={addServer}
      />

      <EditServerModal
        visible={editingServer !== null}
        onClose={handleEditModalClose}
        onEdit={handleEditComplete}
        server={editingServer}
      />
    </View>
  );
};
