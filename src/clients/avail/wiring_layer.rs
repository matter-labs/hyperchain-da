use crate::clients::avail::{client::AvailClient, config::AvailConfig};

use zksync_da_client::DataAvailabilityClient;
use zksync_node_framework::implementations::resources::da_client::DAClientResource;
use zksync_node_framework::{
    service::ServiceContext,
    IntoContext,
    wiring_layer::{WiringError, WiringLayer},
};

#[derive(Debug)]
pub struct AvailWiringLayer {
    config: AvailConfig,
}

impl AvailWiringLayer {
    pub fn new(config: AvailConfig) -> Self {
        Self { config }
    }
}

#[derive(Debug, IntoContext)]
pub struct Output {
    pub client: DAClientResource,
}

#[async_trait::async_trait]
impl WiringLayer for AvailWiringLayer {
    type Input = ();
    type Output = Output;

    fn layer_name(&self) -> &'static str {
        "avail_client_layer"
    }

    async fn wire(self, _input: Self::Input) -> Result<Self::Output, WiringError> {
        let client: Box<dyn DataAvailabilityClient> = Box::new(AvailClient::new().await?);

        Ok(Self::Output {
            client: DAClientResource(client),
        })
    }
}
