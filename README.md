# roadguard
A Rust binary for easy [road warrior](https://en.wikipedia.org/wiki/Road_warrior_(computing)) 
[Wireguard](https://www.wireguard.com/) VPN setup. 

## Quickstart
At this time `roadguard` only runs on Linux.

**Ubuntu 21.04 Quickstart**

Install dependencies:
```bash
sudo apt-get install -y \
    wireguard \
    resolvconf \
    build-essential \
    qrencode
```

Install `roadguard`:
```bash
wget https://github.com/rigzba21/roadguard/releases/download/v0.0.2/roadguard-x86_64 -O roadguard
sudo install -m 755 roadguard /usr/local/bin/roadguard
```

Setup WireGuard as a server with traffic port-forwarding for regular internet access:
```bash
sudo roadguard setup
```

Enable the `wg0` interface:
```bash
sudo wg-quick up wg0
```

Verify that it's up and running:
```bash
sudo wg show
```

At this point, you will have to set up a public endpoint for your WireGuard server. If you are running this on a cloud instance like EC2, you can either use the public IP address (or public DNS) or set up a DNS
record with a domain using your DNS Provider of choice. 

If you run the server on your network, such as a Raspberry Pi, you will need to configure your router to port-forward traffic on `51900`. Once you've configured port-forwarding to your server, you'll need to set up a DNS record that points towards your public IP address or set up a Dynamic DNS.


Once you've configured a public endpoint (Dynamic DNS, Public IP, etc.), you can now add a client (peer).

Adding a client (peer) and generate a config file named `MY-CLIENT-NAME.conf`:
```bash
sudo roadguard --endpoint MY-SERVER-ENDPOINT add-client
```
You'll be prompted to enter in a name for this client, and will generate a config file named 
`MY-CLIENT-NAME.conf`.

If you've installed `qrencode` you can generate a QR code for easy mobile device configuration
with the WireGuard mobile apps:
```bash
qrencode -t ansiutf8 -r MY-CLIENT-NAME.conf
```

## Development Setup

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

# License

[MIT License](https://github.com/rigzba21/roadguard/blob/main/LICENSE)

