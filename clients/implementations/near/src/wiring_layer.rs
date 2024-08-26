use zksync_da_client::DataAvailabilityClient;
use zksync_node_framework::implementations::resources::da_client::DAClientResource;
use zksync_node_framework::{
    wiring_layer::{WiringError, WiringLayer},
    IntoContext,
};

use crate::client::NearClient;

#[derive(Debug, Default)]
pub struct NearWiringLayer {}

impl NearWiringLayer {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, IntoContext)]
pub struct Output {
    pub client: DAClientResource,
}

#[async_trait::async_trait]
impl WiringLayer for NearWiringLayer {
    type Input = ();
    type Output = Output;

    fn layer_name(&self) -> &'static str {
        "near_client_layer"
    }

    async fn wire(self, _input: Self::Input) -> Result<Self::Output, WiringError> {
        let client: Box<dyn DataAvailabilityClient> = Box::new(NearClient::new().await?);

        Ok(Self::Output {
            client: DAClientResource(client),
        })
    }
}
