use alloy::{sol, sol_types::SolValue};
use serde::Deserialize;
use zksync_env_config::FromEnv;

use std::{fmt::Debug, ops::Deref, sync::Arc};

use async_trait::async_trait;
use near_da_primitives::{Blob, Mode};
use near_da_rpc::{
    near::{
        config::{Config, KeyType, Network},
        Client,
    },
    CryptoHash, DataAvailability,
};
use near_jsonrpc_client::methods::light_client_proof::RpcLightClientExecutionProofResponse;
use zksync_da_client::{types, DataAvailabilityClient};

use crate::evm_types::BlobInclusionProof;
use da_config::near::NearConfig;

#[derive(Clone)]
pub struct NearClient {
    pub config: NearConfig,
    pub da_rpc_client: Arc<Client>,
}

impl Debug for NearClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NearClient")
            .field("config", &self.da_rpc_client.config)
            .field("client", &self.da_rpc_client.client)
            .field("archive", &self.da_rpc_client.client)
            .finish()
    }
}

impl NearClient {
    pub async fn new() -> anyhow::Result<Self> {
        let config = NearConfig::from_env().unwrap();

        let client_config = Config {
            key: KeyType::SecretKey(config.account_id.clone(), config.secret_key.clone()),
            network: Network::try_from(config.network.as_str()).unwrap(),
            contract: config.contract.clone(),
            mode: Mode::default(),
            namespace: None,
        };

        let da_rpc_client = Client::new(&client_config);

        Ok(Self {
            config,
            da_rpc_client: da_rpc_client.into(),
        })
    }
}

#[derive(Deserialize)]
struct ProofResponse {
    head_block_root: CryptoHash,
    proof: RpcLightClientExecutionProofResponse,
}

#[async_trait]
impl DataAvailabilityClient for NearClient {
    async fn dispatch_blob(
        &self,
        _batch_number: u32,
        data: Vec<u8>,
    ) -> Result<types::DispatchResponse, types::DAError> {
        let result = self.da_rpc_client.submit(Blob::new(data)).await.unwrap();

        Ok(types::DispatchResponse {
            blob_id: bs58::encode(result.0.deref()).into_string(),
        })
    }

    async fn get_inclusion_data(
        &self,
        blob_id: &str,
    ) -> Result<Option<types::InclusionData>, types::DAError> {
        // Obtain the inclusion data for the blob_id from the near light client using the POST /proof(blob_id) endpoint with reqwest

        let url = format!("{}/proof", self.config.light_client_url);

        let api_client = reqwest::Client::new();

        let response = api_client
            .post(&url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(
                serde_json::to_string(&serde_json::json!({
                    "type": "transaction",
                    "transaction_hash": blob_id,
                    "sender_id": &self.config.account_id,
                }))
                .unwrap(),
            )
            .send()
            .await
            .unwrap();

        let proof_response: ProofResponse = response.json().await.unwrap();

        let attestation_data: BlobInclusionProof = proof_response.proof.try_into().unwrap();

        Ok(Some(types::InclusionData {
            data: attestation_data.abi_encode(),
        }))
    }

    fn clone_boxed(&self) -> Box<dyn DataAvailabilityClient> {
        Box::new(self.clone())
    }

    fn blob_size_limit(&self) -> std::option::Option<usize> {
        Some(1973786)
    }
}
