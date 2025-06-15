import React, {useEffect} from 'react';
import {SafeAreaView, StatusBar} from 'react-native';
import {ServersScreen} from './components/ServersScreen';
import {styles} from './components/ServersScreen.styles';
import {appLifecycleManager} from './services/appLifecycle';

function App() {
  useEffect(() => {
    console.log('[App] Initializing app lifecycle manager');
    appLifecycleManager.init();

    // Cleanup on unmount (though this rarely happens in React Native)
    return () => {
      console.log('[App] Cleaning up app lifecycle manager');
      appLifecycleManager.cleanup();
    };
  }, []);

  return (
    <SafeAreaView style={styles.container}>
      <StatusBar barStyle="light-content" backgroundColor="#18171c" />
      <ServersScreen />
    </SafeAreaView>
  );
}

export default App;
