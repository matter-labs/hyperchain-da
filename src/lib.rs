use std::{fmt};
use async_trait::async_trait;
use crate::clients::celestia::CelestiaClient;

use crate::types::{DispatchResponse, InclusionData};

pub mod clients;
pub mod types;

#[async_trait]
pub trait DataAvailabilityInterface: Sync + Send + fmt::Debug {
    async fn dispatch_blob(
        &self,
        batch_number: u32,
        data: Vec<u8>,
    ) -> Result<DispatchResponse, types::DataAvailabilityError>;
    async fn get_inclusion_data(&self, blob_id: Vec<u8>) -> Result<Option<InclusionData>, types::DataAvailabilityError>;
}

pub async fn new_da_layer_client(da_layer_name: String, private_key: Vec<u8>) -> Box<dyn DataAvailabilityInterface> {
    match da_layer_name.to_lowercase().as_ref() {
        "celestia" => Box::new(CelestiaClient::new(private_key)),
        // "some_da_layer" => Box::new(), // TODO: Implement some_da_layer client
        _ => panic!("Unknown DA layer")
    }
}
