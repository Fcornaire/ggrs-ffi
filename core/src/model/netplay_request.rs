use ggrs::GgrsRequest;
use serde::{Deserialize, Serialize};

use crate::config::ggrs_config::GGRSConfig;

#[repr(C)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, PartialEq)]
pub enum NetplayRequest {
    SaveGameState = 0,
    LoadGameState = 1,
    AdvanceFrame = 2,
}

impl NetplayRequest {
    pub fn new(request: &GgrsRequest<GGRSConfig>) -> Self {
        match request {
            GgrsRequest::AdvanceFrame { inputs: _ } => NetplayRequest::AdvanceFrame,
            GgrsRequest::LoadGameState { cell: _, frame: _ } => NetplayRequest::LoadGameState,
            GgrsRequest::SaveGameState { cell: _, frame: _ } => NetplayRequest::SaveGameState,
        }
    }
}
