pub mod config;

use anyhow;
use std::fmt::{Debug, Formatter};
use async_trait::async_trait;
use crate::{DataAvailabilityClient, types};
use crate::clients::celestia::config::CelestiaConfig;

pub struct CelestiaClient {
    light_node_url: String,
    private_key: String,
}

impl CelestiaClient {
    pub fn new(config: CelestiaConfig) -> Self {
        Self {
            light_node_url: config.light_node_url,
            private_key: config.private_key,
        }
    }
}

#[async_trait]
impl DataAvailabilityClient for CelestiaClient {
    async fn dispatch_blob(
        &self,
        batch_number: u32,
        data: Vec<u8>,
    ) -> Result<types::DispatchResponse, anyhow::Error> {
        todo!()
    }

    async fn get_inclusion_data(&self, blob_id: Vec<u8>) -> Result<Option<types::InclusionData>, anyhow::Error> {
        todo!()
    }
}

impl Debug for CelestiaClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CelestiaClient")
            .field("light_node_url", &self.light_node_url)
            .finish()
    }
}
