use std::sync::Arc;

use async_trait::async_trait;
use ethers::abi::Token;
use ethers::contract::builders::ContractCall;
use ethers::core::k256::SecretKey;
use ethers::prelude::*;
use ethers::prelude::k256::ecdsa::SigningKey;
use ethers::providers::Provider;
use ethers::utils::{hex, keccak256};
use log::debug;

use crate::ethers_ext::solidity_keccak256;
use crate::safe::{Safe, SafeError, SafeInfo, SafeResponse};
use crate::safe_config::SafeConfig;

// not the best idea, bruh
#[macro_use]
pub(self) mod helper_macro {
    #[macro_export]  macro_rules! as_addr_err {
    ($ee: expr) => {
        $ee.map_err(|e| SafeError::BadAddress(format!("to {e}")))?
    }
}

    #[macro_export]  macro_rules! as_u256_err {
    ($ee: expr) => {
        $ee.map_err(|e| SafeError::BadParams(format!("to {e}")))?
    }
}

    #[macro_export] macro_rules! as_rpc_err {
    ($ee: expr) => {
        $ee.map_err(|e| SafeError::RpcError(format!("to {e}")))?
    }
}
}

const THRESHOLD: usize = 1;

type Signer = SignerMiddleware<Provider<Http>, Wallet<SigningKey>>;

abigen!(
        ProxyFactory, "./abi/proxy_factory_abi.json";
        MasterCopy, "./abi/safe_master_abi.json";
    );

#[derive(Clone)]
pub(crate) struct SafeService {
    provider: Provider<Http>,
    client: Arc<Signer>,
    fallback_addr: Address,
    master_copy_addr: Address,
    master_copy: MasterCopy<Signer>,
    proxy_factory_addr: Address,
    proxy_factory: ProxyFactory<Signer>,
    salt_nonce: Vec<u8>,
}

enum Operation {
    Call = 0,
    DelegateCall,
}

impl TryFrom<u8> for Operation {
    type Error = SafeError;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            _ if Operation::Call as u8 == v => Ok(Operation::Call),
            _ if Operation::DelegateCall as u8 == v => Ok(Operation::DelegateCall),
            _ => Err(SafeError::BadParams(format!("Unknown Operation enum variant {v}")))
        }
    }
}

impl SafeService {
    pub(crate) async fn new(safe_config: SafeConfig) -> Self {
        let provider = Provider::<Http>::try_from(safe_config.rpc_url).unwrap();
        debug!("Provider's chain id is {:?}", provider.get_chainid().await.unwrap());

        let secret_key = SecretKey::from_be_bytes(
            hex::decode(safe_config.backend_private_key).unwrap().as_slice()
        ).unwrap();
        let signing_key = SigningKey::from(secret_key);
        let signer = LocalWallet::from(signing_key);

        let client = SignerMiddleware::new_with_provider_chain(
            provider.clone(),
            signer,
        ).await.unwrap();

        let client = Arc::new(client);

        let fallback_addr = safe_config.fallback_addr.parse::<Address>().unwrap();
        let master_copy_addr = safe_config.master_copy_addr.parse::<Address>().unwrap();
        let proxy_factory_addr = safe_config.proxy_factory_addr.parse::<Address>().unwrap();

        let proxy_factory = ProxyFactory::new(proxy_factory_addr, client.clone());
        let master_copy = MasterCopy::new(master_copy_addr, client.clone());
        let salt_nonce = hex::decode(safe_config.salt_nonce).unwrap();

        Self {
            provider,
            client,
            fallback_addr,
            master_copy_addr,
            master_copy,
            proxy_factory_addr,
            proxy_factory,
            salt_nonce,
        }
    }

    async fn calculate_address(&self, user_address: &str) -> Result<Address, SafeError> {
        let user_address = as_addr_err!(user_address.parse::<Address>());
        let initializer = self.encode_initializer(user_address)?;
        let initializer_hash = keccak256(initializer);
        debug!("Initializer hash: {:?}", ethers::utils::hex::encode(&initializer_hash));

        let salt = solidity_keccak256(&[
            Token::Bytes(initializer_hash.to_vec()),
            Token::Uint(U256::from(self.salt_nonce.as_slice())),
        ]);
        debug!("Salt: {:?}", ethers::utils::hex::encode(&salt));

        let init_code: Bytes = as_rpc_err!(self.proxy_factory.proxy_creation_code().call().await);
        let init_code_hash = solidity_keccak256(&[
            Token::Bytes(init_code.to_vec()),
            Token::Uint(U256::from(self.master_copy_addr.as_bytes())),
        ]);
        debug!("Init code hash: {:?}", ethers::utils::hex::encode(&init_code_hash));

        let create2_address = ethers::utils::get_create2_address_from_hash(
            self.proxy_factory_addr,
            salt,
            init_code_hash,
        );
        debug!("Create2 address: {}", ethers::utils::to_checksum(&create2_address, None));
        Ok(create2_address)
    }

    async fn is_deployed(&self, address: Address) -> Result<bool, SafeError> {
        let code = as_rpc_err!(self.provider.get_code(address, None).await);
        let code = hex::encode(&code);
        debug!("Code from address {:?}: {}", address, code);
        Ok(!code.is_empty())
    }

    fn encode_initializer(&self, user_address: Address) -> Result<Bytes, SafeError> {
        let tokens: &[Token] = &[
            Token::Array(vec![Token::Address(user_address)]), // owners
            Token::Uint(U256::from(THRESHOLD)), // threshold
            Token::Address(Address::zero()), // to
            Token::Bytes(vec![]), // data
            Token::Address(self.fallback_addr), // fallbackHandler
            Token::Address(Address::zero()), // paymentToken
            Token::Uint(U256::from(0)), // payment
            Token::Address(Address::zero()) // paymentReceiver
        ];

        let encoded_initializer = as_rpc_err!(self.master_copy.encode("setup", tokens));
        debug!("Encoded initializer: {:?}", ethers::utils::hex::encode(&encoded_initializer));
        Ok(encoded_initializer)
    }
}

#[async_trait]
impl Safe for SafeService {
    async fn info(&self, user_address: &str) -> Result<SafeInfo, SafeError> {
        let address = self.calculate_address(user_address).await?;
        let is_deployed = self.is_deployed(address).await?;
        let address = ethers::utils::to_checksum(&address, None);
        Ok(SafeInfo {
            address,
            is_deployed,
        })
    }

    async fn deploy(&self, user_address: &str) -> Result<SafeResponse, SafeError> {
        if self.info(user_address).await?.is_deployed {
            return Err(SafeError::AlreadyExists);
        }
        let user_address = user_address.parse::<Address>().unwrap();

        let mut receipt: TransactionReceipt = as_rpc_err!(as_rpc_err!(self.proxy_factory.create_proxy_with_nonce(
            self.master_copy_addr,
            self.encode_initializer(user_address)?,
            U256::from(self.salt_nonce.as_slice()),
        ).send().await).await).unwrap();

        let Log { block_hash, transaction_hash, .. } = receipt.logs.pop().unwrap();
        debug!("Receipt of deployment: {:?}", receipt);

        Ok(SafeResponse {
            block_hash: format!("0x{}", hex::encode(block_hash.unwrap())),
            transaction_hash: format!("0x{}", hex::encode(transaction_hash.unwrap())),
        })
    }

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
                  signatures: Vec<u8>) -> Result<SafeResponse, SafeError> {
        let SafeInfo { address, is_deployed } = self.info(user_address).await?;
        if !is_deployed {
            return Err(SafeError::NotDeployed);
        }

        let _ = Operation::try_from(operation)?;

        let master_copy = MasterCopy::new(as_addr_err!(address.parse::<Address>()), self.client.clone());

        let contract_call: ContractCall<_, _> = master_copy.exec_transaction(
            as_addr_err!(to.parse::<Address>()),
            as_u256_err!(U256::from_dec_str(value)),
            Bytes::from(data),
            operation,
            as_u256_err!(U256::from_dec_str(safe_tx_gas)),
            as_u256_err!(U256::from_dec_str(base_gas)),
            as_u256_err!(U256::from_dec_str(gas_price)),
            as_addr_err!(gas_token.parse::<Address>()),
            as_addr_err!(refund_receiver.parse::<Address>()),
            Bytes::from(signatures),
        );

        let mut receipt: TransactionReceipt = as_rpc_err!(as_rpc_err!(contract_call.send().await).await).unwrap();

        let Log { block_hash, transaction_hash, .. } = receipt.logs.pop().unwrap();
        debug!("Receipt of exec_transaction: {:?}", receipt);

        Ok(SafeResponse {
            block_hash: format!("0x{}", hex::encode(block_hash.unwrap())),
            transaction_hash: format!("0x{}", hex::encode(transaction_hash.unwrap())),
        })
    }
}
