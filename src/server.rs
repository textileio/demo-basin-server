use log::info;
use warp::Filter;

use crate::Cli;

use routes::all_routes;
use util::log_request_details;

mod list;
mod routes;
mod set;
mod shared;
mod util;

/// Server entrypoint for the faucet service.
pub async fn run(cli: Cli) -> anyhow::Result<()> {
    let private_key = cli.private_key;
    let os_address = cli.os_address;
    let network = cli.network;
    let listen_addr = cli.listen;

    let log_request_details = warp::log::custom(log_request_details);

    let router = all_routes(private_key.clone(), os_address.clone(), network.clone())
        .with(
            warp::cors()
                .allow_any_origin()
                .allow_headers(vec!["Content-Type"])
                .allow_methods(vec!["POST"]),
        )
        .with(log_request_details)
        .recover(shared::handle_rejection);

    info!("Starting server at {}", listen_addr);
    warp::serve(router).run(listen_addr).await;
    Ok(())
}
