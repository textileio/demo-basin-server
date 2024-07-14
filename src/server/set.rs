use std::error::Error;
use std::ops::Deref;

use bytes::Buf;
use ethers::prelude::TransactionReceipt;
use fendermint_crypto::SecretKey;
use futures::TryStreamExt;
use fvm_shared::{address::Address, econ::TokenAmount};
use serde::Deserialize;
use serde_json::json;
use warp::{
    multipart::{FormData, Part},
    Filter, Rejection, Reply,
};

use adm_sdk::{account::Account, network::Network as SdkNetwork};

use crate::server::{
    shared::{get_faucet_wallet, with_private_key, BadRequest, BaseRequest},
    util::log_request_body,
};

/// Maximum file size for uploaded files.
const MAX_FILE_SIZE: u64 = 1024 * 1024 * 100; // 100 MB

/// Setting a value on the subnet (string or file).
#[derive(Deserialize)]
#[serde(untagged)]
pub enum SetValue {
    String(String),
    File(FileData),
}

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
    /// The value to set at the given key.
    pub value: SetValue,
}

impl std::fmt::Display for SetRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.value {
            SetValue::String(s) => write!(f, "{}, value: {}", self.base, s),
            SetValue::File(file) => write!(
                f,
                "{}, file: {}, size: {} bytes",
                self.base,
                file.filename,
                file.content.len()
            ),
        }
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
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("set")
        .and(warp::post())
        .and(warp::header::header("content-type"))
        .and(warp::multipart::form().max_length(MAX_FILE_SIZE)) // Adjust max_length as needed
        .and(with_private_key(private_key.clone()))
        .and_then(validate_content_type)
}

async fn validate_content_type(
    content_type: String,
    form: warp::multipart::FormData,
    private_key: SecretKey,
) -> Result<impl Reply, Rejection> {
    log_request_body("new req", &format!("{:?}", content_type));
    if content_type.starts_with("multipart/form-data") {
        handle_set(form, private_key).await
    } else {
        Err(warp::reject::custom(BadRequest {
            message: "Invalid Content-Type".to_string(),
        }))
    }
}

pub async fn handle_set(form: FormData, private_key: SecretKey) -> Result<impl Reply, Rejection> {
    let mut fields = Vec::new();
    let mut file_content = Vec::new();

    let mut parts = form;

    while let Ok(Some(mut part)) = parts.try_next().await {
        let name = part.name().to_string();

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

        if name == "file" {
            file_content = bytes.clone(); // Store file content separately
        }

        fields.push((name, String::from_utf8_lossy(&bytes).to_string()));
    }

    // Use `field_names` and `file_content` as needed
    let json = json!({
        "fields": fields,
        "file_content": String::from_utf8_lossy(&file_content).to_string(),
    });

    Ok(warp::reply::json(&json))
}

/// Set key with data as value on the subnet.
pub async fn set(
    network: SdkNetwork,
    address: Address,
    key: String,
    value: String,
    private_key: SecretKey,
) -> anyhow::Result<serde_json::Value, Box<dyn Error>> {
    // let signer = get_faucet_wallet(private_key, network)?;
    // let config = network.subnet_config(Default::default())?;
    // let amount = TokenAmount::from_whole(0);
    // let tx = Account::transfer(&signer, address, config, amount).await?;
    let tx = json!("{}");
    Ok(tx)
}
