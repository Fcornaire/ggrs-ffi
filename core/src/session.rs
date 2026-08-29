use tracing::{info, warn};

use ggrs::{
    GgrsError, GgrsEvent, GgrsRequest, NetworkStats, P2PSession, SpectatorSession, SyncTestSession,
};

use crate::{
    config::ggrs_config::GGRSConfig, model::input::Input, neplay::Netplay, set_netplay_disconnected,
};

pub enum SessionType {
    P2P(P2PSession<GGRSConfig>),
    Test(SyncTestSession<GGRSConfig>),
    Spectate(SpectatorSession<GGRSConfig>),
}

pub trait Session<Config: ggrs::Config> {
    fn events(&mut self, netplay: &mut Netplay) -> Vec<String>;
    fn poll_remote(&mut self);
    fn is_synchronized(&self) -> bool;
    fn add_local_input(&mut self, player_handle: usize, input: Input) -> Result<(), GgrsError>;
    fn advance_frame(&mut self) -> Result<Vec<GgrsRequest<Config>>, GgrsError>;
    fn net_stats(&mut self, remote_player_handle: usize) -> Result<NetworkStats, GgrsError>;
    fn get_frames_ahead(&mut self) -> i32;
    fn retrieve(self: Box<Self>) -> SessionType;
    fn disconnect_all(&mut self, netplay: &mut Netplay) -> Result<(), GgrsError>;
}

impl Session<GGRSConfig> for P2PSession<GGRSConfig> {
    fn events(&mut self, netplay: &mut Netplay) -> Vec<String> {
        let mut events: Vec<String> = vec![];

        for event in self.events() {
            info!("Event: {:?}", event);

            match event {
                GgrsEvent::Synchronizing { addr, total, count } => {
                    let str = format!(
                        "Synchronizing with {} total {} count {}",
                        addr, total, count
                    );
                    events.push(str)
                }
                GgrsEvent::Synchronized { addr } => {
                    if !netplay.is_a_remote_player(addr.clone()) {
                        continue;
                    }

                    let str = format!("Synchronized with {addr}");
                    events.push(str)
                }
                GgrsEvent::Disconnected { addr } => {
                    if !netplay.is_a_remote_player(addr.clone()) {
                        continue;
                    }

                    if netplay.remove_remote_player(&addr) == 0 {
                        set_netplay_disconnected(true);
                    }

                    let str = format!("Disconnected from {addr}");
                    events.push(str)
                }

                GgrsEvent::NetworkInterrupted {
                    addr,
                    disconnect_timeout,
                } => {
                    if !netplay.is_a_remote_player(addr.clone()) {
                        continue;
                    }

                    let str = format!(
                        "NetworkInterrupted with {}, will disconnect in {} ms",
                        addr, disconnect_timeout
                    );
                    events.push(str)
                }

                GgrsEvent::WaitRecommendation { skip_frames } => {
                    let str = format!("WaitRecommendation skip frames {} (Ignored)", skip_frames);
                    events.push(str)
                }

                GgrsEvent::NetworkResumed { addr } => {
                    if !netplay.is_a_remote_player(addr.clone()) {
                        continue;
                    }

                    let str = format!("NetworkResumed with {}", addr);
                    events.push(str)
                }
                GgrsEvent::DesyncDetected {
                    frame,
                    local_checksum,
                    remote_checksum,
                    addr,
                } => {
                    let str = format!("DesyncDetected from {addr} at frame {frame} , local checksum {local_checksum} , remote checksum {remote_checksum}");
                    events.push(str)
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

    fn add_local_input(&mut self, player_handle: usize, input: Input) -> Result<(), GgrsError> {
        self.add_local_input(player_handle, input)
    }

    fn advance_frame(&mut self) -> Result<Vec<GgrsRequest<GGRSConfig>>, GgrsError> {
        self.advance_frame()
    }

    fn net_stats(&mut self, remote_player_handle: usize) -> Result<NetworkStats, GgrsError> {
        self.network_stats(remote_player_handle)
    }

    fn get_frames_ahead(&mut self) -> i32 {
        self.frames_ahead()
    }

    fn retrieve(self: Box<Self>) -> SessionType {
        SessionType::P2P(*self)
    }

    //TODO: Properly disconnect all players
    fn disconnect_all(&mut self, netplay: &mut Netplay) -> Result<(), GgrsError> {
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
    fn events(&mut self, _netplay: &mut Netplay) -> Vec<String> {
        vec![]
    }

    fn poll_remote(&mut self) {}

    fn is_synchronized(&self) -> bool {
        false
    }

    fn add_local_input(&mut self, player_handle: usize, input: Input) -> Result<(), GgrsError> {
        self.add_local_input(player_handle, input)
    }

    fn advance_frame(&mut self) -> Result<Vec<GgrsRequest<GGRSConfig>>, GgrsError> {
        self.advance_frame()
    }

    fn net_stats(&mut self, _remote_player_handle: usize) -> Result<NetworkStats, GgrsError> {
        Ok(NetworkStats::new())
    }

    fn get_frames_ahead(&mut self) -> i32 {
        0
    }

    fn retrieve(self: Box<Self>) -> SessionType {
        SessionType::Test(*self)
    }

    fn disconnect_all(&mut self, _netplay: &mut Netplay) -> Result<(), GgrsError> {
        Ok(())
    }
}

impl Session<GGRSConfig> for SpectatorSession<GGRSConfig> {
    fn events(&mut self, netplay: &mut Netplay) -> Vec<String> {
        let mut events: Vec<String> = vec![];

        for event in (self).events() {
            match event {
                GgrsEvent::Synchronizing { addr, total, count } => {
                    let str = format!(
                        "Synchronizing with {} total {} count {}",
                        addr, total, count
                    );
                    println!("{}", str);

                    events.push(str)
                }
                GgrsEvent::Synchronized { addr } => {
                    let str = format!("Synchronized with {addr}");
                    println!("{}", str);

                    events.push(str)
                }
                GgrsEvent::Disconnected { addr } => {
                    netplay.mark_host_gone();

                    let str = format!("Disconnected from {addr}");
                    println!("{}", str);

                    events.push(str)
                }

                GgrsEvent::NetworkInterrupted {
                    addr,
                    disconnect_timeout,
                } => {
                    let str = format!(
                        "NetworkInterrupted with {}, will disconnect in {} ms",
                        addr, disconnect_timeout
                    );
                    println!("{}", str);

                    events.push(str)
                }

                GgrsEvent::WaitRecommendation { skip_frames } => {
                    let str = format!("WaitRecommendation skip frames {} (Ignored)", skip_frames);
                    println!("{}", str);

                    events.push(str)
                }

                GgrsEvent::NetworkResumed { addr } => {
                    let str = format!("NetworkResumed with {}", addr);
                    println!("{}", str);

                    events.push(str)
                }
                GgrsEvent::DesyncDetected {
                    frame,
                    local_checksum,
                    remote_checksum,
                    addr,
                } => {
                    let str = format!("DesyncDetected from {addr} at frame {frame} , local checksum {local_checksum} , remote checksum {remote_checksum}");
                    events.push(str)
                }
            }
        }

        events
    }

    fn poll_remote(&mut self) {
        self.poll_remote_clients();
    }

    fn advance_frame(&mut self) -> Result<Vec<GgrsRequest<GGRSConfig>>, GgrsError> {
        self.advance_frame()
    }

    fn is_synchronized(&self) -> bool {
        self.current_state() == ggrs::SessionState::Running
    }

    fn add_local_input(&mut self, _player_handle: usize, _input: Input) -> Result<(), GgrsError> {
        Ok(())
    }

    fn get_frames_ahead(&mut self) -> i32 {
        0
    }

    fn net_stats(&mut self, _remote_player_handle: usize) -> Result<NetworkStats, GgrsError> {
        self.network_stats()
    }

    fn disconnect_all(&mut self, _netplay: &mut Netplay) -> Result<(), GgrsError> {
        Ok(())
    }

    fn retrieve(self: Box<Self>) -> SessionType {
        SessionType::Spectate(*self)
    }
}
