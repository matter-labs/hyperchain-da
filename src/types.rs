use zksync_object_store::ObjectStoreError;

pub enum DataAvailabilityError {
    GCSClientError(ObjectStoreError)
    // TODO: add errors for different client implementations here
}

#[derive(Default)]
pub struct DispatchResponse {
    pub(crate) blob_id: Vec<u8>,
}

#[derive(Default)]
pub struct InclusionData {
    data: Vec<u8>,
}
