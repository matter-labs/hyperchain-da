use anyhow::Context;
use da_config::near::NearConfig;
use zksync_protobuf::{required, ProtoRepr};

use crate::proto::near as proto;

impl ProtoRepr for proto::NearConfig {
    type Type = NearConfig;

    fn read(&self) -> anyhow::Result<Self::Type> {
        let near = self.near.clone().context("near")?;
        Ok(Self::Type {
            light_client_url: required(&near.light_client_url)
                .context("light_client_url")?
                .clone(),
            network: required(&near.network).context("network")?.clone(),
            contract: required(&near.contract).context("contract")?.clone(),
            account_id: required(&near.account_id).context("account_id")?.clone(),
            secret_key: required(&near.secret_key).context("secret_key")?.clone(),
        })
    }

    fn build(this: &Self::Type) -> Self {
        Self {
            near: Some(proto::Near {
                light_client_url: Some(this.light_client_url.clone()),
                network: Some(this.network.clone()),
                contract: Some(this.contract.clone()),
                account_id: Some(this.account_id.clone()),
                secret_key: Some(this.secret_key.clone()),
            }),
        }
    }
}
