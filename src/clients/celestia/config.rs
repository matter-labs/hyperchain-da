use serde::Deserialize;

// feel free to redefine all the fields in this struct, this is just a placeholder
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct CelestiaConfig {
    pub light_node_url: String,
    pub private_key: String,
}
