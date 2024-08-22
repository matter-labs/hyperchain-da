use da_config::avail::AvailConfig;
use alloy::{
    primitives::{B256, U256},
    sol,
};
use async_trait::async_trait;
use avail_core::AppId;
use avail_subxt::{
    api::{self},
    AvailClient as AvailSubxtClient,
};
use serde::Deserialize;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use subxt_signer::{bip39::Mnemonic, sr25519::Keypair};
use zksync_da_client::{
    types::{self, DAError, DispatchResponse, InclusionData},
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

#[derive(Clone)]
pub struct AvailClient {
    config: AvailConfig,
    client: Arc<AvailSubxtClient>,
    api_client: Arc<reqwest::Client>,
    keypair: Keypair,
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
    const MAX_BLOB_SIZE: usize = 512 * 1024; // 512 kibibytes

    pub async fn new() -> anyhow::Result<Self> {
        let config = match da_utils::proto_config_parser::try_parse_proto_config::<proto_config::proto::avail::Avail>()? {
            Some(config) => config,
            None => AvailConfig::from_env()?,
        };

        let client = AvailSubxtClient::new(config.api_node_url.clone())
            .await
            .map_err(to_non_retriable_da_error)?;

        let mnemonic = Mnemonic::parse(&config.seed).map_err(to_non_retriable_da_error)?;

        let keypair = Keypair::from_phrase(&mnemonic, None).map_err(to_non_retriable_da_error)?;

        let api_client = reqwest::Client::new();

        Ok(Self {
            config,
            client: client.into(),
            api_client: api_client.into(),
            keypair,
        })
    }
}

pub fn to_non_retriable_da_error(error: impl Into<anyhow::Error>) -> types::DAError {
    DAError {
        error: error.into(),
        is_retriable: false,
    }
}

#[async_trait]
impl DataAvailabilityClient for AvailClient {
    async fn dispatch_blob(
        &self,
        _batch_number: u32,
        data: Vec<u8>,
    ) -> Result<DispatchResponse, DAError> {
        let call = api::tx()
            .data_availability()
            .submit_data(BoundedVec(data.clone()));
        let tx_progress = tx::send(
            &self.client,
            &call,
            &self.keypair,
            AppId(self.config.app_id),
        )
            .await
            .map_err(to_non_retriable_da_error)?;
        let block_hash = tx::then_in_block(tx_progress)
            .await
            .map_err(to_non_retriable_da_error)?
            .block_hash();

        // Retrieve the data from the block hash
        let block = self
            .client
            .blocks()
            .at(block_hash)
            .await
            .map_err(to_non_retriable_da_error)?;
        let extrinsics = block
            .extrinsics()
            .await
            .map_err(to_non_retriable_da_error)?;
        let mut found = false;
        let mut tx_idx = 0;
        for ext in extrinsics.iter() {
            let ext = ext.map_err(to_non_retriable_da_error)?;
            let call = ext.as_extrinsic::<SubmitData>();
            if let Ok(Some(call)) = call {
                if data.clone() == call.data.0 {
                    found = true;
                    break;
                }
            }
            tx_idx += 1;
        }

        if !found {
            return Err(to_non_retriable_da_error(anyhow!(
                "No DA submission found in block: {}",
                block_hash
            )));
        }

        Ok(DispatchResponse {
            blob_id: format!("{}:{}", block_hash, tx_idx),
        })
    }

    async fn get_inclusion_data(
        &self,
        _blob_id: &str,
    ) -> Result<Option<types::InclusionData>, types::DAError> {
        // let (block_hash, tx_idx) = blob_id.split_once(':').ok_or_else(|| DAError {
        //     error: anyhow!("Invalid blob ID format"),
        //     is_retriable: false,
        // })?;
        // let url = format!(
        //     "{}/eth/proof/{}?index={}",
        //     self.config.bridge_api_url, block_hash, tx_idx
        // );
        // let mut response: Response;
        // let mut retries = 0usize;
        // loop {
        //     response = self
        //         .api_client
        //         .get(&url)
        //         .send()
        //         .await
        //         .map_err(|e| self.to_non_retriable_da_error(e))?;
        //     if response.status().is_success() {
        //         break;
        //     }
        //     sleep(Duration::from_secs(
        //         u64::try_from(self.config.timeout).unwrap(),
        //     ))
        //     .await;
        //     retries += 1;
        //     if retries > self.config.max_retries {
        //         return Err(DAError {
        //             error: anyhow!("Failed to get inclusion data"),
        //             is_retriable: true,
        //         });
        //     }
        // }
        // let bridge_api_data: BridgeAPIResponse = response
        //     .json()
        //     .await
        //     .map_err(|e| self.to_non_retriable_da_error(e))?;
        // let attestation_data: MerkleProofInput = MerkleProofInput {
        //     dataRootProof: bridge_api_data.data_root_proof,
        //     leafProof: bridge_api_data.leaf_proof,
        //     rangeHash: bridge_api_data.range_hash,
        //     dataRootIndex: bridge_api_data.data_root_index,
        //     blobRoot: bridge_api_data.blob_root,
        //     bridgeRoot: bridge_api_data.bridge_root,
        //     leaf: bridge_api_data.leaf,
        //     leafIndex: bridge_api_data.leaf_index,
        // };
        // Ok(Some(InclusionData {
        //     data: attestation_data.abi_encode(),
        // }))
        Ok(Some(InclusionData { data: vec![] }))
    }

    fn clone_boxed(&self) -> Box<dyn DataAvailabilityClient> {
        Box::new(self.clone())
    }

    fn blob_size_limit(&self) -> Option<usize> {
        Some(AvailClient::MAX_BLOB_SIZE)
    }
}

impl Debug for AvailClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AvailClient")
            .field("api_node_url", &self.config.api_node_url)
            .field("bridge_api_url", &self.config.bridge_api_url)
            .field("app_id", &self.config.app_id)
            .field("timeout", &self.config.timeout)
            .field("max_retries", &self.config.max_retries)
            .finish()
    }
}
