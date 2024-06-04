use std::fmt::{Debug, Formatter};
use async_trait::async_trait;
use crate::clients::celestia::config::CelestiaConfig;

use zksync_env_config::FromEnv;
use zksync_da_client::{DataAvailabilityClient, types};

#[derive(Clone)]
pub struct CelestiaClient {
    light_node_url: String,
    private_key: String,
}

impl CelestiaClient {
    pub fn new() -> anyhow::Result<Self> {
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
        batch_number: u32,
        data: Vec<u8>,
    ) -> Result<types::DispatchResponse, types::DAError> {
        todo!()
    }

    async fn get_inclusion_data(&self, blob_id: String) -> Result<Option<types::InclusionData>, types::DAError> {
        todo!()
    }

    fn clone_boxed(&self) -> Box<dyn DataAvailabilityClient> {
        Box::new(self.clone())
    }

    fn blob_size_limit(&self) -> usize {
        1973786
    }
}

impl Debug for CelestiaClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CelestiaClient")
            .field("light_node_url", &self.light_node_url)
            .finish()
    }
}
