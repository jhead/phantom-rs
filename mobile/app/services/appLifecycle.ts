import {AppState, AppStateStatus} from 'react-native';
import {serverStateService} from './serverState';

class AppLifecycleManager {
  private isInitialized = false;
  private appStateSubscription: any = null;

  init() {
    if (this.isInitialized) return;

    console.log('[AppLifecycle] Initializing app lifecycle manager');

    // Handle app state changes
    this.appStateSubscription = AppState.addEventListener(
      'change',
      this.handleAppStateChange.bind(this),
    );

    this.isInitialized = true;
  }

  cleanup() {
    if (!this.isInitialized) return;

    console.log('[AppLifecycle] Cleaning up app lifecycle manager');

    if (this.appStateSubscription) {
      this.appStateSubscription.remove();
      this.appStateSubscription = null;
    }

    this.isInitialized = false;
  }

  private async handleAppStateChange(nextAppState: AppStateStatus) {
    console.log('[AppLifecycle] App state changed to:', nextAppState);

    try {
      if (nextAppState === 'background' || nextAppState === 'inactive') {
        // App is going to background, potentially cleanup resources
        console.log('[AppLifecycle] App going to background');
      } else if (nextAppState === 'active') {
        // App is becoming active, potentially reinitialize resources
        console.log('[AppLifecycle] App becoming active');
      }
    } catch (error) {
      console.error('[AppLifecycle] Error handling app state change:', error);
    }
  }

  async shutdown() {
    console.log('[AppLifecycle] Shutting down app');

    try {
      // Cleanup server state service
      await serverStateService.shutdown();

      // Cleanup lifecycle manager
      this.cleanup();

      console.log('[AppLifecycle] App shutdown complete');
    } catch (error) {
      console.error('[AppLifecycle] Error during shutdown:', error);
    }
  }
}

export const appLifecycleManager = new AppLifecycleManager();
