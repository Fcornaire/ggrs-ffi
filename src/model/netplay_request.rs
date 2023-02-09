use ggrs::GGRSRequest;
use serde::{Deserialize, Serialize};

use crate::GGRSConfig;

#[repr(C)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, PartialEq)]
pub enum NetplayRequest {
    SaveGameState,
    LoadGameState,
    AdvanceFrame,
}

impl NetplayRequest {
    pub fn new(request: &GGRSRequest<GGRSConfig>) -> Self {
        match request {
            GGRSRequest::AdvanceFrame { inputs: _ } => NetplayRequest::AdvanceFrame,
            GGRSRequest::LoadGameState { cell: _, frame: _ } => NetplayRequest::AdvanceFrame,
            GGRSRequest::SaveGameState { cell: _, frame: _ } => NetplayRequest::SaveGameState,
        }
    }
}

#[repr(C)]
pub struct NetplayRequests {
    pub data: *const NetplayRequest,
    pub len: usize,
}

impl NetplayRequests {
    pub fn new(netplay_requests: Vec<NetplayRequest>) -> Self {
        let len = netplay_requests.len();
        let requests = netplay_requests.as_ptr();

        Self {
            data: requests,
            len,
        }
    }

    pub fn empty() -> Self {
        Self {
            data: std::ptr::null_mut(),
            len: 0,
        }
    }
}
