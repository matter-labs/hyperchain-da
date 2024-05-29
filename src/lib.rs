use std::{fmt};
use async_trait::async_trait;

use crate::types::{DispatchResponse, InclusionData};

pub mod clients;
pub mod types;

#[async_trait]
pub trait DataAvailabilityClient: Sync + Send + fmt::Debug {
    async fn dispatch_blob(
        &self,
        batch_number: u32,
        data: Vec<u8>,
    ) -> Result<DispatchResponse, anyhow::Error>;

    async fn get_inclusion_data(&self, blob_id: String) -> Result<Option<InclusionData>, anyhow::Error>;
    fn client_name(&self) -> String;
    fn clone_boxed(&self) -> Box<dyn DataAvailabilityClient>;
}

impl Clone for Box<dyn DataAvailabilityClient> {
    fn clone(&self) -> Box<dyn DataAvailabilityClient> {
        self.clone_boxed()
    }
}
