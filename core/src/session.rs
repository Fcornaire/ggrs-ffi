use ggrs::{GGRSError, GGRSEvent, GGRSRequest, NetworkStats, P2PSession, SyncTestSession};
use tracing::{info, warn};

use crate::{
    config::ggrs_config::GGRSConfig, model::input::Input, neplay::Netplay, set_netplay_disconnected,
};

pub enum SessionType {
    P2P(P2PSession<GGRSConfig>),
    Test(SyncTestSession<GGRSConfig>),
}

pub trait Session<Config: ggrs::Config> {
    fn events(&mut self, netplay: &mut Netplay) -> Vec<&'static str>;
    fn poll_remote(&mut self);
    fn is_synchronized(&self) -> bool;
    fn add_local_input(&mut self, player_handle: usize, input: Input) -> Result<(), GGRSError>;
    fn advance_frame(&mut self) -> Result<Vec<GGRSRequest<Config>>, GGRSError>;
    fn net_stats(&mut self, remote_player_handle: usize) -> Result<NetworkStats, GGRSError>;
    fn get_frames_ahead(&mut self) -> i32;
    fn retrieve(self: Box<Self>) -> SessionType;
    fn disconnect_all(&mut self, netplay: &mut Netplay) -> Result<(), GGRSError>;
}

impl Session<GGRSConfig> for P2PSession<GGRSConfig> {
    fn events(&mut self, _netplay: &mut Netplay) -> Vec<&'static str> {
        let mut events: Vec<&'static str> = vec![];

        for event in self.events() {
            info!("Event: {:?}", event);

            match event {
                GGRSEvent::Synchronizing { addr, total, count } => {
                    let str = format!(
                        "Synchronizing with {} total {} count {}",
                        addr, total, count
                    );
                    let str: &'static str = Box::leak(str.into_boxed_str());

                    events.push(str)
                }
                GGRSEvent::Synchronized { addr } => {
                    let str = format!("Synchronized with {addr}");
                    let str: &'static str = Box::leak(str.into_boxed_str());
                    events.push(str)
                }
                GGRSEvent::Disconnected { addr } => {
                    set_netplay_disconnected(true);
                    let str = format!("Disconnected from {addr}");
                    let str: &'static str = Box::leak(str.into_boxed_str());

                    events.push(str)
                }

                GGRSEvent::NetworkInterrupted {
                    addr,
                    disconnect_timeout,
                } => {
                    let str = format!(
                        "NetworkInterrupted with {}, will disconnect in {} ms",
                        addr, disconnect_timeout
                    );
                    let str: &'static str = Box::leak(str.into_boxed_str());
                    events.push(str)
                }

                GGRSEvent::WaitRecommendation { skip_frames } => {
                    let str = format!("WaitRecommendation skip frames {} (Ignored)", skip_frames);
                    let str: &'static str = Box::leak(str.into_boxed_str());
                    events.push(str)
                }

                GGRSEvent::NetworkResumed { addr } => {
                    let str = format!("NetworkResumed with {}", addr);
                    let str: &'static str = Box::leak(str.into_boxed_str());
                    events.push(str)
                }
                GGRSEvent::DesyncDetected {
                    frame: _,
                    local_checksum: _,
                    remote_checksum: _,
                    addr: _,
                } => {
                    // let str = format!("DesyncDetected from {addr} at frame {frame} , local checksum {local_checksum} , remote checksum {remote_checksum}");
                    // let str: &'static str = Box::leak(str.into_boxed_str());
                    // events.push(str)
                }
            }
        }

        events
    }

    fn poll_remote(&mut self) {
        self.poll_remote_clients();
    }

    fn is_synchronized(&self) -> bool {
        self.current_state() == ggrs::SessionState::Running
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

    //TODO: Properly disconnect all players
    fn disconnect_all(&mut self, netplay: &mut Netplay) -> Result<(), GGRSError> {
        match self.disconnect_player(netplay.remote_player_handle() as usize) {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("Error disconnecting player: {:?}", e); //The other probably already disconnected
                Ok(())
            }
        }
    }
}

impl Session<GGRSConfig> for SyncTestSession<GGRSConfig> {
    fn events(&mut self, _netplay: &mut Netplay) -> Vec<&'static str> {
        vec![]
    }

    fn poll_remote(&mut self) {}

    fn is_synchronized(&self) -> bool {
        false
    }

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

    fn disconnect_all(&mut self, _netplay: &mut Netplay) -> Result<(), GGRSError> {
        Ok(())
    }
}
