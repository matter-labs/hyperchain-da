use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use async_trait::async_trait;
use celestia_types::nmt::NamespacedHashExt;
use crate::clients::celestia::config::CelestiaConfig;

use zksync_env_config::FromEnv;
use zksync_da_client::{DataAvailabilityClient, types};

use serde::{Serialize, Deserialize};
use anyhow::anyhow;

use celestia_rpc::{BlobClient, HeaderClient, Client};
use celestia_types::{Blob, nmt::{Namespace, NamespaceProof, NamespacedHash}, blob::Commitment, hash::Hash, TxConfig};
use nmt_rs::{
    TmSha2Hasher,
    simple_merkle::{tree::MerkleTree, db::MemDb, proof::Proof},
};

#[derive(Clone)]
pub struct CelestiaClient {
    light_node_url: String,
    namespace: String,
    auth_token: String,
    client: Arc<Client>,
}

impl CelestiaClient {
    pub async fn new() -> anyhow::Result<Self> {
        let config = CelestiaConfig::from_env()?;

        let client = Client::new(&config.api_node_url, Some(&config.auth_token))
            .await
            .expect("could not create client");

        Ok(Self {
            light_node_url: config.api_node_url,
            auth_token: config.auth_token,
            client: Arc::new(client),
            namespace: config.namespace,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct BlobId {
    pub commitment: Commitment,
    pub height: u64,
}

#[async_trait]
impl DataAvailabilityClient for CelestiaClient {
    async fn dispatch_blob(
        &self,
        batch_number: u32,
        data: Vec<u8>,
    ) -> Result<types::DispatchResponse, types::DAError> {
        let namespace_bytes = self.namespace.as_bytes();
        let namespace = Namespace::new_v0(namespace_bytes)
            .map_err(|e| types::DAError { error: e.into(), is_transient: false })?;
        let blob = Blob::new(namespace, data)
            .map_err(|e| types::DAError { error: e.into(), is_transient: false })?;
        let commitment = blob.commitment.clone();
        let height = self.client.blob_submit(&[blob], TxConfig{
            signer_address: None,
            key_name: None,
            gas_price: None,
            gas: None,
            fee_granter_address: None,
        })
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

    async fn get_inclusion_data(&self, blob_id: &str) -> Result<Option<types::InclusionData>, types::DAError> {        // How do we want to do namespaces?

        // Does nothing for now
        Ok(Some(types::InclusionData {
            data: vec![],
        }))

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
