use crate::client::CelestiaClient;

use zksync_da_client::DataAvailabilityClient;
use zksync_node_framework::implementations::resources::da_client::DAClientResource;
use zksync_node_framework::{
    wiring_layer::{WiringError, WiringLayer},
    IntoContext,
};

#[derive(Debug, Default)]
pub struct CelestiaWiringLayer;

#[derive(Debug, IntoContext)]
pub struct Output {
    pub client: DAClientResource,
}

#[async_trait::async_trait]
impl WiringLayer for CelestiaWiringLayer {
    type Input = ();
    type Output = Output;

    fn layer_name(&self) -> &'static str {
        "celestia_client_layer"
    }

    async fn wire(self, _: Self::Input) -> Result<Self::Output, WiringError> {
        let client: Box<dyn DataAvailabilityClient> = Box::new(CelestiaClient::new()?);

        Ok(Self::Output {
            client: DAClientResource(client),
        })
    }
}
