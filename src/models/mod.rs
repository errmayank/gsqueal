use google::Settings;
use serde::{Deserialize, Serialize};

pub mod cli;
pub mod google;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceUpdateDto {
    pub settings: Settings,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Network {
    pub last_operation: Option<NetworkOperation>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NetworkOperation {
    pub project_id: String,
    pub instance_id: String,
    pub network_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkUpdateDto {
    pub name: String,
    pub value: String,
}
