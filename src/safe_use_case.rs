use std::sync::Arc;

use crate::safe::{Safe, SafeError, SafeResponse};
use crate::SafeInfo;

type SafeType = Arc<dyn Safe + Send + Sync + 'static>;

#[derive(Clone)]
pub(crate) struct SafeUseCase {
    safe: SafeType,
}

impl SafeUseCase {
    pub(crate) fn new(safe: SafeType) -> Self {
        Self {
            safe
        }
    }

    pub(crate) async fn info(&self, user_address: &str) -> Result<SafeInfo, SafeError> {
        self.safe.info(user_address).await
    }

    pub(crate) async fn deploy(&self, user_address: &str) -> Result<SafeResponse, SafeError> {
        self.safe.deploy(user_address).await
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn exec(&self,
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
                             signatures: Vec<u8>) -> Result<SafeResponse, SafeError> {
        self.safe.exec(user_address,
                       to,
                       value,
                       data,
                       operation,
                       safe_tx_gas,
                       base_gas,
                       gas_price,
                       gas_token,
                       refund_receiver,
                       signatures).await
    }
}