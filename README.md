# GPROXY
An async reverse proxy for video games using deep packet inspection to grab a target hostname and forward it to the correct server.

## Currently supported games:
 - Minecraft -- Version 1.6 and up

# Setup 
## Installation
Prerequisets:
- Rust
- Git

1. Run `git clone https://github.com/lukeo102/gproxy.git`
2. `cd` into `gproxy`
3. Run `cargo build --release`
4. Use the compiled binary in `./target/release/gproxy` to run the proxy

## Config
### Location
By default, the server looks for a `config.json` file in `/etc/grpoxy/config.json`, this can be changed by setting the environment variable `CONFIG_LOCATION=/path/to/config/file/`. It must be a directory containing the file `config.json`.

### Layout
An example layout can be found in [config.json](./config.json)
```
{
    "minecraft": {
      "mapping": {
        "hostname": [
          "server_address",
          port
        ],
      }
    }
}
```
