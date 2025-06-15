import {TurboModule} from 'react-native';
import {TurboModuleRegistry} from 'react-native';

export interface Spec extends TurboModule {
  create(opts: string): void;
}

export default TurboModuleRegistry.getEnforcing<Spec>('NativePhantom');
