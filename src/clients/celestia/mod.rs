pub mod config;

use core::fmt;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use async_trait::async_trait;
use celestia_types::nmt::{NamespaceProof, NamespacedHashExt};
use serde::{Serialize, Deserialize};
use crate::{DataAvailabilityClient, types};
use crate::clients::celestia::config::CelestiaConfig;
use celestia_rpc::{BlobClient, HeaderClient, Client};
use celestia_types::{Blob, nmt::Namespace, blob::Commitment, hash::Hash};
use bincode;
use hex;
use anyhow::anyhow;
use nmt_rs::{
    NamespacedHash, TmSha2Hasher,
    simple_merkle::{tree::MerkleTree, db::MemDb, proof::Proof},
};

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

pub struct InclusionDataPayload {
    pub row_inclusion_range_proof: Proof<TmSha2Hasher>,
    pub share_to_row_root_proofs: Vec<NamespaceProof>,
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
        // (namespace doesn't really matter for L2 rollups such as zkStack)
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
        // How do we want to do namespaces?
        let my_namespace = Namespace::new_v0(&[1, 2, 3, 4, 5]).expect("Invalid namespace");
        let blob_id: BlobId = bincode::deserialize(&hex::decode(blob_id).unwrap())
            .map_err(|e| types::DAError { error: e.into(), is_transient: false })?;
        let blob = self.client.blob_get(blob_id.height, my_namespace, blob_id.commitment)
            .await
            .map_err(|e| types::DAError { error: e.into(), is_transient: false })?;
        let shares_to_row_roots_proofs = self.client.blob_get_proof(blob_id.height, my_namespace, blob_id.commitment)
            .await
            .map_err(|e| types::DAError { error: e.into(), is_transient: false })?;
        let header = self.client.header_get_by_height(blob_id.height)
            .await
            .map_err(|e| types::DAError { error: e.into(), is_transient: false })?;

        // Get the EDS row and column roots
        let eds_row_roots = header.dah.row_roots();
        let eds_col_roots = header.dah.column_roots();
        let data_tree_leaves: Vec<_> = eds_row_roots.iter()
            .chain(eds_col_roots.iter())
            .map(|root| root.to_array())
            .collect();

        // Create a merkle tree of the row and column roots
        let hasher = TmSha2Hasher{};
        let mut tree: MerkleTree<MemDb<[u8; 32]>, TmSha2Hasher> = MerkleTree::with_hasher(hasher);
        for leaf in data_tree_leaves {
            tree.push_raw_leaf(&leaf);
        }
        assert_eq!(header.dah.hash(), Hash::Sha256(tree.root()));

        // extended data square (EDS) size
        let eds_size = eds_row_roots.len();
        // original data square (ODS) size
        let ods_size = eds_size/2;

        let blob_index = blob.index
            .ok_or(types::DAError { error: anyhow!("Blob index not found"), is_transient: false })?;
        let blob_size = blob.data.len();
        let first_row_index: usize = blob_index as usize / ods_size;
        let last_row_index: usize = first_row_index + (blob_size / ods_size);
        let range_proof = tree.build_range_proof(first_row_index..last_row_index+1);
        let inclusion_data_payload = InclusionDataPayload {
            row_inclusion_range_proof: range_proof,
            share_to_row_root_proofs: shares_to_row_roots_proofs,
        };
        todo!() // Need to go and implement SSZ serialization for this
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
