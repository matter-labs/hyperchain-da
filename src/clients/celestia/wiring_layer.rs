use crate::clients::celestia::{config::CelestiaConfig, client::CelestiaClient};

use zksync_node_framework::{
    service::ServiceContext,
    wiring_layer::{WiringError, WiringLayer},
};
use zksync_node_framework::implementations::resources::da_client::DAClientResource;
use zksync_da_client::DataAvailabilityClient;

#[derive(Debug)]
pub struct CelestiaWiringLayer {
    config: CelestiaConfig,
}

impl CelestiaWiringLayer {
    pub fn new(
        config: CelestiaConfig,
    ) -> Self {
        Self {
            config,
        }
    }
}

#[async_trait::async_trait]
impl WiringLayer for CelestiaWiringLayer {
    fn layer_name(&self) -> &'static str {
        "celestia_client_layer"
    }

    async fn wire(self: Box<Self>, mut context: ServiceContext<'_>) -> Result<(), WiringError> {
        let client: Box<dyn DataAvailabilityClient> = Box::new(CelestiaClient::new()?);

        context.insert_resource(DAClientResource(client))?;

        Ok(())
    }
}
