use serde::{Deserialize, Serialize};

use crate::core::unmanaged::safe_bytes::SafeBytes;

#[derive(Clone, Debug, PartialOrd, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AppConfig {
    pub input_delay: i32,
    name: String, //TODO: remove this, useless here
    pub netplay: NetplayConfig,
    pub test: Option<TestConfig>,
}

impl AppConfig {
    pub unsafe fn new(safe_bytes: SafeBytes) -> Self {
        serde_json::from_slice(safe_bytes.slice()).unwrap()
    }

    pub fn is_test(&self) -> bool {
        self.test.is_some()
    }
}

#[derive(Clone, Debug, PartialOrd, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NetplayConfig {
    pub num_players: i32,
    pub spectators: Option<Vec<String>>,
    pub players: Option<Vec<String>>,
    pub local_conf: Option<NetplayLocalConfig>,
    pub server_conf: Option<NetplayServerConfig>,
    pub spectator_conf: Option<NetplaySpectatorConfig>,
}

#[derive(Clone, Debug, PartialOrd, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NetplayLocalConfig {
    pub remote_addr: String,
    pub port: u16,
    pub player_draw: u32,
}

#[derive(Clone, Debug, PartialOrd, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NetplayServerConfig {
    pub room_url: Option<String>,
    pub is_host: bool,
}

//Add a spectator config
#[derive(Clone, Debug, PartialOrd, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NetplaySpectatorConfig {
    pub room_url: Option<String>,
    pub to_spectate: Option<String>,
}

#[derive(Clone, Debug, PartialOrd, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TestConfig {
    pub check_distance: i32,
}
