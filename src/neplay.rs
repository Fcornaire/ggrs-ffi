use ggrs::{GGRSRequest, P2PSession};

use crate::{model::netplay_request::NetplayRequest, GGRSConfig};

pub struct Netplay {
    session: Option<*mut P2PSession<GGRSConfig>>,
    requests: Vec<GGRSRequest<GGRSConfig>>,
}

impl Netplay {
    pub const fn new(session: Option<*mut P2PSession<GGRSConfig>>) -> Self {
        Self {
            session,
            requests: vec![],
        }
    }

    pub fn session(&self) -> Option<*mut P2PSession<GGRSConfig>> {
        self.session
    }

    pub fn update_session(&mut self, session: *mut P2PSession<GGRSConfig>) {
        self.session = Some(session);
    }

    pub fn requests(&self) -> Vec<NetplayRequest> {
        self.requests
            .iter()
            .map(|req| NetplayRequest::new(req))
            .collect()
    }

    pub fn update_requests(&mut self, requests: Vec<GGRSRequest<GGRSConfig>>) {
        self.requests = requests;
    }
}

unsafe impl Send for Netplay {}
