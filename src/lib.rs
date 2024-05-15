use std::fmt;

use zksync_types::L1BatchNumber;
use zksync_config::configs::da_dispatcher::DataAvailabilityMode;
use crate::types::{DispatchResponse, InclusionData};

pub mod clients;
mod types;

pub trait DataAvailabilityInterface: Sync + Send + fmt::Debug {
    fn dispatch_blob(
        &self,
        batch_number: L1BatchNumber,
        data: Vec<u8>,
    ) -> Result<DispatchResponse, types::Error>;
    fn get_inclusion_data(&self, blob_id: Vec<u8>) -> Result<InclusionData, types::Error>;
}

pub fn new_da_client(config: zksync_config::DADispatcherConfig) -> Box<dyn DataAvailabilityInterface> {
    match config.mode {
        DataAvailabilityMode::GCS(config) => {
            Box::new(clients::gcs::GCSDAClient::new(config))
        }
        DataAvailabilityMode::NoDA => {
            Box::new(1) // TODO: Implement NoDA client
        }
        DataAvailabilityMode::DALayer(config) => {
            match config.name.as_str() {
                "some_da_layer" => Box::new(2), // TODO: Implement some_da_layer client
                _ => panic!("Unknown DA layer")
            }
        }
    }
}
