use serde::Deserialize;
use crate::clients::celestia::config::CelestiaConfig;

/// Enum representing the configuration for the different data availability layers
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(tag = "config")]
pub enum DALayerConfig {
    Celestia(CelestiaConfig),
}
