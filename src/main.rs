use clap::Parser;

use std::process::Command;
use std::fs::File;
use std::io::{Write};

#[derive(Debug)]
pub enum WgInitErrors {
    FailedToGenPrivateKey,
    FailedToGenPublicKey,
}

#[derive(Parser, Debug)]
#[clap(author = "Jon V [rigzba21]", version = "0.0.1", about = "roadguard - Setup a Road Warrior Style Wireguard VPN", long_about = None)]
struct Args {
    /// IP Address to set the Wireguard Server
    #[clap(short, long, default_value = "10.253.3.1")]
    ip: String,
}

fn main() {
    let args = Args::parse();
    println!("Setting Wireguard Server to: {}", args.ip);

    generate_private_key();
}

/// Generate the WireGuard Server Private Key
fn generate_private_key() {
    let wg_genkey_status = wg_genkey();
    match wg_genkey_status {
        Ok(private_key) => {
            write_private_key_file(private_key).unwrap();
        }
        _ => {
            println!("{:#?}", WgInitErrors::FailedToGenPrivateKey);
        }
    }
}

/// Write our private key to a file for basic persistence 
fn write_private_key_file(key: String) -> std::io::Result<()> {
    println!("{:#?}", key);
    let mut file = File::create("server_private_key")?;
    file.write_all(key.as_bytes())?;
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
            return Ok(wg_private_key);
        }
        _ => {
            let std_err = String::from_utf8(output.stderr).unwrap();
            println!("{:#?}", std_err);
            return Err(WgInitErrors::FailedToGenPrivateKey)
        }
    }

}


