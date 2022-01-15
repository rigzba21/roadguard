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

There is a `Vagrantfile` for easy development and testing.

Run the Vagrant machine:
```bash
vagrant up
```

SSH Into the Vagrant machine:
```bash
vagrant ssh
```

Make sure Rust is installed:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Build the `roadguard` binary:
```bash
cd /vagrant

cargo build --release
```

Run `roadguard` setup:
```bash
sudo ./target/release/roadguard setup
```

Verify that `wg0` is running:
```bash
sudo wg-quick up wg0

sudo wg show
```


