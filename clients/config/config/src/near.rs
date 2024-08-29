use serde::Deserialize;

use zksync_env_config::{envy_load, FromEnv};

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct NearConfig {
    pub evm_provider_url: String,
    pub network: String,
    pub contract: String,
    pub bridge_contract: String,
    pub account_id: String,
    pub secret_key: String,
}

impl FromEnv for NearConfig {
    fn from_env() -> anyhow::Result<Self> {
        envy_load("near_client", "NEAR_CLIENT_")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::near::NearConfig;
    use da_utils::test_utils::EnvMutex;

    static MUTEX: EnvMutex = EnvMutex::new();

    fn expected_near_da_layer_config(
        evm_provider_url: &str,
        network: &str,
        contract: &str,
        bridge_contract: &str,
        account_id: &str,
        secret_key: &str,
    ) -> NearConfig {
        NearConfig {
            evm_provider_url: evm_provider_url.to_string(),
            network: network.to_string(),
            contract: contract.to_string(),
            bridge_contract: bridge_contract.to_string(),
            account_id: account_id.to_string(),
            secret_key: secret_key.to_string(),
        }
    }

    #[test]
    fn from_env_near_client() {
        let mut lock = MUTEX.lock();
        let config = r#"
            NEAR_CLIENT_evm_provider_url="localhost:12345"
            NEAR_CLIENT_NETWORK="mainnet"
            NEAR_CLIENT_CONTRACT="blob-store.testnet"
            NEAR_CLIENT_BRIDGE_CONTRACT="0x0000000000000000000000000000000000000001"
            NEAR_CLIENT_ACCOUNT_ID="nearuser.testnet"
            NEAR_CLIENT_SECRET_KEY="3D1vMSgusRrUsihyCnACWeMvLdq3ZfusFXrnU3jnSaWvexHuATq4T5EUyUasEM6C1WTdd87ArM5yYAAcn3sTrY6s"
        "#;
        unsafe {
            lock.set_env(config);
        }
        let actual = NearConfig::from_env().unwrap();
        assert_eq!(
            actual,
            expected_near_da_layer_config("localhost:12345", "mainnet", "blob-store.testnet", "0x0000000000000000000000000000000000000001", "nearuser.testnet", "3D1vMSgusRrUsihyCnACWeMvLdq3ZfusFXrnU3jnSaWvexHuATq4T5EUyUasEM6C1WTdd87ArM5yYAAcn3sTrY6s")
        );
    }
}
