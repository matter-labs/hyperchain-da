use serde::Deserialize;

use zksync_env_config::{FromEnv, envy_load};

// feel free to redefine all the fields in this struct, this is just a placeholder
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct CelestiaConfig {
    pub api_node_url: String,
    pub auth_token: String,
    pub namespace: String,
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
        namespace: &str,
    ) -> CelestiaConfig {
        CelestiaConfig {
            api_node_url: api_node_url.to_string(),
            auth_token: pk.to_string(),
            namespace: namespace.to_string(),
        }
    }

    #[test]
    fn from_env_celestia_client() {
        let mut lock = MUTEX.lock();
        let config = r#"
            CELESTIA_CLIENT_API_NODE_URL="localhost:12345"
            CELESTIA_CLIENT_AUTH_TOKEN="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJBbGxvdyI6WyJwdWJsaWMiLCJyZWFkIiwid3JpdGUiLCJhZG1pbiJdfQ.gQoHC03aTDFciDmOtHpe2IBtYu0qavOUlOgZd3J5POI"
            CELESTIA_CLIENT_NAMESPACE="0x1234567890abcdef"
        "#;
        lock.set_env(config);
        let actual = CelestiaConfig::from_env().unwrap();
        assert_eq!(
            actual,
            expected_celestia_da_layer_config(
                "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJBbGxvdyI6WyJwdWJsaWMiLCJyZWFkIiwid3JpdGUiLCJhZG1pbiJdfQ.gQoHC03aTDFciDmOtHpe2IBtYu0qavOUlOgZd3J5POI",
                "localhost:12345",
                "0x1234567890abcdef"
            )
        );
    }
}
