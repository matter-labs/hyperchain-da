use anyhow::Context;
use da_config::avail::AvailConfig;
use zksync_protobuf::{ProtoRepr, required};

use crate::proto::avail as proto;

impl ProtoRepr for proto::Avail {
    type Type = AvailConfig;

    fn read(&self) -> anyhow::Result<Self::Type> {
        Ok(Self::Type {
            api_node_url: required(&self.api_node_url).context("api_node_url")?.clone(),
            bridge_api_url: required(&self.bridge_api_url).context("bridge_api_url")?.clone(),
            seed: required(&self.seed).context("seed")?.clone(),
            app_id: *required(&self.app_id).context("app_id")?,
            timeout: *required(&self.timeout).context("timeout")? as usize,
            max_retries: *required(&self.max_retries).context("max_retries")? as usize,
        })
    }

    fn build(this: &Self::Type) -> Self {
        Self {
            api_node_url: Some(this.api_node_url.clone()),
            bridge_api_url: Some(this.bridge_api_url.clone()),
            seed: Some(this.seed.clone()),
            app_id: Some(this.app_id),
            timeout: Some(this.timeout as u64),
            max_retries: Some(this.max_retries as u64),
        }
    }
}
