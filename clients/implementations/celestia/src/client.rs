use async_trait::async_trait;
use da_config::celestia::CelestiaConfig;
use std::fmt::{Debug, Formatter};

use zksync_da_client::{types, DataAvailabilityClient};
use zksync_env_config::FromEnv;

#[derive(Clone)]
pub struct CelestiaClient {
    light_node_url: String,
    private_key: String,
}

impl CelestiaClient {
    pub fn new() -> anyhow::Result<Self> {
        // TODO: read proto config first
        let config = CelestiaConfig::from_env()?;

        Ok(Self {
            light_node_url: config.api_node_url,
            private_key: config.private_key,
        })
    }
}

#[async_trait]
impl DataAvailabilityClient for CelestiaClient {
    async fn dispatch_blob(
        &self,
        _: u32,     // batch_number
        _: Vec<u8>, // data
    ) -> Result<types::DispatchResponse, types::DAError> {
        todo!()
    }
    async fn get_inclusion_data(
        &self,
        _blob_id: &str,
    ) -> anyhow::Result<Option<types::InclusionData>, types::DAError> {
        todo!()
    }

    fn clone_boxed(&self) -> Box<dyn DataAvailabilityClient> {
        Box::new(self.clone())
    }

    fn blob_size_limit(&self) -> Option<usize> {
        Some(1973786)
    }
}

impl Debug for CelestiaClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CelestiaClient")
            .field("light_node_url", &self.light_node_url)
            .finish()
    }
}
