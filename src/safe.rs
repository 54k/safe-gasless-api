use std::fmt::{Display, Formatter};

use async_trait::async_trait;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SafeInfo {
    pub(crate) address: String,
    pub(crate) is_deployed: bool,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SafeResponse {
    pub(crate) block_hash: String,
    pub(crate) transaction_hash: String,
}

#[derive(Debug, Clone)]
pub(crate) enum SafeError {
    AlreadyExists,
    NotDeployed,
    BadAddress(String),
    BadParams(String),
    RpcError(String),
}

impl Display for SafeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SafeError::AlreadyExists => write!(f, "Safe is already deployed"),
            SafeError::NotDeployed => write!(f, "Safe is not deployed"),
            SafeError::BadAddress(e) => write!(f, "Invalid address: {}", e),
            SafeError::BadParams(e) => write!(f, "Bad parameters passed: {}", e),
            SafeError::RpcError(e) => write!(f, "Rpc unavailable: {}", e)
        }
    }
}

impl std::error::Error for SafeError {}

#[async_trait]
pub(crate) trait Safe {
    async fn info(&self, user_address: &str) -> Result<SafeInfo, SafeError>;

    async fn deploy(&self, user_address: &str) -> Result<SafeResponse, SafeError>;

    #[allow(clippy::too_many_arguments)]
    async fn exec(&self,
                  user_address: &str,
                  to: &str,
                  value: &str,
                  data: Vec<u8>,
                  operation: u8,
                  safe_tx_gas: &str,
                  base_gas: &str,
                  gas_price: &str,
                  gas_token: &str,
                  refund_receiver: &str,
                  signatures: Vec<u8>) -> Result<SafeResponse, SafeError>;
}