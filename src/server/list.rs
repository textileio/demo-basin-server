use std::error::Error;

use fvm_shared::address::Address;
use serde::Deserialize;
use serde_json::{json, Value};
use warp::{Filter, Rejection, Reply};

use adm_provider::json_rpc::JsonRpcProvider;
use adm_sdk::{
    machine::{objectstore::ObjectStore, objectstore::QueryOptions, Machine},
    network::Network as SdkNetwork,
};

use crate::server::{shared::BadRequest, util::log_request_body};

use super::shared::{with_network, with_os_address};

/// List request options.
#[derive(Deserialize, Default)]
pub struct ListRequest {
    /// The prefix to filter objects by.
    pub prefix: Option<String>,
    /// The delimiter used to define object hierarchy.
    pub delimiter: Option<String>,
    /// The offset to start listing objects from.
    pub offset: Option<u64>,
    /// The maximum number of objects to list.
    pub limit: Option<u64>,
}

impl std::fmt::Display for ListRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "prefix: {:?}, delimiter: {:?}, offset: {:?}, limit: {:?}",
            self.prefix, self.delimiter, self.offset, self.limit
        )
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
        .and(with_list_body())
        .and(with_os_address(os_address))
        .and(with_network(network))
        .and_then(handle_list)
}

// Custom filter to allow for empty request body and use defaults.
fn with_list_body() -> impl Filter<Extract = (ListRequest,), Error = Rejection> + Clone {
    warp::body::json()
        .map(Some)
        .or_else(|_| async { Ok::<(Option<ListRequest>,), Rejection>((None,)) })
        .map(|body: Option<ListRequest>| body.unwrap_or_default())
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
            let key = String::from_utf8_lossy(key_bytes).to_string();
            let cid = cid::Cid::try_from(object.cid.clone().0).unwrap_or_default();
            let value = json!({"cid": cid.to_string(), "resolved": object.resolved, "size": object.size, "metadata": object.metadata});
            json!({"key": key, "value": value})
        })
        .collect::<Vec<Value>>();
    let common_prefixes = object_list
        .common_prefixes
        .iter()
        .map(|prefix_bytes| String::from_utf8_lossy(prefix_bytes).to_string())
        .collect::<Vec<String>>();

    let json = json!({
        "objects": objects,
        "common_prefixes": common_prefixes
    });
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
    let options = QueryOptions {
        prefix: req.prefix.unwrap_or_default(),
        delimiter: req.delimiter.unwrap_or_default(),
        offset: req.offset.unwrap_or_default(),
        limit: req.limit.unwrap_or_default(),
        height: Default::default(),
    };

    let list = os.query(&provider, options).await?;
    Ok(list)
}
