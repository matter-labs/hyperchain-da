pub type DataAvailabilityError = String;

#[derive(Default)]
pub struct DispatchResponse {
    pub blob_id: Vec<u8>,
}

#[derive(Default)]
pub struct InclusionData {
    pub data: Vec<u8>,
}
