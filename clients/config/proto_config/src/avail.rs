use anyhow::Context;
use da_config::avail::AvailConfig;
use zksync_protobuf::{required, ProtoRepr};

use crate::proto::avail as proto;

impl ProtoRepr for proto::AvailConfig {
    type Type = AvailConfig;

    fn read(&self) -> anyhow::Result<Self::Type> {
        let avail = self.avail.clone().context("avail")?;
        Ok(Self::Type {
            api_node_url: if avail.gas_relay_mode.unwrap_or(false) {
                None
            } else {
                Some(avail.api_node_url.context("api_node_url")?.clone())
            },
            bridge_api_url: required(&avail.bridge_api_url)
                .context("bridge_api_url")?
                .clone(),
            seed: if avail.gas_relay_mode.unwrap_or(false) {
                None
            } else {
                Some(avail.seed.context("seed")?.clone())
            },
            app_id: if avail.gas_relay_mode.unwrap_or(false) {
                None
            } else {
                Some(avail.app_id.context("app_id")?)
            },
            timeout: *required(&avail.timeout).context("timeout")? as usize,
            max_retries: *required(&avail.max_retries).context("max_retries")? as usize,
            gas_relay_mode: *required(&avail.gas_relay_mode).context("gas_relay_mode")? as bool,
            // if gas_relay_mode is true, then we need to set the gas_relay_api_url and gas_relay_api_key
            gas_relay_api_url: if avail.gas_relay_mode.unwrap_or(false) {
                Some(
                    avail
                        .gas_relay_api_url
                        .context("gas_relay_api_url")?
                        .clone(),
                )
            } else {
                None
            },
            gas_relay_api_key: if avail.gas_relay_mode.unwrap_or(false) {
                Some(
                    avail
                        .gas_relay_api_key
                        .context("gas_relay_api_key")?
                        .clone(),
                )
            } else {
                None
            },
        })
    }

    fn build(this: &Self::Type) -> Self {
        Self {
            avail: Some(proto::Avail {
                api_node_url: this.api_node_url.clone(),
                bridge_api_url: Some(this.bridge_api_url.clone()),
                seed: this.seed.clone(),
                app_id: this.app_id,
                timeout: Some(this.timeout as u64),
                max_retries: Some(this.max_retries as u64),
                gas_relay_mode: Some(this.gas_relay_mode),
                gas_relay_api_url: this.gas_relay_api_url.clone(),
                gas_relay_api_key: this.gas_relay_api_key.clone(),
            }),
        }
    }
}
