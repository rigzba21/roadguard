use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author = "Jon V [rigzba21]", version = "0.0.1", about = "roadguard - Setup a Road Warrior Style Wireguard VPN", long_about = None)]
struct Args {
    /// IP Address to set the Wireguard Server
    #[clap(short, long)]
    ip: String,
}

fn main() {
    let args = Args::parse();
    println!("Setting Wireguard Server to: {}", args.ip)
}
