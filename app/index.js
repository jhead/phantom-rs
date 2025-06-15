/**
 * @format
 */

import {AppRegistry} from 'react-native';
import App from './App';
import {name as appName} from './app.json';
import { uniffiInitAsync } from 'react-native-phantom';

uniffiInitAsync().then(() => {
   AppRegistry.registerComponent(appName, () => App);
});
