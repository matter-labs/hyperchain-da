use serde::Deserialize;

use zksync_env_config::{FromEnv, envy_load};

// feel free to redefine all the fields in this struct, this is just a placeholder
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct AvailConfig {
    pub api_node_url: String,
    pub seed: String,
    pub app_id: usize,
    pub timeout: usize,
    pub max_retries: usize,
}

impl FromEnv for AvailConfig {
    fn from_env() -> anyhow::Result<Self> {
        envy_load("avail_client", "AVAIL_CLIENT_")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::avail::config::AvailConfig;
    use crate::test_utils::EnvMutex;

    static MUTEX: EnvMutex = EnvMutex::new();

    fn expected_avail_da_layer_config(
        seed: &str,
        api_node_url: &str,
        app_id: usize,
        timeout: usize,
        max_retries: usize,
    ) -> AvailConfig {
        AvailConfig {
            api_node_url: api_node_url.to_string(),
            seed: seed.to_string(),
            app_id,
            timeout,
            max_retries,
        }
    }

    #[test]
    fn from_env_avail_client() {
        let mut lock = MUTEX.lock();
        let config = r#"
            AVAIL_CLIENT_API_NODE_URL="localhost:12345"
            AVAIL_CLIENT_SEED="bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice"
            AVAIL_CLIENT_APP_ID=1
            AVAIL_CLIENT_TIMEOUT=120
            AVAIL_CLIENT_MAX_RETRIES=3
        "#;
        lock.set_env(config);
        let actual = AvailConfig::from_env().unwrap();
        assert_eq!(
            actual,
            expected_avail_da_layer_config(
                "bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice",
                "localhost:12345",
                "1".parse::<usize>().unwrap(),
                "120".parse::<usize>().unwrap(),
                "3".parse::<usize>().unwrap(),
            )
        );
    }
}
