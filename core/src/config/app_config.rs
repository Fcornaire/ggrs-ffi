use serde::{Deserialize, Serialize};
use tracing::error;

use crate::core::unmanaged::safe_bytes::SafeBytes;

#[derive(Clone, Debug, PartialOrd, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AppConfig {
    pub input_delay: i32,
    name: String, //TODO: remove this, useless here
    pub netplay: NetplayConfig,
    pub test: Option<TestConfig>,
    #[serde(default)]
    pub fps: i32,
}

impl AppConfig {
    pub fn fps(&self) -> usize {
        if self.fps > 0 {
            self.fps as usize
        } else {
            60
        }
    }
}

impl AppConfig {
    pub unsafe fn new(safe_bytes: SafeBytes) -> Self {
        match serde_json::from_slice(safe_bytes.slice()) {
            Ok(config) => config,
            Err(e) => {
                error!("Failed to deserialize AppConfig: {}", e);
                panic!("AppConfig deserialization failed");
            }
        }
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
    #[serde(default)]
    pub local_peer_id: Option<String>,
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
    #[serde(default)]
    pub allow_late_spectators: Option<bool>,
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
