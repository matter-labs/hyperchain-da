pub enum DataAvailabilityError {
    CelestiaError(String),
    EigenError(String),
    AvailError(String),
}

#[derive(Default)]
pub struct DispatchResponse {
    pub blob_id: Vec<u8>,
}

#[derive(Default)]
pub struct InclusionData {
    pub data: Vec<u8>,
}
