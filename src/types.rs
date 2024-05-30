use std::fmt::Display;
use serde::Serialize;

#[derive(Debug)]
pub struct DAError {
    error: anyhow::Error,
    is_transient: bool,
}

impl Display for DAError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DAError: {}, is_transient: {}", self.error, self.is_transient)
    }
}

/// DispatchResponse is the response received from the DA layer after dispatching a blob.
#[derive(Default)]
pub struct DispatchResponse {
    /// The blob_id is needed to fetch the inclusion data.
    pub blob_id: String,
}

/// InclusionData is the data needed to prove that a blob is included in the DA layer.
#[derive(Default, Serialize)]
pub struct InclusionData {
    /// The inclusion data serialized by the DA client. Serialization is done in a way that allows
    /// the deserialization of the data in Solidity contracts.
    pub data: Vec<u8>,
}
