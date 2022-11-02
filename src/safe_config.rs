use std::env;

#[derive(Clone)]
pub(crate) struct SafeConfig {
    pub(crate) rpc_url: String,
    pub(crate) backend_private_key: String,
    pub(crate) fallback_addr: String,
    pub(crate) master_copy_addr: String,
    pub(crate) proxy_factory_addr: String,
    pub(crate) salt_nonce: String,
}

impl SafeConfig {
    pub(crate) fn new() -> Self {
        let rpc_url = env::var("RPC_URL").expect("RPC_URL must be set");
        let backend_private_key = env::var("BACKEND_PRIVATE_KEY")
            .expect("BACKEND_PRIVATE_KEY must be set");
        let fallback_addr = env::var("FALLBACK_ADDRESS")
            .expect("FALLBACK_ADDRESS must be set");
        let master_copy_addr = env::var("MASTER_COPY_CONTRACT_ADDRESS")
            .expect("MASTER_COPY_CONTRACT_ADDRESS must be set");
        let proxy_factory_addr = env::var("PROXY_FACTORY_CONTRACT_ADDRESS")
            .expect("PROXY_FACTORY_CONTRACT_ADDRESS must be set");
        let salt_nonce = env::var("SALT_NONCE")
            .expect("SALT_NONCE must be set");

        Self {
            rpc_url,
            backend_private_key,
            fallback_addr,
            master_copy_addr,
            proxy_factory_addr,
            salt_nonce,
        }
    }
}