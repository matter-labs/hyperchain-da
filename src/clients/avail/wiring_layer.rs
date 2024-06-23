use crate::clients::avail::{config::AvailConfig, client::AvailClient};

use zksync_node_framework::{
    service::ServiceContext,
    wiring_layer::{WiringError, WiringLayer},
};
use zksync_node_framework::implementations::resources::da_client::DAClientResource;
use zksync_da_client::DataAvailabilityClient;

#[derive(Debug)]
pub struct AvailWiringLayer {
    config: AvailConfig,
}

impl AvailWiringLayer {
    pub fn new(
        config: AvailConfig,
    ) -> Self {
        Self {
            config,
        }
    }
}

#[async_trait::async_trait]
impl WiringLayer for AvailWiringLayer {
    fn layer_name(&self) -> &'static str {
        "avail_client_layer"
    }

    async fn wire(self: Box<Self>, mut context: ServiceContext<'_>) -> Result<(), WiringError> {
        let client: Box<dyn DataAvailabilityClient> = Box::new(AvailClient::new()?);

        context.insert_resource(DAClientResource(client))?;

        Ok(())
    }
}
