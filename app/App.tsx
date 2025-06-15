import React, {useState} from 'react';
import {
  SafeAreaView,
  StatusBar,
  Text,
  TouchableOpacity,
  View,
} from 'react-native';
import {AddServerModal} from './components/AddServerModal';
import {PhantomLogo} from './components/PhantomLogo';
import {ServerList} from './components/ServerList';
import {styles} from './components/ServersScreen.styles';

function App() {
  const [modalVisible, setModalVisible] = useState(false);

  return (
    <SafeAreaView style={styles.container}>
      <StatusBar barStyle="light-content" backgroundColor="#18171c" />
      <View style={styles.header}>
        <PhantomLogo />
        <Text style={styles.headerTitle}>phantom</Text>
        <TouchableOpacity
          style={styles.addButton}
          onPress={() => setModalVisible(true)}>
          <Text style={styles.addButtonText}>+</Text>
        </TouchableOpacity>
      </View>
      <Text style={styles.serversTitle}>Servers</Text>
      <View style={styles.serversListContainer}>
        <ServerList />
      </View>
      <View style={styles.bottomNav}>
        <View style={styles.navIconActive} />
        <View style={styles.navIcon} />
      </View>
      <AddServerModal
        visible={modalVisible}
        onClose={() => setModalVisible(false)}
      />
    </SafeAreaView>
  );
}

export default App;
