use serde::Deserialize;

use zksync_env_config::{FromEnv, envy_load};

// feel free to redefine all the fields in this struct, this is just a placeholder
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct CelestiaConfig {
    pub api_node_url: String,
    pub private_key: String,
}

impl FromEnv for CelestiaConfig {
    fn from_env() -> anyhow::Result<Self> {
        envy_load("celestia_client", "CELESTIA_CLIENT_")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::celestia::config::CelestiaConfig;
    use crate::test_utils::EnvMutex;

    static MUTEX: EnvMutex = EnvMutex::new();

    fn expected_celestia_da_layer_config(
        pk: &str,
        api_node_url: &str,
    ) -> CelestiaConfig {
        CelestiaConfig {
            api_node_url: api_node_url.to_string(),
            private_key: pk.to_string(),
        }
    }

    #[test]
    fn from_env_celestia_client() {
        let mut lock = MUTEX.lock();
        let config = r#"
            CELESTIA_CLIENT_API_NODE_URL="localhost:12345"
            CELESTIA_CLIENT_PRIVATE_KEY="0xf55baf7c0e4e33b1d78fbf52f069c426bc36cff1aceb9bc8f45d14c07f034d73"
        "#;
        lock.set_env(config);
        let actual = CelestiaConfig::from_env().unwrap();
        assert_eq!(
            actual,
            expected_celestia_da_layer_config(
                "0xf55baf7c0e4e33b1d78fbf52f069c426bc36cff1aceb9bc8f45d14c07f034d73",
                "localhost:12345",
            )
        );
    }
}
