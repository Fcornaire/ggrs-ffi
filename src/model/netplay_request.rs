use ggrs::GGRSRequest;
use serde::{Deserialize, Serialize};

use crate::GGRSConfig;

#[repr(C)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, PartialEq)]
pub enum NetplayRequest {
    SaveGameState = 0,
    LoadGameState = 1,
    AdvanceFrame = 2,
}

impl NetplayRequest {
    pub fn new(request: &GGRSRequest<GGRSConfig>) -> Self {
        match request {
            GGRSRequest::AdvanceFrame { inputs: _ } => NetplayRequest::AdvanceFrame,
            GGRSRequest::LoadGameState { cell: _, frame: _ } => NetplayRequest::LoadGameState,
            GGRSRequest::SaveGameState { cell: _, frame: _ } => NetplayRequest::SaveGameState,
        }
    }
}
