pub mod config;

use core::fmt;
use std::fmt::{Debug, Formatter};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use crate::{DataAvailabilityClient, types};
use crate::clients::celestia::config::CelestiaConfig;
use celestia_rpc::{BlobClient, HeaderClient, Client};
use celestia_types::{Blob, nmt::Namespace, blob::Commitment};
// Why did you need clone?
pub struct CelestiaClient {
    light_node_url: String,
    client: Client,
}

#[derive(Serialize, Deserialize)]
pub struct BlobId {
    pub commitment: Commitment,
    pub height: u64,
}

impl CelestiaClient {
    pub async fn new(config: CelestiaConfig) -> Self {
        let client = Client::new(&config.light_node_url, Some(&config.auth_token))
            .await
            .expect("could not create client");
        Self {
            light_node_url: config.light_node_url,
            client: client,
        }
    }
}

#[async_trait]
impl DataAvailabilityClient for CelestiaClient {
    async fn dispatch_blob(
        &self,
        batch_number: u32,
        data: Vec<u8>,
    ) -> Result<types::DispatchResponse, types::DAError> {
        // Note: how does zkStack want to determine namespace?
        let my_namespace = Namespace::new_v0(&[1, 2, 3, 4, 5]).expect("Invalid namespace");
        let blob = Blob::new(my_namespace, data)
            .map_err(|e| types::DAError { error: e.into(), is_transient: false })?;
        let height = self.client.blob_submit(&[blob], None.into())
            .await
            .map_err(|e| types::DAError { error: e.into(), is_transient: false })?;
        let blob_id = BlobId {
            commitment: blob.commitment,
            height: height,
        };
        Ok(types::DispatchResponse {
            blob_id: blob_id_serialized,
        })
    }

    async fn get_inclusion_data(&self, blob_id: String) -> Result<Option<types::InclusionData>, types::DAError> {
        todo!()
    }

    /*fn clone_boxed(&self) -> Box<dyn DataAvailabilityClient> {
        Box::new(self.clone())
    }*/

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
