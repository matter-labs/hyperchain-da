use anyhow::Context;
use da_config::avail::AvailConfig;
use zksync_protobuf::{ProtoRepr, required};

use crate::proto::avail as proto;

impl ProtoRepr for proto::AvailConfig {
    type Type = AvailConfig;

    fn read(&self) -> anyhow::Result<Self::Type> {
        let avail = self.avail.clone().context("avail")?;
        Ok(Self::Type {
            api_node_url: required(&avail.api_node_url).context("api_node_url")?.clone(),
            bridge_api_url: required(&avail.bridge_api_url).context("bridge_api_url")?.clone(),
            seed: required(&avail.seed).context("seed")?.clone(),
            app_id: *required(&avail.app_id).context("app_id")?,
            timeout: *required(&avail.timeout).context("timeout")? as usize,
            max_retries: *required(&avail.max_retries).context("max_retries")? as usize,
        })
    }

    fn build(this: &Self::Type) -> Self {
        Self {
            avail: Some(proto::Avail {
                api_node_url: Some(this.api_node_url.clone()),
                bridge_api_url: Some(this.bridge_api_url.clone()),
                seed: Some(this.seed.clone()),
                app_id: Some(this.app_id),
                timeout: Some(this.timeout as u64),
                max_retries: Some(this.max_retries as u64),
            }),
        }
    }
}
