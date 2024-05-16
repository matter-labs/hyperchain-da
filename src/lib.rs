use std::{fmt};
use async_trait::async_trait;

use zksync_types::L1BatchNumber;
use zksync_config::configs::{da_dispatcher::{DataAvailabilityMode, DADispatcherConfig}};
use crate::types::{DispatchResponse, InclusionData};

pub mod clients;
mod types;

#[async_trait]
pub trait DataAvailabilityInterface: Sync + Send + fmt::Debug {
    async fn dispatch_blob(
        &self,
        batch_number: L1BatchNumber,
        data: Vec<u8>,
    ) -> Result<DispatchResponse, types::DataAvailabilityError>;
    async fn get_inclusion_data(&self, blob_id: Vec<u8>) -> Result<InclusionData, types::DataAvailabilityError>;
}

pub async fn new_da_client(config: DADispatcherConfig) -> Box<dyn DataAvailabilityInterface> {
    match config.mode {
        DataAvailabilityMode::GCS(config) => {
            Box::new(clients::gcs::GCSDAClient::new(config).await)
        }
        DataAvailabilityMode::NoDA => {
            // TODO: Implement NoDA client
            panic!("NoDA client is not implemented")
        }
        DataAvailabilityMode::DALayer(config) => {
            match config.name.as_str() {
                // "some_da_layer" => Box::new(), // TODO: Implement some_da_layer client
                _ => panic!("Unknown DA layer")
            }
        }
    }
}
