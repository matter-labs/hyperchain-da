use serde::Deserialize;

use zksync_env_config::{envy_load, FromEnv};

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct NearConfig {
    pub light_client_url: String,
    pub network: String,
    pub contract: String,
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
    use crate::clients::near::config::NearConfig;
    use crate::test_utils::EnvMutex;

    static MUTEX: EnvMutex = EnvMutex::new();

    fn expected_near_da_layer_config(
        light_client_url: &str,
        network: &str,
        contract: &str,
        account_id: &str,
        secret_key: &str,
    ) -> NearConfig {
        NearConfig {
            light_client_url: light_client_url.to_string(),
            network: network.to_string(),
            contract: contract.to_string(),
            account_id: account_id.to_string(),
            secret_key: secret_key.to_string(),
        }
    }

    #[test]
    fn from_env_near_client() {
        let mut lock = MUTEX.lock();
        let config = r#"
            NEAR_CLIENT_LIGHT_CLIENT_URL="localhost:12345"
            NEAR_CLIENT_NETWORK="mainnet"
            NEAR_CLIENT_CONTRACT="test"
            NEAR_CLIENT_ACCOUNT_ID="test"
            NEAR_CLIENT_SECRET_KEY="test"
        "#;
        lock.set_env(config);
        let actual = NearConfig::from_env().unwrap();
        assert_eq!(
            actual,
            expected_near_da_layer_config("localhost:12345", "mainnet", "test", "test", "test")
        );
    }
}
