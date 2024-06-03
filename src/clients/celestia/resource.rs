use crate::DataAvailabilityClient;
use zksync_node_framework::resource::Resource;

/// Represents a client of a certain DA solution.
#[derive(Clone)]
pub struct CelestiaClientResource(pub Box<dyn DataAvailabilityClient>);

impl Resource for CelestiaClientResource {
    fn name() -> String {
        "da_clients/celestia".into()
    }
}
