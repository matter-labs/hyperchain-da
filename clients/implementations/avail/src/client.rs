use alloy::sol_types::SolValue;
use alloy::{
    primitives::{B256, U256},
    sol,
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use avail_core::AppId;
use avail_subxt::{
    api::{self},
    AvailClient as AvailSubxtClient,
};
use avail_subxt::{
    api::{
        data_availability::calls::types::SubmitData,
        runtime_types::bounded_collections::bounded_vec::BoundedVec,
    },
    tx,
};
use bytes::Bytes;
use da_config::avail::AvailConfig;
use da_utils::proto_config_parser::try_parse_proto_config;
use serde::Deserialize;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use subxt_signer::{bip39::Mnemonic, sr25519::Keypair};
use zksync_da_client::{
    types::{self, DAError, DispatchResponse, InclusionData},
    DataAvailabilityClient,
};
use zksync_env_config::FromEnv;

#[derive(Clone)]
pub struct AvailClient {
    config: AvailConfig,
    api_client: Arc<reqwest::Client>,
    keypair: Option<Keypair>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BridgeAPIResponse {
    blob_root: Option<B256>,
    bridge_root: Option<B256>,
    data_root_index: Option<U256>,
    data_root_proof: Option<Vec<B256>>,
    leaf: Option<B256>,
    leaf_index: Option<U256>,
    leaf_proof: Option<Vec<B256>>,
    range_hash: Option<B256>,
    error: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct GasRelayAPISubmissionResponse {
    submission_id: String,
}

#[derive(Deserialize, Debug)]
pub struct GasRelayAPIStatusResponse {
    submission: GasRelayAPISubmission,
}

#[derive(Deserialize, Debug)]
pub struct GasRelayAPISubmission {
    block_hash: Option<B256>,
    extrinsic_index: Option<u64>,
}

sol! {
    #[derive(Debug)]
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
        let config = match try_parse_proto_config::<proto_config::proto::avail::AvailConfig>()? {
            Some(config) => config,
            None => AvailConfig::from_env()?,
        };

        if config.gas_relay_mode {
            return Ok(Self {
                config,
                api_client: reqwest::Client::new().into(),
                keypair: None,
            });
        }

        let mnemonic =
            Mnemonic::parse(config.seed.clone().unwrap()).map_err(to_non_retriable_da_error)?;

        let keypair = Keypair::from_phrase(&mnemonic, None).map_err(to_non_retriable_da_error)?;

        let api_client = reqwest::Client::new();

        Ok(Self {
            config,
            api_client: api_client.into(),
            keypair: keypair.into(),
        })
    }
}

pub fn to_non_retriable_da_error(error: impl Into<anyhow::Error>) -> types::DAError {
    DAError {
        error: error.into(),
        is_retriable: false,
    }
}

pub fn to_retriable_da_error(error: impl Into<anyhow::Error>) -> types::DAError {
    DAError {
        error: error.into(),
        is_retriable: true,
    }
}

#[async_trait]
impl DataAvailabilityClient for AvailClient {
    async fn dispatch_blob(
        &self,
        _batch_number: u32,
        data: Vec<u8>,
    ) -> Result<DispatchResponse, DAError> {
        if self.config.gas_relay_mode {
            let submit_url = format!(
                "{}/user/submit_raw_data?token=ethereum",
                self.config.gas_relay_api_url.clone().unwrap()
            );
            // send the data to the gas relay
            let submit_response = self
                .api_client
                .post(&submit_url)
                .body(Bytes::from(data))
                .header("Content-Type", "text/plain")
                .header(
                    "Authorization",
                    self.config.gas_relay_api_key.clone().unwrap(),
                )
                .send()
                .await
                .map_err(to_retriable_da_error)?;
            let submit_response_text = submit_response
                .text()
                .await
                .map_err(to_retriable_da_error)?;
            let submit_response_struct: GasRelayAPISubmissionResponse =
                serde_json::from_str(&submit_response_text.clone())
                    .map_err(to_retriable_da_error)?;
            let status_url = format!(
                "{}/user/get_submission_info?submission_id={}",
                self.config.gas_relay_api_url.clone().unwrap(),
                submit_response_struct.submission_id
            );
            let mut retries = 0;
            let mut status_response: reqwest::Response;
            let mut status_response_text: String;
            let mut status_response_struct: GasRelayAPIStatusResponse;
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(u64::try_from(40).unwrap()))
                    .await; // usually takes 20s to finalize
                status_response = self
                    .api_client
                    .get(&status_url)
                    .header(
                        "Authorization",
                        self.config.gas_relay_api_key.clone().unwrap(),
                    )
                    .send()
                    .await
                    .map_err(to_retriable_da_error)?;
                status_response_text = status_response
                    .text()
                    .await
                    .map_err(to_retriable_da_error)?;
                status_response_struct =
                    serde_json::from_str(&status_response_text).map_err(to_retriable_da_error)?;
                if status_response_struct.submission.block_hash.is_some() {
                    break;
                }
                retries += 1;
                if retries > self.config.max_retries {
                    return Err(to_retriable_da_error(anyhow!(
                        "Failed to get gas relay status"
                    )));
                }
            }
            return Ok(DispatchResponse {
                blob_id: format!(
                    "{:x}:{}",
                    status_response_struct.submission.block_hash.unwrap(),
                    status_response_struct.submission.extrinsic_index.unwrap()
                ),
            });
        }
        let client = AvailSubxtClient::new(self.config.api_node_url.clone().unwrap())
            .await
            .map_err(to_retriable_da_error)?;
        let call = api::tx()
            .data_availability()
            .submit_data(BoundedVec(data.clone()));
        let tx_progress = tx::send(
            &client,
            &call,
            &self.keypair.clone().unwrap(),
            AppId(self.config.app_id.unwrap()),
        )
        .await
        .map_err(to_retriable_da_error)?;
        let block_hash = tx::then_in_finalized_block(tx_progress)
            .await
            .map_err(to_retriable_da_error)?
            .block_hash();

        // Retrieve the data from the block hash
        let block = client
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
            blob_id: format!("{:x}:{}", block_hash, tx_idx),
        })
    }

    async fn get_inclusion_data(
        &self,
        blob_id: &str,
    ) -> Result<Option<types::InclusionData>, types::DAError> {
        let (block_hash, tx_idx) = blob_id.split_once(':').ok_or_else(|| DAError {
            error: anyhow!("Invalid blob ID format"),
            is_retriable: false,
        })?;

        let url = format!(
            "{}/eth/proof/{}?index={}",
            self.config.bridge_api_url, block_hash, tx_idx
        );
        let mut response: reqwest::Response;
        let mut retries = self.config.max_retries;
        let mut response_text: String;
        let mut bridge_api_data: BridgeAPIResponse;
        loop {
            response = self
                .api_client
                .get(&url)
                .send()
                .await
                .map_err(to_retriable_da_error)?;
            response_text = response.text().await.unwrap();

            if let Ok(data) = serde_json::from_str::<BridgeAPIResponse>(&response_text) {
                bridge_api_data = data;
                if bridge_api_data.error.is_none() {
                    break;
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(
                u64::try_from(480).unwrap(),
            ))
            .await; // usually takes 15 mins on Hex
            retries += 1;
            if retries > self.config.max_retries {
                return Err(DAError {
                    error: anyhow!("Failed to get inclusion data"),
                    is_retriable: true,
                });
            }
        }
        let attestation_data: MerkleProofInput = MerkleProofInput {
            dataRootProof: bridge_api_data.data_root_proof.unwrap(),
            leafProof: bridge_api_data.leaf_proof.unwrap(),
            rangeHash: bridge_api_data.range_hash.unwrap(),
            dataRootIndex: bridge_api_data.data_root_index.unwrap(),
            blobRoot: bridge_api_data.blob_root.unwrap(),
            bridgeRoot: bridge_api_data.bridge_root.unwrap(),
            leaf: bridge_api_data.leaf.unwrap(),
            leafIndex: bridge_api_data.leaf_index.unwrap(),
        };
        Ok(Some(InclusionData {
            data: attestation_data.abi_encode(),
        }))
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
