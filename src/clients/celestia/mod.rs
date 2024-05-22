use std::fmt::{Debug, Formatter};
use async_trait::async_trait;
use crate::{DataAvailabilityInterface, types};

pub struct CelestiaClient {
    private_key: Vec<u8>,
}

impl CelestiaClient {
    pub fn new(private_key: Vec<u8>) -> Self {
        Self {
            private_key
        }
    }
}

#[async_trait]
impl DataAvailabilityInterface for CelestiaClient {
    async fn dispatch_blob(
        &self,
        batch_number: u32,
        data: Vec<u8>,
    ) -> Result<types::DispatchResponse, types::DataAvailabilityError> {
        todo!()
    }

    async fn get_inclusion_data(&self, blob_id: Vec<u8>) -> Result<Option<types::InclusionData>, types::DataAvailabilityError> {
        todo!()
    }
}

impl Debug for CelestiaClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CelestiaClient")
            .finish()
    }
}
