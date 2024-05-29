use std::{fmt};
use async_trait::async_trait;

use crate::types::{DispatchResponse, InclusionData};

pub mod clients;
pub mod types;
pub mod config;

#[async_trait]
pub trait DataAvailabilityClient: Sync + Send + fmt::Debug {
    async fn dispatch_blob(
        &self,
        batch_number: u32,
        data: Vec<u8>,
    ) -> Result<DispatchResponse, anyhow::Error>;
    async fn get_inclusion_data(&self, blob_id: Vec<u8>) -> Result<Option<InclusionData>, anyhow::Error>;
    fn name() -> String;
}
