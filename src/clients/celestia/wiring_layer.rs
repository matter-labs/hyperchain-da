use zksync_config::configs::da_dispatcher::{DADispatcherConfig};
use zksync_env_config::FromEnv;

use zksync_node_framework::{
    service::ServiceContext,
    wiring_layer::{WiringError, WiringLayer},
};

use crate::clients::celestia::{CelestiaClient, config::CelestiaConfig};
use crate::clients::celestia::resource::CelestiaClientResource;
use crate::DataAvailabilityClient;

#[derive(Debug)]
pub struct CelestiaWiringLayer {
    da_config: DADispatcherConfig,
}

impl CelestiaWiringLayer {
    pub fn new(
        da_config: DADispatcherConfig,
    ) -> Self {
        Self {
            da_config,
        }
    }
}

#[async_trait::async_trait]
impl WiringLayer for CelestiaWiringLayer {
    fn layer_name(&self) -> &'static str {
        "celestia_client_layer"
    }

    async fn wire(self: Box<Self>, mut context: ServiceContext<'_>) -> Result<(), WiringError> {
        let config = CelestiaConfig::from_env();
        let client: Box<dyn DataAvailabilityClient> = Box::new(CelestiaClient::new(config?));

        context.insert_resource(CelestiaClientResource(client))?;

        Ok(())
    }
}
