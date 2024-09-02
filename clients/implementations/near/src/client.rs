use alloy::{
    primitives::Address,
    providers::{network::Ethereum, RootProvider},
    transports::http::Http,
};
use anyhow::anyhow;
use async_trait::async_trait;
use borsh::to_vec;
use std::{fmt::Debug, ops::Deref, sync::Arc};

use near_da_primitives::{Blob, Mode};
use near_da_rpc::{
    near::{
        config::{Config, KeyType, Network},
        Client,
    },
    DataAvailability,
};
use near_jsonrpc_client::{
    methods::{
        block::RpcBlockRequest,
        light_client_proof::{
            RpcLightClientExecutionProofRequest, RpcLightClientExecutionProofResponse,
        },
    },
    JsonRpcClient,
};
use near_primitives::{
    block_header::BlockHeader,
    hash::CryptoHash,
    merkle::compute_root_from_path,
    types::{AccountId, TransactionOrReceiptId},
    views::LightClientBlockLiteView,
};

use zksync_da_client::{
    types::{self, DAError},
    DataAvailabilityClient,
};
use zksync_env_config::FromEnv;

use crate::types::{BlobInclusionProof, NearX::NearXInstance};

use da_config::near::NearConfig;
use da_utils::{errors::to_non_retriable_da_error, proto_config_parser::try_parse_proto_config};

type Provider = RootProvider<Http<reqwest::Client>, Ethereum>;

#[derive(Clone)]
pub struct NearClient {
    pub config: NearConfig,
    pub da_rpc_client: Arc<Client>,
    pub light_client: Arc<JsonRpcClient>,
}

#[async_trait]
trait LightClient {
    async fn latest_header(&self) -> Result<CryptoHash, types::DAError>;
    async fn get_header(
        &self,
        latest_header: CryptoHash,
    ) -> Result<LightClientBlockLiteView, types::DAError>;
    async fn get_proof(
        &self,
        transaction_hash: &str,
        latest_header: CryptoHash,
    ) -> Result<RpcLightClientExecutionProofResponse, types::DAError>;
}

#[async_trait]
impl LightClient for NearClient {
    async fn latest_header(&self) -> Result<CryptoHash, types::DAError> {
        let url = reqwest::Url::parse(&self.config.evm_provider_url)
            .map_err(to_non_retriable_da_error)?;
        let inner = Provider::new_http(url);
        let bridge_address = self
            .config
            .bridge_contract
            .parse::<Address>()
            .map_err(to_non_retriable_da_error)?;
        let bridge: NearXInstance<Http<reqwest::Client>, Provider, Ethereum> =
            NearXInstance::new(bridge_address, inner);
        let latest_header: CryptoHash = bridge
            .latestHeader()
            .call()
            .await
            .map(|x| *x._0)
            .map(CryptoHash)
            .map_err(|e| DAError {
                error: anyhow!(e),
                is_retriable: true,
            })?;

        Ok(latest_header)
    }

    async fn get_header(
        &self,
        latest_header: CryptoHash,
    ) -> Result<LightClientBlockLiteView, types::DAError> {
        let req = RpcBlockRequest {
            block_reference: near_primitives::types::BlockReference::BlockId(
                near_primitives::types::BlockId::Hash(latest_header),
            ),
        };

        let block_light_view: LightClientBlockLiteView = self
            .light_client
            .call(req)
            .await
            .map_err(|e| DAError {
                error: anyhow!(e),
                is_retriable: true,
            })
            .map(|x| x.header)
            .map(BlockHeader::from)
            .map(Into::into)?;

        Ok(block_light_view)
    }

    async fn get_proof(
        &self,
        transaction_hash: &str,
        latest_header: CryptoHash,
    ) -> Result<RpcLightClientExecutionProofResponse, types::DAError> {
        let req = RpcLightClientExecutionProofRequest {
            id: TransactionOrReceiptId::Transaction {
                transaction_hash: transaction_hash
                    .parse::<CryptoHash>()
                    .map_err(|e| to_non_retriable_da_error(anyhow!(e)))?,
                sender_id: self
                    .config
                    .account_id
                    .parse::<AccountId>()
                    .map_err(to_non_retriable_da_error)?,
            },
            light_client_head: latest_header,
        };

        let proof = self.light_client.call(req).await.map_err(|e| DAError {
            error: anyhow!(e),
            is_retriable: true,
        })?;

        Ok(proof)
    }
}

impl Debug for NearClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NearClient")
            .field("config", &self.da_rpc_client.config)
            .field("client", &self.da_rpc_client.client)
            .field("archive", &self.da_rpc_client.archive)
            .finish()
    }
}

impl NearClient {
    pub async fn new() -> anyhow::Result<Self> {
        let config = match try_parse_proto_config::<proto_config::proto::near::NearConfig>()? {
            Some(config) => config,
            None => NearConfig::from_env()?,
        };

        let client_config = Config {
            key: KeyType::SecretKey(config.account_id.clone(), config.secret_key.clone()),
            network: Network::try_from(config.network.as_str())
                .map_err(|e| to_non_retriable_da_error(anyhow!(e)))?,
            contract: config.contract.clone(),
            mode: Mode::default(),
            namespace: None,
        };

        let da_rpc_client = Client::new(&client_config);
        let light_client = Arc::new(JsonRpcClient::connect(da_rpc_client.client.server_addr()));

        Ok(Self {
            config,
            da_rpc_client: da_rpc_client.into(),
            light_client,
        })
    }

    fn verify_proof(
        &self,
        head_block_root: CryptoHash,
        proof: &RpcLightClientExecutionProofResponse,
    ) -> Result<(), DAError> {
        let expected_outcome_root = proof.block_header_lite.inner_lite.outcome_root;

        let outcome_hash = CryptoHash::hash_borsh(proof.outcome_proof.to_hashes());
        let outcome_root = compute_root_from_path(&proof.outcome_proof.proof, outcome_hash);
        let leaf = CryptoHash::hash_borsh(outcome_root);
        let outcome_root = compute_root_from_path(&proof.outcome_root_proof, leaf);

        if expected_outcome_root != outcome_root {
            return Err(DAError {
                error: anyhow!("Calculated outcome_root does not match proof.block_header_lite.inner_lite.outcome_root"),
                is_retriable: false,
            });
        }

        // Verify proof block root matches the light client head block root
        let block_hash = proof.block_header_lite.hash();
        let block_root = compute_root_from_path(&proof.block_proof, block_hash);
        if head_block_root != block_root {
            return Err(DAError {
                error: anyhow!("Calculated block_merkle_root does not match head_block_root"),
                is_retriable: false,
            });
        }

        Ok(())
    }
}

#[async_trait]
impl DataAvailabilityClient for NearClient {
    async fn dispatch_blob(
        &self,
        _batch_number: u32,
        data: Vec<u8>,
    ) -> Result<types::DispatchResponse, types::DAError> {
        let result = self
            .da_rpc_client
            .submit(Blob::new(data))
            .await
            .map_err(|e| DAError {
                error: anyhow!(e),
                is_retriable: true,
            })?;

        Ok(types::DispatchResponse {
            blob_id: CryptoHash(*result.0.deref()).to_string(),
        })
    }

    async fn get_inclusion_data(
        &self,
        blob_id: &str,
    ) -> Result<Option<types::InclusionData>, types::DAError> {
        // Call bridge_contract `latestHeader` method to get the latest ZK-verified block header hash
        let latest_header = self.latest_header().await?;
        let latest_header_view = self.get_header(latest_header).await?;
        let latest_header_hash = latest_header_view.hash();

        if latest_header_hash != latest_header {
            return Err(DAError {
                error: anyhow!("Light client header mismatch"),
                is_retriable: false,
            });
        }

        let proof = self.get_proof(blob_id, latest_header).await?;
        let head_block_root = latest_header_view.inner_lite.block_merkle_root;

        self.verify_proof(head_block_root, &proof)?;

        let attestation_data = BlobInclusionProof {
            outcome_proof: proof
                .outcome_proof
                .try_into()
                .map_err(to_non_retriable_da_error)?,
            outcome_root_proof: proof.outcome_root_proof,
            block_header_lite: proof
                .block_header_lite
                .try_into()
                .map_err(to_non_retriable_da_error)?,
            block_proof: proof.block_proof,
            head_merkle_root: head_block_root.0,
        };

        Ok(Some(types::InclusionData {
            data: to_vec(&attestation_data).map_err(to_non_retriable_da_error)?,
        }))
    }

    fn clone_boxed(&self) -> Box<dyn DataAvailabilityClient> {
        Box::new(self.clone())
    }

    fn blob_size_limit(&self) -> std::option::Option<usize> {
        Some(1572864)
    }
}
