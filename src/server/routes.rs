use adm_sdk::network::Network as SdkNetwork;
use fendermint_crypto::SecretKey;
use fvm_shared::address::Address;
use warp::{Filter, Rejection, Reply};

use crate::server::list::list_route;
use crate::server::set::set_route;

pub fn all_routes(
    private_key: SecretKey,
    os_address: Address,
    network: SdkNetwork,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    list_route(os_address, network).or(set_route(private_key, os_address, network))
}
