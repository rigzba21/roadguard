# roadguard
A Rust binary for easy [road warrior](https://en.wikipedia.org/wiki/Road_warrior_(computing)) 
[Wireguard](https://www.wireguard.com/) VPN setup. 

## Quickstart
At this time `roadguard` only runs on Linux.

#### Prerequisites
* Wireguard
    * See [Wireguard Installation Instructions](https://www.wireguard.com/install/)

### Setup a WireGuard Server with Default Values
In a terminal run:

```bash
cargo build --release
sudo ./target/release/roadguard setup
```

This configures the system to act as a WireGuard server with IPTables rules configured 
to allow regular internet access via traffic forwarding. 

## Development

There is a `Containerfile` for easy development and testing.

Build the container:
```bash
podman build -t roadguard:dev
```

Run it:
```bash
podman run --rm -it roadguard:dev sudo ./roadguard setup
```




