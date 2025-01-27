use serde::Deserialize;

use zksync_env_config::{envy_load, FromEnv};

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct AvailConfig {
    pub api_node_url: Option<String>,
    pub bridge_api_url: String,
    pub seed: Option<String>,
    pub app_id: Option<u32>,
    pub timeout: usize,
    pub max_retries: usize,
    pub gas_relay_mode: bool,
    pub gas_relay_api_url: Option<String>,
    pub gas_relay_api_key: Option<String>,
}

impl FromEnv for AvailConfig {
    fn from_env() -> anyhow::Result<Self> {
        envy_load("avail_client", "AVAIL_CLIENT_")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avail::AvailConfig;
    use da_utils::test_utils::EnvMutex;

    static MUTEX: EnvMutex = EnvMutex::new();

    fn expected_avail_da_layer_config(
        seed: &str,
        api_node_url: &str,
        bridge_api_url: &str,
        app_id: u32,
        timeout: usize,
        max_retries: usize,
        gas_relay_mode: bool,
        gas_relay_api_url: &str,
        gas_relay_api_key: &str,
    ) -> AvailConfig {
        AvailConfig {
            // if api_node_url is of length 0, then set it None
            api_node_url: if api_node_url.is_empty() {
                None
            } else {
                Some(api_node_url.to_string())
            },
            bridge_api_url: bridge_api_url.to_string(),
            seed: if seed.is_empty() {
                None
            } else {
                Some(seed.to_string())
            },
            app_id: if app_id == 0 { None } else { Some(app_id) },
            timeout,
            max_retries,
            gas_relay_mode,
            gas_relay_api_url: if gas_relay_api_url.is_empty() {
                None
            } else {
                Some(gas_relay_api_url.to_string())
            },
            gas_relay_api_key: if gas_relay_api_key.is_empty() {
                None
            } else {
                Some(gas_relay_api_key.to_string())
            },
        }
    }

    #[test]
    fn from_env_avail_client() {
        let mut lock = MUTEX.lock();
        let config = r#"
            AVAIL_CLIENT_API_NODE_URL="localhost:12345"
            AVAIL_CLIENT_BRIDGE_API_URL="localhost:54321"
            AVAIL_CLIENT_SEED="bottom drive obey lake curtain smoke basket hold race lonely fit walk"
            AVAIL_CLIENT_APP_ID=1
            AVAIL_CLIENT_TIMEOUT=2
            AVAIL_CLIENT_MAX_RETRIES=3
            AVAIL_CLIENT_GAS_RELAY_MODE=true
            AVAIL_CLIENT_GAS_RELAY_API_URL="localhost:65432"
            AVAIL_CLIENT_GAS_RELAY_API_KEY="key"
        "#;
        unsafe {
            lock.set_env(config);
        }
        let actual = AvailConfig::from_env().unwrap();
        assert_eq!(
            actual,
            expected_avail_da_layer_config(
                "bottom drive obey lake curtain smoke basket hold race lonely fit walk",
                "localhost:12345",
                "localhost:54321",
                "1".parse::<u32>().unwrap(),
                "2".parse::<usize>().unwrap(),
                "3".parse::<usize>().unwrap(),
                true,
                "localhost:65432",
                "key"
            )
        );
    }
}
