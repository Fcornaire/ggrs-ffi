use ggrs::P2PSession;

use crate::GGRSConfig;

pub struct Netplay {
    session: Option<*mut P2PSession<GGRSConfig>>,
}

impl Netplay {
    pub fn new(session: Option<*mut P2PSession<GGRSConfig>>) -> Self {
        Self { session }
    }

    pub fn session(&self) -> Option<*mut P2PSession<GGRSConfig>> {
        self.session
    }

    pub fn update_session(&mut self, session: *mut P2PSession<GGRSConfig>) {
        self.session = Some(session);
    }
}

unsafe impl Send for Netplay {}
