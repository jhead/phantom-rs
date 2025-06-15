# Phantom

WIP rust port of [jhead/phantom](https://github.com/jhead/phantom) + mobile apps for iOS and Android

## CLI

```
$ phantom-cli --help
Usage: phantom-cli [OPTIONS] --server <SERVER>

Options:
  -s, --server <SERVER>        Bedrock/MCPE server IP address and port (ex: 1.2.3.4:19132)
      --bind <BIND>            IP address to listen on. Defaults to all interfaces [default: 0.0.0.0]
      --bind-port <BIND_PORT>  Port to listen on. Defaults to 0, which selects a random port. Note that phantom always binds to port 19132 as well, so both ports need to be open [default: 0]
      --timeout <TIMEOUT>      Seconds to wait before cleaning up a disconnected client [default: 60]
      --debug                  Enables debug logging
  -6, --ipv6                   Enables IPv6 support on port 19133 (experimental)
  -h, --help                   Print help
  -V, --version                Print version
```

## Project Layout

- `phantom-rs/`: Core Rust library with FFI bindings
- `phantom-cli/`: Phantom CLI
- `mobile/`
  - `app/`: Phantom React Native mobile app
  - `react-native-phantom/`: React Native wrapper of phantom-rs Rust library

## Building

TBD
