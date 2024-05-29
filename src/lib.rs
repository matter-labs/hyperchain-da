use std::{fmt};
use async_trait::async_trait;

use crate::types::{DispatchResponse, InclusionData};

pub mod clients;
pub mod types;

/// Trait that defines the interface for the data availability layer clients.
#[async_trait]
pub trait DataAvailabilityClient: Sync + Send + fmt::Debug {
    /// Dispatches a blob to the data availability layer.
    async fn dispatch_blob(
        &self,
        batch_number: u32,
        data: Vec<u8>,
    ) -> Result<DispatchResponse, anyhow::Error>;

    /// Fetches the inclusion data for a given blob_id.
    async fn get_inclusion_data(&self, blob_id: String) -> Result<Option<InclusionData>, anyhow::Error>;

    /// Clones the client and wraps it in a Box.
    fn clone_boxed(&self) -> Box<dyn DataAvailabilityClient>;
}

impl Clone for Box<dyn DataAvailabilityClient> {
    fn clone(&self) -> Box<dyn DataAvailabilityClient> {
        self.clone_boxed()
    }
}
