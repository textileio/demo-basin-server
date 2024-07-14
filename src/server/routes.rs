use fendermint_crypto::SecretKey;
use warp::{Filter, Rejection, Reply};

use crate::server::get::get_route;
use crate::server::set::set_route;

pub fn all_routes(
    private_key: SecretKey,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    get_route(private_key.clone()).or(set_route(private_key))
}
