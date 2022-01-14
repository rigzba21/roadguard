use clap::Parser;

use std::process::{Command, Stdio};
use std::fs::File;
use std::os::unix::fs::PermissionsExt;
use std::fs;
use std::io::{Write};

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
   Setup,
   AddClient,
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
        }
    }
}

/// Write key files for persistant usage
fn write_key_file(key: String, file: String) -> std::io::Result<()> {
    let mut file = File::create(file)?;
    file.write_all(key.as_bytes())?;
    Ok(())
}

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
            println!("PRIVATE KEY: {}", wg_private_key);
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
            println!("PUBLIC KEY: {}", wg_public_key);
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
            println!("DEFAULT INTERFACE: {}", default_device);
            return Ok(default_device);
        }
        _ => {
            let std_err = String::from_utf8(output.stderr).unwrap();
            println!("{:#?}", std_err);
            return Err(WgInitErrors::FailedToGetDefaultDevice)
        }
    }
}

fn generate_wg0_conf(ip: String, interface: String) {
    let private_key = fs::read_to_string("server_private_key")
        .expect("Error Reading server_private_key");
    
    let wg0_conf = format!(
"[Interface]
Address = {}/24
SaveConfig = true
PrivateKey = {}
ListenPort = 51900
DNS = 1.1.1.1

PostUp = iptables -A FORWARD -i %i -j ACCEPT; iptables -A FORWARD -o %i -j ACCEPT; iptables -t nat -A POSTROUTING -o {} -j MASQUERADE
PostDown = iptables -D FORWARD -i %i -j ACCEPT; iptables -D FORWARD -o %i -j ACCEPT; iptables -t nat -D POSTROUTING -o {} -j MASQUERADE
", ip, private_key, interface, interface);

    println!("{}", wg0_conf);

    let result = write_wg0_conf(wg0_conf, String::from("/etc/wireguard/wg0.conf"));

    match result {
        Ok(()) => {
            println!("Successfully Wrote wg0.conf");
        }
        _ => eprintln!("Error Writing /etc/wireguard/wg0.conf, please run: \n sudo roadguard setup")
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
            
       } 
       RoadGuardAction::AddClient => {
        // TODO
        println!("This functionality is a WIP");
       }
       RoadGuardAction::RemoveClient => {
        // TODO
        println!("This functionality is a WIP");
       }
    }

}