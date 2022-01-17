use clap::Parser;

use std::process::{Command, Stdio};
use std::fs::File;
use std::os::unix::fs::PermissionsExt;
use std::fs;
use std::io::{Write};
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::process::exit;

#[derive(Debug)]
pub enum WgInitErrors {
    FailedToGenPrivateKey,
    FailedToGenPublicKey,
    FailedToGetDefaultDevice,
    FailedToWriteWG0Conf,
}

#[derive(Debug)]
#[derive(clap::Subcommand)]
enum RoadGuardAction {

   /// Setup host as a WireGuard Server and allow for regular internet traffic through port-forwarding 
   Setup,

   /// Generate a new client (peer) and add to the Server config, and generate a QR code for mobile app use
   AddClient,

   /// Remove an existing client (peer) from the Server config
   RemoveClient,
}

#[derive(Parser, Debug)]
#[clap(author = "Jon V [rigzba21]", version = "0.0.1", about = "roadguard - Setup a Road Warrior Style Wireguard VPN", long_about = None)]
struct Args {
    /// IP Address to set the Wireguard Server
    #[clap(short, long, default_value = "10.253.3.1")]
    ip: String,

    /// Subcommand to generate client config
    #[clap(subcommand)]
    action: RoadGuardAction,

    /// [Required for 'add-client'] Public Endpoint (Pubilc IP or DNS) of the WireGuard Server
    #[clap(short, long, default_value = "")]
    endpoint: String,
}

/// Generate the WireGuard Server Private Key
fn generate_private_key() {
    let wg_genkey_status = wg_genkey();
    match wg_genkey_status {
        Ok(private_key) => {
            write_key_file(private_key, String::from("server_private_key")).unwrap();
        }
        _ => {
            println!("{:#?}", WgInitErrors::FailedToGenPrivateKey);
            exit(1);
        }
    }
}

/// Write key files for persistant usage
fn write_key_file(key: String, file: String) -> std::io::Result<()> {
    let mut file = File::create(file)?;
    file.write_all(key.as_bytes())?;
    Ok(())
}

/// Write the wg0.conf file using the 077 file permissions
fn write_wg0_conf(wg0_conf: String, file: String) -> std::io::Result<()> {
    let mut file = File::create(file)?;
    let metadata = file.metadata()?;
    let mut permissions = metadata.permissions();

    // umask 077
    permissions.set_mode(0o077); 
    assert_eq!(permissions.mode(), 0o077);

    file.write_all(wg0_conf.as_bytes())?;
    Ok(())
}

/// Execute the wg genkey command to return a generated private key
fn wg_genkey() -> Result<String, WgInitErrors> {
    let output = Command::new("wg")
        .arg("genkey")
        .output()
        .expect("failed to execute process");

    let status_code = output.status.code();

    match status_code {
        Some(0) => {
            let wg_private_key = String::from_utf8(output.stdout).unwrap();
            println!("PRIVATE KEY: {}", wg_private_key.replace("\n", ""));
            return Ok(wg_private_key);
        }
        _ => {
            let std_err = String::from_utf8(output.stderr).unwrap();
            println!("{:#?}", std_err);
            return Err(WgInitErrors::FailedToGenPrivateKey)
        }
    }

}

/// Execute the wg pubkey command to return the server public key
fn wg_pubkey() -> Result<String, WgInitErrors>{
    let _cat_private_key = Command::new("cat")
        .arg("server_private_key")
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to cat server_private_key");

    let output = Command::new("wg")
        .arg("pubkey")
        .stdin(_cat_private_key.stdout.unwrap())
        .output()
        .expect("failed to execute command \"wg pubkey\"");

    let status_code = output.status.code();

    match status_code {
        Some(0) => {
            let wg_public_key = String::from_utf8(output.stdout).unwrap();
            println!("PUBLIC KEY: {}", wg_public_key.replace("\n", ""));
            return Ok(wg_public_key);
        }
        _ => {
            let std_err = String::from_utf8(output.stderr).unwrap();
            println!("{:#?}", std_err);
            return Err(WgInitErrors::FailedToGenPrivateKey)
        }
    }
}

/// Generate the WireGuard Server Public Key
fn generate_public_key() {
    let wg_pubkey_status = wg_pubkey();
    match wg_pubkey_status {
        Ok(public_key) => {
            write_key_file(public_key, String::from("server_public_key")).unwrap();
        }
        _ => {
            println!("{:#?}", WgInitErrors::FailedToGenPrivateKey);
        }
    }
}

/// Get the Default Network Interface Device Name
fn get_default_ip_dev() -> Result<String, WgInitErrors> {
    let _ip_route_show_to_default = Command::new("ip")
        .arg("-o")
        .arg("-4")
        .arg("route")
        .arg("show")
        .arg("to")
        .arg("default")
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to run the command \"ip -o -4 show to default\"");

    let output = Command::new("awk")
        .arg("{print $5}")
        .stdin(_ip_route_show_to_default.stdout.unwrap())
        .output()
        .expect("failed to execute command \"awk \'{print $5}\'\"");

    let _status_code = output.status.code();

    match _status_code {
        Some(0) => {
            let default_device = String::from_utf8(output.stdout).unwrap();
            println!("DEFAULT INTERFACE: {}", default_device.replace("\n", ""));
            return Ok(default_device);
        }
        _ => {
            let std_err = String::from_utf8(output.stderr).unwrap();
            println!("{:#?}", std_err);
            return Err(WgInitErrors::FailedToGetDefaultDevice)
        }
    }
}

/// Generate the wg0.conf file contents
fn generate_wg0_conf(ip: String, interface: String) {
    let private_key = fs::read_to_string("server_private_key")
        .expect("Error Reading server_private_key");
    
    // removing newlines
    let ip_val = ip.replace("\n", "");
    let private_key_val = private_key.replace("\n", "");
    let interface_val = interface.replace("\n", "");
    
    // TODO: make this more configurable...
    let wg0_conf = format!(
"[Interface]
Address = {}/24
SaveConfig = true
PrivateKey = {}
ListenPort = 51900
DNS = 1.1.1.1

PostUp = iptables -A FORWARD -i %i -j ACCEPT; iptables -A FORWARD -o %i -j ACCEPT; iptables -t nat -A POSTROUTING -o {} -j MASQUERADE
PostDown = iptables -D FORWARD -i %i -j ACCEPT; iptables -D FORWARD -o %i -j ACCEPT; iptables -t nat -D POSTROUTING -o {} -j MASQUERADE
", ip_val, private_key_val, interface_val, interface_val);

    println!("{}", wg0_conf);

    let result = write_wg0_conf(wg0_conf, String::from("/etc/wireguard/wg0.conf"));

    match result {
        Ok(()) => {
            println!("Successfully Wrote wg0.conf");
        }
        _ => {
            eprintln!("Error Writing /etc/wireguard/wg0.conf, please run: \n sudo roadguard setup");
            exit(1);
        }
    }
}

/// Setup port-forarding in /etc/sysct.conf
fn setup_port_forwarding() {
    let output = Command::new("sed")
        .arg("-i")
        .arg("/net.ipv4.ip_forward=1/s/^#//g")
        .arg("/etc/sysctl.conf")
        .output()
        .expect("failed to execute process");

    let status_code = output.status.code();

    match status_code {
        Some(0) => {
            println!("Successfully configured traffic port-forwarding in /etc/sysctl.conf")
        }
        _ => {
            eprintln!("Error configuring port-forwarding in /etc/sysctl.conf\n {:#?}", String::from_utf8(output.stderr).unwrap());
            exit(1);
        }
    }
}

/// Reload sysctl
fn reload_sysctl() {
    let output = Command::new("sysctl")
    .arg("-p")
    .output()
    .expect("failed to execute process");

    let status_code = output.status.code();
    match status_code {
        Some(0) => println!("Successfully reloaded sysctl"),
        _ => {
            eprintln!("Error reloading sysctl");
            exit(1);
        }
    }
}

/// Enable WireGuard on Startup
fn enable_wg_on_startup() {
    let systemctl_output = Command::new("systemctl")
    .arg("enable")
    .arg("wg-quick@wg0")
    .output()
    .expect("failed to execute process");

    let systemctl_enable_status_code = systemctl_output.status.code();
    match systemctl_enable_status_code {
        Some(0) =>  {
            println!("Successfully ran: systemctl enable wg-quick@wg0");
        }
        _ => {
            eprintln!("Error running systemctl enable wg-quick@wg0\n {:#?}", String::from_utf8(systemctl_output.stderr).unwrap());
            exit(1);
        }
    }

    let chown_root = Command::new("chown")
    .arg("-R")
    .arg("root:root")
    .arg("/etc/wireguard/")
    .output()
    .expect("failed to execute process");

    let chown_root_code = chown_root.status.code();
    match chown_root_code {
        Some(0) => println!("Successfully ran: chown -R root:root /etc/wireguard/"),
        _ => {
            eprintln!("Error running: chown -R root:root /etc/wireguard/\n {:#?}", String::from_utf8(chown_root.stderr).unwrap());
            exit(1);
        }
    }

    let chmod_permissions = Command::new("chmod")
    .arg("-R")
    .arg("og-rwx")
    .arg("/etc/wireguard/wg0.conf")
    .output()
    .expect("failed to execute process");

    let chmod_permissions_status = chmod_permissions.status.code();
    match chmod_permissions_status {
        Some(0) => println!("Successfully ran: chmod -R og-rwx /etc/wireguard/*"),
        _ => {
            eprintln!("Error running: chmod -R og-rwx /etc/wireguard/*\n {:#?}", String::from_utf8(chmod_permissions.stderr).unwrap());
            exit(1);
        }
    }
}

/// Generate a new Client Configuration
fn wg_client_config(server_ip: String, endpoint: String) {
    let _client_private_key = Command::new("wg")
        .arg("genkey")
        .output()
        .expect("Error running wg genkey");

    let client_private_key = String::from_utf8(_client_private_key.stdout).unwrap().replace("\n", "");

    let echo_private_key = Command::new("echo")
        .arg(client_private_key.clone())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Error echo'ing private key to stdio");

    let output = Command::new("wg")
        .arg("pubkey")
        .stdin(echo_private_key.stdout.unwrap())
        .output()
        .expect("failed to generate client public key");
    
    let client_public_key = String::from_utf8(output.stdout).unwrap().replace("\n", "");

    let _client = get_client_name();

    let peers_count = get_num_peers();

    let client_ip = generate_client_ip(peers_count, server_ip);

    let _config = format!(
    "[Interface]
PrivateKey = {}
Address = {}/32
DNS = 1.1.1.1

[Peer]
PublicKey = {}
AllowedIPs = 0.0.0.0/0, ::/0
Endpoint = {}:51900
", client_private_key, client_ip, client_public_key, endpoint);

    println!("{}", _config);

    let client_config_filename = format!("{}.conf", _client);
    let result = write_key_file(_config, client_config_filename);

    match result {
        Ok(()) => {
            println!("Successfully Wrote client config file");
            wg0_set_peer(client_public_key, client_ip);
        }
        _ => {
            eprintln!("Error Writing client config file");
            exit(1);
        }
    }

}

/// Helper function to get input for a client config name
fn get_client_name() -> String {
    let mut client_name = String::new();
    println!("Enter Name of Client:");
    let _line_bytes = std::io::stdin().read_line(&mut client_name).unwrap();
    return client_name.replace("\n", "")
}

/// Helper function to get the currently number of Wireguard Peers
fn get_num_peers() -> i32 {
    let wg_show = Command::new("wg")
        .arg("show")
        .stdout(Stdio::piped())
        .spawn()
        .expect("Error running wg show");

    let grep_peer = Command::new("grep")
        .arg("peer")
        .stdin(wg_show.stdout.unwrap())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Error grepping");
    
    let wc_l = Command::new("wc")
        .arg("-l")
        .stdin(grep_peer.stdout.unwrap())
        .output()
        .expect("Error running wc -l");

    let num_clients_str = String::from_utf8(wc_l.stdout).unwrap().replace("\n", "");
    let num_clients: i32 = num_clients_str.parse().unwrap();
    num_clients
}

/// Generate an IP adress for a new client
fn generate_client_ip(_peer_count: i32, server_ip: String) -> String {
    let ip_addr = Ipv4Addr::from_str(&server_ip).unwrap();
    let octets = ip_addr.octets();

    let new_octet: u8 = (_peer_count + 2).try_into().unwrap();
    let new_ip: [u8; 4] = [octets[0], octets[1], octets[2], new_octet];
    let client_ip = Ipv4Addr::to_string(&Ipv4Addr::from(new_ip));
    return client_ip;
}

fn wg0_set_peer(public_key: String, allowed_ip: String) {
    let wg0_set_peer = Command::new("wg")
    .arg("set")
    .arg("wg0")
    .arg("peer")
    .arg(public_key)
    .arg("allowed-ips")
    .arg(allowed_ip)
    .output()
    .expect("Error running wc -l");

    let status_code = wg0_set_peer.status.code();
    match status_code {
        Some(0) => {
            println!("Successfully set wg0 peer");
        }
        _ => {
            eprintln!("Error running wg set wg0 peer");
            eprintln!("{:#?}", String::from_utf8(wg0_set_peer.stderr).unwrap());
            exit(1);
        }
    }
}

fn main() {
    let args = Args::parse();

    let subcommand = args.action;

    match subcommand {
       RoadGuardAction::Setup => {
            let ip = args.ip;
            println!("IP ADDRESS: {}", ip);

            generate_private_key();

            generate_public_key();
        
            let default_interface = get_default_ip_dev().unwrap();
            
            generate_wg0_conf(ip, default_interface);

            setup_port_forwarding();

            reload_sysctl();
            
            enable_wg_on_startup();
       } 
       RoadGuardAction::AddClient => {
            let _ip = args.ip;
            let _endpoint = args.endpoint;

            if _endpoint == "" {
                eprintln!("--endpoint <MY-ENDPOINT> flag not valid\n Please use: roadguard --endpoint MY-ENDPOINT add-client");
                exit(1);
            }

            wg_client_config(_ip, _endpoint);
            println!("To generate a QR code for easy mobile app configuration, run the following:\n");
            println!("qrencode -t ansiutf8 -r MY-CLIENT.conf");
       }
       RoadGuardAction::RemoveClient => {
        // TODO
        println!("This functionality is a WIP");
       }
    }
}