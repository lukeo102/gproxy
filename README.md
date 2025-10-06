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

# How it works
## Packet inspection for the hostname
The first Vanilla and Fabric Minecraft packet sent from the client to the server is a handshake packet that contains a variable length hostname block at byte position 6, this length is determined by a byte at position 5.
We can then extract the hostname from the packet by doing the following:
```rust
// See lines 43 & 44 in src/minecraft/minecraft.rs
address_segment_len = packet[4]
packet[5..address_segment_len + 5]
```

Following this we can match the hostname from the packet to one of the hostnames in the config.

This only covers Vanilla and Fabric servers, for Forge servers they include the version of the Forge mod loader they use appended to the end of the hostname block.
This only minorly changes how we extract the hostname to exclude this metadata:
```rust
// See lines 47 & 48 in src/minecraft/minecraft.rs
address_segment_len = packet[4]
packet[5..address_segment_len]
```

## Wont this break encryption?
This proxy only inspects the first packet, after which it is handed off to a simple TCP proxy (seen in lines 100 to 136 in src/proxy.rs).
This first packet is not encrypted, and as the proxy does not inspect other packets, just passes them between the client and server, it does not interfere with the encryption.
