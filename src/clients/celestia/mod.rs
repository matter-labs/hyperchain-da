pub mod config;

use core::fmt;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use crate::{DataAvailabilityClient, types};
use crate::clients::celestia::config::CelestiaConfig;
use celestia_rpc::{BlobClient, HeaderClient, Client};
use celestia_types::{Blob, nmt::Namespace, blob::Commitment};
use bincode;
use hex;

#[derive(Clone)]
pub struct CelestiaClient {
    light_node_url: String,
    client: Arc<Client>,
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
            client: Arc::new(client),
        }
    }
}

#[async_trait]
impl DataAvailabilityClient for CelestiaClient {
    async fn dispatch_blob(
        &self,
        batch_number: u32, /* what's the purpose of batch_number? */
        data: Vec<u8>,
    ) -> Result<types::DispatchResponse, types::DAError> {
        // Note: how does zkStack want to determine namespace?
        let my_namespace = Namespace::new_v0(&[1, 2, 3, 4, 5]).expect("Invalid namespace");
        let blob = Blob::new(my_namespace, data)
            .map_err(|e| types::DAError { error: e.into(), is_transient: false })?;
        let commitment = blob.commitment.clone();
        let height = self.client.blob_submit(&[blob], None.into())
            .await
            .map_err(|e| types::DAError { error: e.into(), is_transient: false })?;
        let blob_id = BlobId {
            commitment: commitment,
            height: height,
        };
        let blob_bytes = bincode::serialize(&blob_id)
            .map_err(|e| types::DAError { error: e.into(), is_transient: false })?;
        let blob_hex_string = hex::encode(&blob_bytes);
        Ok(types::DispatchResponse {
            blob_id: blob_hex_string,
        })
    }

    async fn get_inclusion_data(&self, blob_id: String) -> Result<Option<types::InclusionData>, types::DAError> {
        let my_namespace = Namespace::new_v0(&[1, 2, 3, 4, 5]).expect("Invalid namespace");
        let blob_id: BlobId = bincode::deserialize(&hex::decode(blob_id).unwrap())
            .map_err(|e| types::DAError { error: e.into(), is_transient: false })?;
        let shares_to_row_roots_proofs = self.client.blob_get_proof(blob_id.height, my_namespace, blob_id.commitment)
            .await
            .map_err(|e| types::DAError { error: e.into(), is_transient: false })?;
        let header = self.client.header_get_by_height(blob_id.height)
            .await
            .map_err(|e| types::DAError { error: e.into(), is_transient: false })?;
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
