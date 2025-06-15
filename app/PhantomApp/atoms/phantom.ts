import {atom} from 'jotai';
import {Server} from './servers';
import {Phantom, PhantomOpts} from 'react-native-phantom';

export const phantomInstancesAtom = atom<Map<string, Phantom>>(new Map());

export const managePhantomInstanceAtom = atom(
  null,
  async (
    get,
    set,
    {server, action}: {server: Server; action: 'start' | 'stop'},
  ) => {
    const existingPhantom = get(phantomInstancesAtom).get(server.id);
    if (existingPhantom && action === 'stop') {
      console.log('stopping existing phantom', server, existingPhantom);
      await existingPhantom.stop();
      return;
    }

    if (existingPhantom && action === 'start') {
      console.log('starting existing phantom', server, existingPhantom);
      await existingPhantom.start();
      return;
    }

    console.log('creating new phantom', server);
    const opts: PhantomOpts = {
      server: `${server.address}:${server.port}`,
      bind: '0.0.0.0',
      bindPort: 0,
      debug: true,
      ipv6: false,
      timeout: 10000n,
    };

    const phantom = new Phantom(opts);

    try {
      phantom.setLogger({
        logString(str: string) {
          console.log('phantom log', str);
        },
      });
    } catch (e) {}

    await phantom.start();
    set(phantomInstancesAtom, new Map([[server.id, phantom]]));
  },
);
