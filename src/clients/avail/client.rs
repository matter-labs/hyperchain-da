use crate::clients::avail::config::AvailConfig;
use alloy::{
    primitives::{B256, U256},
    sol,
    sol_types::SolValue,
};
use async_trait::async_trait;
use avail_core::AppId;
use avail_subxt::{
    api::{self},
    AvailClient as AvailSubxtClient,
};
use serde::Deserialize;
use std::fmt::{Debug, Formatter};
use subxt_signer::{bip39::Mnemonic, sr25519::Keypair};
use zksync_da_client::{
    types::{self, DAError, InclusionData},
    DataAvailabilityClient,
};
use zksync_env_config::FromEnv;

use anyhow::{anyhow, Result};
use avail_subxt::{
    api::{
        data_availability::calls::types::SubmitData,
        runtime_types::bounded_collections::bounded_vec::BoundedVec,
    },
    tx,
};
use tracing::error;

#[derive(Clone)]
pub struct AvailClient {
    api_node_url: String,
    bridge_api_url: String,
    seed: String,
    app_id: usize,
    timeout: usize,
    max_retries: usize,
}

#[derive(Deserialize)]
pub struct BridgeAPIResponse {
    blob_root: B256,
    bridge_root: B256,
    data_root_index: U256,
    data_root_proof: Vec<B256>,
    leaf: B256,
    leaf_index: U256,
    leaf_proof: Vec<B256>,
    range_hash: B256,
}

sol! {
    struct MerkleProofInput {
        // proof of inclusion for the data root
        bytes32[] dataRootProof;
        // proof of inclusion of leaf within blob/bridge root
        bytes32[] leafProof;
        // abi.encodePacked(startBlock, endBlock) of header range commitment on vectorx
        bytes32 rangeHash;
        // index of the data root in the commitment tree
        uint256 dataRootIndex;
        // blob root to check proof against, or reconstruct the data root
        bytes32 blobRoot;
        // bridge root to check proof against, or reconstruct the data root
        bytes32 bridgeRoot;
        // leaf being proven
        bytes32 leaf;
        // index of the leaf in the blob/bridge root tree
        uint256 leafIndex;
    }
}

impl AvailClient {
    pub fn new() -> anyhow::Result<Self> {
        let config = AvailConfig::from_env()?;

        Ok(Self {
            api_node_url: config.api_node_url,
            bridge_api_url: config.bridge_api_url,
            seed: config.seed,
            app_id: config.app_id,
            timeout: config.timeout,
            max_retries: config.max_retries,
        })
    }
}

#[async_trait]
impl DataAvailabilityClient for AvailClient {
    async fn dispatch_blob(
        &self,
        _batch_number: u32,
        data: Vec<u8>,
    ) -> Result<types::DispatchResponse, types::DAError> {
        let client = AvailSubxtClient::new(self.api_node_url.clone())
            .await
            .map_err(|e| anyhow!("Client cannot be connected: {e:?}"))
            .unwrap();

        let mnemonic = Mnemonic::parse(&self.seed).unwrap();
        let keypair = Keypair::from_phrase(&mnemonic, None).unwrap();
        let call = api::tx()
            .data_availability()
            .submit_data(BoundedVec(data.clone()));

        let nonce = avail_subxt::tx::nonce(&client, &keypair).await.unwrap();
        let tx_progress = tx::send_with_nonce(
            &client,
            &call,
            &keypair,
            AppId(u32::try_from(self.app_id).unwrap()),
            nonce,
        )
        .await
        .unwrap();
        let block_hash = tx::then_in_block(tx_progress).await.unwrap().block_hash();

        // Retrieve the data from the block hash
        let block = client.blocks().at(block_hash).await.unwrap();
        let extrinsics = block.extrinsics().await.unwrap();
        let mut found = false;
        let mut tx_idx = 0;
        for ext in extrinsics.iter() {
            let ext = ext.unwrap();
            let call = ext.as_extrinsic::<SubmitData>();
            if let Ok(Some(call)) = call {
                if data.clone() == call.data.0 {
                    found = true;
                }
            }
            tx_idx += 1;
        }

        if !found {
            error!("No DA submission found in block: {}", block_hash);
            return Err(DAError {
                error: anyhow!("No DA submission found in block: {}", block_hash),
                is_transient: false,
            });
        }

        Ok(types::DispatchResponse {
            blob_id: format!("{}:{}", block_hash, tx_idx),
        })
    }

    async fn get_inclusion_data(
        &self,
        blob_id: &str,
    ) -> Result<Option<types::InclusionData>, types::DAError> {
        let (block_hash, tx_idx) = blob_id.split_once(':').unwrap();
        let client = reqwest::Client::new();
        let url = format!(
            "{}/eth/proof/{}?index={}",
            self.bridge_api_url, block_hash, tx_idx
        );
        let response = client.get(&url).send().await.unwrap();
        let body = response.text().await.unwrap();
        let bridge_api_data: BridgeAPIResponse = serde_json::from_str(&body).unwrap();
        let attestation_data: MerkleProofInput = MerkleProofInput {
            dataRootProof: bridge_api_data.data_root_proof,
            leafProof: bridge_api_data.leaf_proof,
            rangeHash: bridge_api_data.range_hash,
            dataRootIndex: bridge_api_data.data_root_index,
            blobRoot: bridge_api_data.blob_root,
            bridgeRoot: bridge_api_data.bridge_root,
            leaf: bridge_api_data.leaf,
            leafIndex: bridge_api_data.leaf_index,
        };
        Ok(Some(InclusionData {
            data: attestation_data.abi_encode(),
        }))
    }

    fn clone_boxed(&self) -> Box<dyn DataAvailabilityClient> {
        Box::new(self.clone())
    }

    fn blob_size_limit(&self) -> Option<usize> {
        Some(usize::try_from(512 * 1024).unwrap()) // 512 KB is the limit per blob
    }
}

impl Debug for AvailClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AvailClient")
            .field("api_node_url", &self.api_node_url)
            .field("bridge_api_url", &self.bridge_api_url)
            .field("app_id", &self.app_id)
            .field("timeout", &self.timeout)
            .field("max_retries", &self.max_retries)
            .finish()
    }
}
