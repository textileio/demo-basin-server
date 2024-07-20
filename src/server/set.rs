use std::error::Error;
use std::ops::Deref;

use adm_sdk::machine::Machine;
use bytes::Buf;
use fendermint_crypto::SecretKey;
use futures::TryStreamExt;
use fvm_shared::address::Address;
use serde::Deserialize;
use serde_json::json;
use warp::{multipart::FormData, Filter, Rejection, Reply};

use adm_provider::{json_rpc::JsonRpcProvider, response::Cid, tx::TxReceipt, util::parse_address};
use adm_sdk::{machine::objectstore::ObjectStore, network::Network as SdkNetwork};

use crate::server::{
    shared::{get_faucet_wallet, with_private_key, BadRequest, BaseRequest},
    util::log_request_body,
};

use super::shared::{deserialize_address, with_network, with_os_address};

/// Maximum file size for uploaded files.
const MAX_FILE_SIZE: u64 = 1024 * 1024 * 100; // 100 MB

#[derive(Deserialize)]
pub struct FileData {
    pub filename: String,
    pub content: Vec<u8>,
}

/// Set request (a [`BaseRequest`] plus file path and content).
#[derive(Deserialize)]
pub struct SetRequest {
    #[serde(flatten)]
    pub base: BaseRequest,
    /// The address (0x prefix or t4/f4) of the account making the request.
    #[serde(deserialize_with = "deserialize_address")]
    pub address: Address,
    /// The value to set at the given key (string or file).
    pub value: FileData,
}

impl std::fmt::Display for FileData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "filename: {}, content: {:?}",
            self.filename, self.content
        )
    }
}

impl std::fmt::Display for SetRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, address, {}, value: {}",
            self.base, self.address, self.value
        )
    }
}

impl Deref for SetRequest {
    type Target = BaseRequest;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

/// Route filter for `/set` endpoint.
pub fn set_route(
    private_key: SecretKey,
    os_address: Address,
    network: SdkNetwork,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("set")
        .and(warp::post())
        .and(warp::header::header("content-type"))
        .and(warp::multipart::form().max_length(MAX_FILE_SIZE)) // Adjust max_length as needed
        .and(with_private_key(private_key))
        .and(with_os_address(os_address))
        .and(with_network(network))
        .and_then(validate_content_type)
}

async fn validate_content_type(
    content_type: String,
    form: warp::multipart::FormData,
    private_key: SecretKey,
    os_address: Address,
    network: SdkNetwork,
) -> Result<impl Reply, Rejection> {
    log_request_body("set", &format!("{:?}", content_type));
    if content_type.starts_with("multipart/form-data") {
        handle_set(form, private_key, os_address, network).await
    } else {
        Err(warp::reject::custom(BadRequest {
            message: "Invalid Content-Type".to_string(),
        }))
    }
}

pub async fn handle_set(
    form: FormData,
    private_key: SecretKey,
    os_address: Address,
    network: SdkNetwork,
) -> Result<impl Reply, Rejection> {
    let net = network.init();
    let os = ObjectStore::attach(os_address);

    let mut filename: Option<String> = None;
    let mut file_content: Vec<u8> = Vec::new();
    let mut address: Option<Address> = None;
    let mut key: Option<String> = None;

    let mut parts = form;

    while let Ok(Some(mut part)) = parts.try_next().await {
        let field = part.name().to_string();
        if field == "file" {
            filename = part.filename().map(String::from);
            while let Some(content) = part.data().await {
                match content {
                    Ok(content) => {
                        file_content.extend_from_slice(content.chunk());
                    }
                    Err(e) => {
                        eprintln!("form error: {}", e);
                        return Err(warp::reject::reject());
                    }
                }
            }
        } else {
            let mut bytes: Vec<u8> = Vec::new();
            while let Some(content) = part.data().await {
                match content {
                    Ok(content) => {
                        bytes.extend_from_slice(content.chunk());
                    }
                    Err(e) => {
                        eprintln!("form error: {}", e);
                        return Err(warp::reject::reject());
                    }
                }
            }
            match field.as_str() {
                "address" => {
                    let addr = String::from_utf8_lossy(&bytes).to_string();
                    address = Some(parse_address(&addr).unwrap());
                }
                "key" => key = Some(String::from_utf8_lossy(&bytes).to_string()),
                _ => (),
            }
        }
    }

    let address = address.ok_or_else(warp::reject::reject)?;
    let key = key.ok_or_else(warp::reject::reject)?;

    let base = BaseRequest { key };
    let value = FileData {
        filename: filename.unwrap_or_default(),
        content: file_content,
    };
    let set_req = SetRequest {
        base,
        address,
        value,
    };

    let res = set(*net, os, private_key, set_req).await.map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("set error: {}", e),
        })
    })?;
    let json = json!(res);

    Ok(warp::reply::json(&json))
}

/// Set key with data as value on the subnet.
pub async fn set(
    network: SdkNetwork,
    os: ObjectStore,
    private_key: SecretKey,
    req: SetRequest,
) -> anyhow::Result<TxReceipt<Cid>, Box<dyn Error>> {
    let mut signer = get_faucet_wallet(private_key, network)?;
    let provider =
        JsonRpcProvider::new_http(network.rpc_url()?, None, Some(network.object_api_url()?))?;
    let file = async_tempfile::TempFile::new().await?;

    signer.init_sequence(&provider).await?;
    let tx = os
        .add(
            &provider,
            &mut signer,
            req.key.as_str(),
            file,
            Default::default(),
        )
        .await?;
    Ok(tx)
}
