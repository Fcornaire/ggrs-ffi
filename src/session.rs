use ggrs::{GGRSError, GGRSEvent, GGRSRequest, NetworkStats, P2PSession, SyncTestSession};

use crate::{model::input::Input, neplay::Netplay, GGRSConfig};

pub enum SessionType {
    P2P(P2PSession<GGRSConfig>),
    Test(SyncTestSession<GGRSConfig>),
}

impl SessionType {
    pub fn new_p2p(p2p: P2PSession<GGRSConfig>) -> Self {
        Self::P2P(p2p)
    }

    pub fn new_test(test: SyncTestSession<GGRSConfig>) -> Self {
        Self::Test(test)
    }

    pub fn p2p(&mut self) -> &mut P2PSession<GGRSConfig> {
        match self {
            Self::P2P(p2p) => p2p,
            Self::Test(_) => panic!("Wrong call! this is a Test session"),
        }
    }

    pub fn test(&mut self) -> &mut SyncTestSession<GGRSConfig> {
        match self {
            Self::Test(test) => test,
            Self::P2P(_) => panic!("Wrong call! this is a P2P session"),
        }
    }
}

pub trait Session {
    fn events(&mut self, netplay: &mut Netplay) -> Vec<&'static str>;
    fn poll_remote(&mut self);
    fn add_local_input(&mut self, player_handle: usize, input: Input) -> Result<(), GGRSError>;
    fn advance_frame(&mut self) -> Result<Vec<GGRSRequest<GGRSConfig>>, GGRSError>;
    fn net_stats(&mut self, remote_player_handle: usize) -> Result<NetworkStats, GGRSError>;
    fn get_frames_ahead(&mut self) -> i32;
    fn retrieve(self: Box<Self>) -> SessionType;
}

impl Session for P2PSession<GGRSConfig> {
    fn events(&mut self, netplay: &mut Netplay) -> Vec<&'static str> {
        let mut events: Vec<&'static str> = vec![];

        for (_, event) in (self).events().enumerate() {
            match event {
                GGRSEvent::Synchronizing { addr, total, count } => {
                    let str = format!(
                        "Synchronizing addr {} total {} count {}",
                        addr, total, count
                    );
                    let str: &'static str = Box::leak(str.into_boxed_str());

                    events.push(str)
                }
                GGRSEvent::Synchronized { addr } => {
                    let str = format!("Synchronized addr {}", addr);
                    let str: &'static str = Box::leak(str.into_boxed_str());
                    events.push(str)
                }
                GGRSEvent::Disconnected { addr: _ } => events.push("Disconnected"),

                GGRSEvent::NetworkInterrupted {
                    addr,
                    disconnect_timeout,
                } => {
                    let str = format!(
                        "NetworkInterrupted addr {} disconnect timout {}",
                        addr, disconnect_timeout
                    );
                    let str: &'static str = Box::leak(str.into_boxed_str());
                    events.push(str)
                }

                GGRSEvent::WaitRecommendation { skip_frames } => {
                    netplay.update_skip_frames(skip_frames + 1);

                    let str = format!("WaitRecommendation skip frames {}", skip_frames);
                    let str: &'static str = Box::leak(str.into_boxed_str());
                    events.push(str)
                }

                GGRSEvent::NetworkResumed { addr } => {
                    let str = format!("NetworkResumed addr {}", addr);
                    let str: &'static str = Box::leak(str.into_boxed_str());
                    events.push(str)
                }
            }
        }

        events
    }

    fn poll_remote(&mut self) {
        self.poll_remote_clients();
    }

    fn add_local_input(&mut self, player_handle: usize, input: Input) -> Result<(), GGRSError> {
        self.add_local_input(player_handle, input)
    }

    fn advance_frame(&mut self) -> Result<Vec<GGRSRequest<GGRSConfig>>, GGRSError> {
        self.advance_frame()
    }

    fn net_stats(&mut self, remote_player_handle: usize) -> Result<NetworkStats, GGRSError> {
        self.network_stats(remote_player_handle)
    }

    fn get_frames_ahead(&mut self) -> i32 {
        self.frames_ahead()
    }

    fn retrieve(self: Box<Self>) -> SessionType {
        SessionType::P2P(*self)
    }
}

impl Session for SyncTestSession<GGRSConfig> {
    fn events(&mut self, _netplay: &mut Netplay) -> Vec<&'static str> {
        vec![]
    }

    fn poll_remote(&mut self) {}

    fn add_local_input(&mut self, player_handle: usize, input: Input) -> Result<(), GGRSError> {
        self.add_local_input(player_handle, input)
    }

    fn advance_frame(&mut self) -> Result<Vec<GGRSRequest<GGRSConfig>>, GGRSError> {
        self.advance_frame()
    }

    fn net_stats(&mut self, _remote_player_handle: usize) -> Result<NetworkStats, GGRSError> {
        Ok(NetworkStats::new())
    }

    fn get_frames_ahead(&mut self) -> i32 {
        0
    }

    fn retrieve(self: Box<Self>) -> SessionType {
        SessionType::Test(*self)
    }
}
