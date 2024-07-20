use std::error::Error;
use std::ops::Deref;

use bytes::buf;
use ethers::prelude::TransactionReceipt;
use fendermint_crypto::SecretKey;
use fvm_shared::address::Address;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::io::AsyncWriteExt;
use warp::{http::Response, Filter, Rejection, Reply};

use adm_provider::json_rpc::JsonRpcProvider;
use adm_sdk::{
    account::Account,
    machine::{objectstore::ObjectStore, Machine},
    network::Network as SdkNetwork,
};

use crate::server::{
    shared::{get_faucet_wallet, with_private_key, BadRequest, BaseRequest },
    util::log_request_body,
};

use super::shared::{with_network, with_os_address};

/// List request (essentially, equivalent to [`BaseRequest`]).
#[derive(Deserialize)]
pub struct ListRequest {
    #[serde(flatten)]
    pub base: BaseRequest,
}

impl std::fmt::Display for ListRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.base)
    }
}

impl Deref for ListRequest {
    type Target = BaseRequest;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

/// Route filter for `/list` endpoint.
pub fn list_route(
    os_address: Address,
    network: SdkNetwork,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("list")
        .and(warp::post())
        .and(warp::header::exact("content-type", "application/json"))
        .and(warp::body::json())
        .and(with_os_address(os_address.clone()))
        .and(with_network(network.clone()))
        .and_then(handle_list)
}

/// Handles the `/list` request, first initializing the network.
pub async fn handle_list(
    req: ListRequest,
    os_address: Address,
    network: SdkNetwork,
) -> anyhow::Result<impl Reply, Rejection> {
    let net = network.init();
    let os = ObjectStore::attach(os_address);
    log_request_body("list", &format!("{}", req));

    let object_list = list(net, os, req).await.map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("list error: {}", e),
        })
    })?;
    let objects = object_list
        .objects
        .iter()
        .map(|(key_bytes, object)| {
            let key = core::str::from_utf8(&key_bytes).unwrap_or_default().to_string();                    
            let cid = cid::Cid::try_from(object.cid.clone().0).unwrap_or_default();                    
            let value = json!({"cid": cid.to_string(), "resolved": object.resolved, "size": object.size, "metadata": object.metadata});
            json!({"key": key, "value": value})
        })
        .collect::<Vec<Value>>();
        
    let json = json!(objects);
    Ok(warp::reply::json(&json))
}

/// List keys in the object store.
pub async fn list(
    network: &SdkNetwork,
    os: ObjectStore,
    req: ListRequest,
) -> anyhow::Result<fendermint_actor_objectstore::ObjectList, Box<dyn Error>> {
    let provider =
        JsonRpcProvider::new_http(network.rpc_url()?, None, Some(network.object_api_url()?))?;
    // let mut buf = tokio::io::BufWriter::new(tokio::fs::File::create("temp").await?);

    let list = os.query(&provider, Default::default()).await?;

    Ok(list)
}
