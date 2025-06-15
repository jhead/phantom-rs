import React, {useState} from 'react';
import {View, Text, TouchableOpacity} from 'react-native';
import {styles} from './ServersScreen.styles';
import {PhantomLogo} from './PhantomLogo';
import {ServerList} from './ServerList';
import {AddServerModal} from './AddServerModal';
import {useServers} from '../hooks/useServers';

export const ServersScreen: React.FC = () => {
  const [isAddModalVisible, setIsAddModalVisible] = useState(false);
  const {servers, addServer, isInitialized, error} = useServers();

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
        <TouchableOpacity
          style={styles.addButton}
          onPress={() => setIsAddModalVisible(true)}>
          <Text style={styles.addButtonText}>+</Text>
        </TouchableOpacity>
      </View>

      <Text style={styles.serversTitle}>Servers</Text>
      <View style={styles.serversListContainer}>
        <ServerList servers={servers} />
      </View>

      <AddServerModal
        visible={isAddModalVisible}
        onClose={() => setIsAddModalVisible(false)}
        addServer={addServer}
      />
    </View>
  );
};
