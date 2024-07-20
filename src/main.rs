use std::net::{SocketAddr, ToSocketAddrs};

use anyhow::anyhow;
use clap::Parser;
use fendermint_crypto::SecretKey;
use fvm_shared::address::Address;
use stderrlog::Timestamp;

use adm_provider::util::parse_address;
use adm_sdk::network::Network as SdkNetwork;
use adm_signer::key::parse_secret_key;

use crate::server::run;

mod server;

#[derive(Clone, Debug, Parser)]
#[command(name = "basin_server", author, version, about, long_about = None)]
struct Cli {
    /// Wallet private key (ECDSA, secp256k1) for sending faucet funds.
    #[arg(short, long, env, value_parser = parse_secret_key)]
    private_key: SecretKey,
    /// Faucet `host:port` string for running the HTTP server.
    #[arg(long, env, value_parser = parse_faucet_url)]
    listen: SocketAddr,
    /// Object store address.
    #[arg(short, long, env, value_parser = parse_address)]
    os_address: Address,
    /// Subnet network type (note: currently hardcoded to `testnet`).
    #[arg(short, long, env, value_parser = parse_network)]
    network: SdkNetwork,
    /// Logging verbosity (repeat for more verbose logging).
    #[arg(short, long, env, action = clap::ArgAction::Count)]
    verbosity: u8,
    /// Silence logging.
    #[arg(short, long, env, default_value_t = false)]
    quiet: bool,
}

/// Parse the [`SocketAddr`] from a faucet URL string.
fn parse_faucet_url(listen: &str) -> anyhow::Result<SocketAddr> {
    match listen.to_socket_addrs()?.next() {
        Some(addr) => Ok(addr),
        None => Err(anyhow!(
            "failed to convert to any socket address: {}",
            listen
        )),
    }
}

/// Parse the [`SocketAddr`] from a faucet URL string.
fn parse_network(_: &str) -> anyhow::Result<SdkNetwork> {
    Ok(SdkNetwork::Testnet)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    stderrlog::new()
        .module(module_path!())
        .quiet(cli.quiet)
        .verbosity(cli.verbosity as usize)
        .timestamp(Timestamp::Millisecond)
        .init()
        .unwrap();

    run(cli).await
}
