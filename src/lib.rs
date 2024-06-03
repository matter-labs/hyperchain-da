use std::{fmt};
use async_trait::async_trait;

use crate::types::{DispatchResponse, InclusionData, DAError};

pub mod clients;
pub mod types;
pub mod config;
mod test_utils;

/// Trait that defines the interface for the data availability layer clients.
#[async_trait]
pub trait DataAvailabilityClient: Sync + Send + fmt::Debug {
    /// Dispatches a blob to the data availability layer.
    async fn dispatch_blob(
        &self,
        batch_number: u32,
        data: Vec<u8>,
    ) -> Result<DispatchResponse, DAError>;

    /// Fetches the inclusion data for a given blob_id.
    async fn get_inclusion_data(&self, blob_id: String) -> Result<Option<InclusionData>, DAError>;

    /// Clones the client and wraps it in a Box.
    fn clone_boxed(&self) -> Box<dyn DataAvailabilityClient>;

    /// Returns the maximum size of the blob (in bytes) that can be dispatched.
    fn blob_size_limit(&self) -> usize;
}

impl Clone for Box<dyn DataAvailabilityClient> {
    fn clone(&self) -> Box<dyn DataAvailabilityClient> {
        self.clone_boxed()
    }
}
