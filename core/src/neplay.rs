use futures::{select, FutureExt};
use futures_timer::Delay;
use matchbox_socket::{PeerId, WebRtcSocket};
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant};
use tracing::{error, info, warn};
use uuid::Uuid;

use ggrs::{
    DesyncDetection, GgrsError, GgrsRequest, InputStatus, PlayerType, SessionBuilder,
    SyncTestSession, UdpNonBlockingSocket,
};

use crate::core::unmanaged::safe_bytes::SafeBytes;
use crate::exts::WebRtcSocketGgrsExtensions;
use crate::{
    config::{
        app_config::AppConfig,
        ggrs_config::{Address, GGRSConfig},
    },
    model::{
        game_state::GameState, input::Input, netplay_request::NetplayRequest,
        network_stats::NetworkStats,
    },
    session::{Session, SessionType},
};
use crate::{
    get_runtime, mark_matchbox_loop, matchbox_stop_requested, set_connected_peers,
    set_matchbox_stop, set_netplay_disconnected, stop_matchbox_loop,
};

const DESYNC_CHECK_INTERVAL: u32 = 120;
const PLAYERS_WAIT_TIMEOUT: Duration = Duration::from_secs(20);

pub struct Netplay {
    pub local_player_handle: Option<usize>,
    pub remote_player_handles: Vec<usize>,
    num_players: usize,
    spectators_handles: Vec<usize>,
    session: Option<SessionType>,
    is_test: bool,
    is_spectator: bool,
    requests: Vec<GgrsRequest<GGRSConfig>>,
    game_state: GameState,
    current_inputs: Option<Vec<Input>>,
    current_remote_players: Option<Vec<Address>>,
    test_inputs: Option<Vec<Input>>,
    pending_spectators: Vec<Uuid>,
    registered_spectators: HashSet<Uuid>,
    host_gone: bool,
}

unsafe impl Send for Netplay {}

#[allow(dead_code)]
fn assert_sockets_are_send() {
    fn is_send<T: Send>() {}

    is_send::<UdpNonBlockingSocket>();
    is_send::<matchbox_socket::WebRtcChannel>();
    is_send::<WebRtcSocket>();
}

impl Netplay {
    pub fn new(session: Option<SessionType>) -> Self {
        Self {
            local_player_handle: None,
            remote_player_handles: vec![],
            num_players: 2,
            spectators_handles: vec![],
            session,
            is_test: false,
            is_spectator: false,
            requests: vec![],
            game_state: GameState::empty(),
            current_inputs: Some(vec![]),
            current_remote_players: Some(vec![]),
            test_inputs: None,
            pending_spectators: vec![],
            registered_spectators: HashSet::new(),
            host_gone: false,
        }
    }

    pub fn mark_host_gone(&mut self) {
        self.host_gone = true;
    }

    pub fn set_test_inputs(&mut self, inputs: Vec<Input>) {
        self.test_inputs = if inputs.is_empty() {
            None
        } else {
            Some(inputs)
        };
    }

    pub fn local_player_handle(&self) -> i32 {
        match self.local_player_handle {
            Some(handle) => handle as i32,
            None => -1,
        }
    }

    pub fn remote_player_handle(&self) -> i32 {
        match self.remote_player_handles.first() {
            Some(handle) => *handle as i32,
            None => -1,
        }
    }

    pub fn remote_player_handles(&self) -> Vec<usize> {
        self.remote_player_handles.clone()
    }

    pub fn current_remote_players(&self) -> Vec<Address> {
        match self.current_remote_players {
            Some(ref players) => players.clone(),
            None => vec![],
        }
    }

    pub fn remove_remote_player(&mut self, addr: &Address) -> usize {
        if let Some(ref mut players) = self.current_remote_players {
            players.retain(|player| player != addr);

            return players.len();
        }

        0
    }

    pub fn is_a_remote_player(&self, addr: Address) -> bool {
        match self.current_remote_players {
            Some(ref players) => players.contains(&addr),
            None => false,
        }
    }

    pub fn reset(&mut self) -> Result<(), String> {
        let session_res = self.session();
        let had_session = session_res.is_some();

        if let Some(mut session) = session_res {
            if let Err(e) = session.disconnect_all(self) {
                warn!("reset : disconnect_all failed : {:?}", e);
            }
        }

        *self = Netplay::new(None);

        stop_matchbox_loop();
        set_connected_peers(HashSet::new());

        if !had_session {
            return Err("reset : No session found".to_string());
        }

        set_netplay_disconnected(true);

        Ok(())
    }

    pub fn session(&mut self) -> Option<Box<dyn Session<GGRSConfig>>> {
        let session = self.session.take();

        match (session, self.is_test) {
            (Some(SessionType::P2P(p2p)), false) => Some(Box::new(p2p)),
            (Some(SessionType::Test(test)), true) => Some(Box::new(test)),
            (Some(SessionType::Spectate(spectate)), false) => Some(Box::new(spectate)),
            _ => None,
        }
    }

    fn base_session_builder(config: &AppConfig) -> SessionBuilder<GGRSConfig> {
        let fps = config.fps();
        let scale = |frames_at_60: usize| frames_at_60 * fps / 60;

        SessionBuilder::<GGRSConfig>::new()
            .with_input_delay(config.input_delay as usize)
            .with_max_input_delay(config.max_input_delay.max(0) as usize)
            .with_max_prediction_window(scale(15))
            .with_max_rollback_window(scale(45))
            .with_fps(fps)
            .unwrap()
            .with_disconnect_timeout(Duration::from_secs(15))
            .with_max_frames_behind(scale(50))
            .unwrap()
            .with_catchup_speed(scale(4).max(1))
            .unwrap()
            .with_desync_detection_mode(DesyncDetection::On {
                interval: DESYNC_CHECK_INTERVAL * fps as u32 / 60,
            })
    }

    fn spawn_matchbox_loop<F>(
        thread_name: &str,
        mut socket: WebRtcSocket,
        future_msg: F,
        shared_players: Arc<Mutex<Vec<PlayerType<PeerId>>>>,
    ) -> Result<(), String>
    where
        F: std::future::Future<Output = Result<(), matchbox_socket::Error>> + Send + 'static,
    {
        stop_matchbox_loop();
        set_matchbox_stop(false);
        set_netplay_disconnected(false);
        mark_matchbox_loop(true);

        let spawned = std::thread::Builder::new()
            .name(thread_name.to_string())
            .spawn(move || {
                info!("Starting matchbox thread");

                get_runtime().block_on(async {
                    let loop_fut = async {
                        match future_msg.await {
                            Ok(()) => info!("Matchbox thread exited cleanly!"),
                            Err(e) => match e {
                                matchbox_socket::Error::ConnectionFailed(e) => {
                                    error!("Connection failed: {}", e);
                                }
                                matchbox_socket::Error::Disconnected(e) => {
                                    error!("Connection closed: {}", e);
                                }
                            },
                        }
                    }
                    .fuse();

                    futures::pin_mut!(loop_fut);

                    let timeout = Delay::new(Duration::from_millis(10));
                    futures::pin_mut!(timeout);

                    while !matchbox_stop_requested() {
                        socket.update_peers();
                        set_connected_peers(socket.connected_peers().map(|p| p.0).collect());

                        if let Ok(mut players) = shared_players.lock() {
                            *players = socket.players();
                        }

                        select! {
                            _ = (&mut timeout).fuse() => {
                                timeout.reset(Duration::from_millis(10));
                            }
                            _ = &mut loop_fut => {
                                info!("Matchbox message loop ended!");

                                set_matchbox_stop(true);
                                set_netplay_disconnected(true);
                            }
                        }
                    }
                });

                mark_matchbox_loop(false);
            });

        if let Err(e) = spawned {
            mark_matchbox_loop(false);

            return Err(format!("Failed to spawn {} : {}", thread_name, e));
        }

        Ok(())
    }

    fn parse_peer(id: &str) -> Result<PeerId, String> {
        Uuid::parse_str(id)
            .map(PeerId)
            .map_err(|e| format!("invalid peer id '{}' : {}", id, e))
    }

    fn seats_from_config(players_from_config: &[String]) -> Result<Vec<(usize, PeerId)>, String> {
        if players_from_config.is_empty() {
            return Err("No players in the lobby config".to_string());
        }

        players_from_config
            .iter()
            .enumerate()
            .map(|(seat, id)| Self::parse_peer(id).map(|peer_id| (seat, peer_id)))
            .collect()
    }

    fn remote_peers(players: &[PlayerType<PeerId>]) -> HashSet<PeerId> {
        players
            .iter()
            .filter_map(|player| match player {
                PlayerType::Remote(peer_id) => Some(*peer_id),
                _ => None,
            })
            .collect()
    }

    fn wait_for_players(
        shared_players: &Arc<Mutex<Vec<PlayerType<PeerId>>>>,
        required: &[PeerId],
    ) -> (HashSet<PeerId>, String) {
        let start_time = Instant::now();
        let mut connected = HashSet::new();
        let mut all_connected = false;
        let mut stopped = false;
        let mut done = false;

        while !done {
            if let Ok(players) = shared_players.try_lock() {
                connected = Self::remote_peers(&players);
            }

            all_connected = required.iter().all(|peer_id| connected.contains(peer_id));
            stopped = matchbox_stop_requested();

            done = all_connected || stopped || start_time.elapsed() >= PLAYERS_WAIT_TIMEOUT;

            if !done {
                sleep(Duration::from_millis(17));
            }
        }

        let outcome = if all_connected {
            "all connected"
        } else if stopped {
            "signaling loop ended"
        } else {
            "timed out"
        };

        (
            connected,
            format!(
                "{} after {:.1} s",
                outcome,
                start_time.elapsed().as_secs_f32()
            ),
        )
    }

    fn ensure_players_connected(
        required: &[PeerId],
        connected: &HashSet<PeerId>,
        outcome: &str,
    ) -> Result<(), String> {
        let missing: Vec<Uuid> = required
            .iter()
            .filter(|peer_id| !connected.contains(peer_id))
            .map(|peer_id| peer_id.0)
            .collect();

        if missing.is_empty() {
            return Ok(());
        }

        Err(format!(
            "Initialization failed, missing players {:?} ({})",
            missing, outcome
        ))
    }

    pub unsafe fn init(&mut self, config: AppConfig) -> Result<(), String> {
        self.is_test = config.is_test();
        self.num_players = (config.netplay.num_players as usize).max(2);

        let session = Self::base_session_builder(&config);

        let result = if config.netplay.spectator_conf.is_some() {
            self.init_spectator(session, config)
        } else if config.netplay.server_conf.is_some() {
            self.init_p2p(session, config)
        } else if config.netplay.local_conf.is_some() {
            self.init_local(config)
        } else if config.is_test() {
            self.init_test(config)
        } else {
            Err("Not suitable configuration found".to_string())
        };

        if result.is_err() {
            stop_matchbox_loop();
            set_connected_peers(HashSet::new());
        }

        result
    }

    fn init_spectator(
        &mut self,
        session: SessionBuilder<GGRSConfig>,
        mut config: AppConfig,
    ) -> Result<(), String> {
        let spectate = config.netplay.spectator_conf.take().unwrap();
        let players_from_config = config.netplay.players.clone().unwrap_or_default();
        let seats = Self::seats_from_config(&players_from_config)?;
        let host_peer = Self::parse_peer(&spectate.to_spectate.clone().unwrap_or_default())?;
        let room_url = spectate
            .room_url
            .ok_or("spectator : room url missing".to_string())?;

        let session = session
            .with_num_players(config.netplay.num_players as usize)
            .map_err(|e| e.to_string())?
            .with_catchup_speed(1)
            .map_err(|e| e.to_string())?;

        let (mut socket, future_msg) = WebRtcSocket::new_unreliable(room_url);
        let channel = socket
            .take_channel(0)
            .map_err(|e| format!("take_channel : {:?}", e))?;

        let shared_players: Arc<Mutex<Vec<PlayerType<PeerId>>>> = Arc::new(Mutex::new(vec![]));

        Self::spawn_matchbox_loop(
            "matchbox-thread-spectate",
            socket,
            future_msg,
            shared_players.clone(),
        )?;

        let required: Vec<PeerId> = seats.iter().map(|(_, peer_id)| *peer_id).collect();
        let (connected, outcome) = Self::wait_for_players(&shared_players, &required);

        Self::ensure_players_connected(&required, &connected, &outcome)?;

        for (seat, peer_id) in seats {
            self.current_remote_players
                .get_or_insert_with(Vec::new)
                .push(Address::Peer(peer_id));

            if self.local_player_handle.is_none() {
                self.local_player_handle = Some(seat);
            } else {
                self.remote_player_handles.push(seat);
            }
        }

        let sess = session.start_spectator_session(Address::Peer(host_peer), channel);

        self.session = Some(SessionType::Spectate(sess));
        self.is_spectator = true;

        Ok(())
    }

    fn init_p2p(
        &mut self,
        session: SessionBuilder<GGRSConfig>,
        mut config: AppConfig,
    ) -> Result<(), String> {
        let server = config.netplay.server_conf.take().unwrap();
        let players_from_config = config.netplay.players.clone().unwrap_or_default();
        let spectators_from_config = config.netplay.spectators.clone().unwrap_or_default();
        let local_peer_id = config.netplay.local_peer_id.clone().unwrap_or_default();
        let num_players = config.netplay.num_players as usize;
        let is_host = server.is_host;
        let allow_late_spectators = server.allow_late_spectators.unwrap_or(true);
        let room_url = server
            .room_url
            .ok_or("p2p : room url missing".to_string())?;

        let seats = Self::seats_from_config(&players_from_config)?;
        let local_seat = players_from_config
            .iter()
            .position(|p| *p == local_peer_id)
            .ok_or_else(|| {
                format!(
                    "Local peer {} is not in the lobby ordering {:?}",
                    local_peer_id, players_from_config
                )
            })?;
        let configured_spectators = spectators_from_config
            .iter()
            .map(|id| Self::parse_peer(id))
            .collect::<Result<Vec<PeerId>, String>>()?;

        let mut session = session
            .with_num_players(num_players)
            .map_err(|e| e.to_string())?;

        if is_host && allow_late_spectators {
            session = session.with_late_spectators(true);
        }

        let (mut socket, future_msg) = WebRtcSocket::new_unreliable(room_url);
        let channel = socket
            .take_channel(0)
            .map_err(|e| format!("take_channel : {:?}", e))?;

        let shared_players: Arc<Mutex<Vec<PlayerType<PeerId>>>> = Arc::new(Mutex::new(vec![]));

        Self::spawn_matchbox_loop(
            "matchbox-thread",
            socket,
            future_msg,
            shared_players.clone(),
        )?;

        let required: Vec<PeerId> = seats
            .iter()
            .filter(|(seat, _)| *seat != local_seat)
            .map(|(_, peer_id)| *peer_id)
            .collect();
        let (connected, outcome) = Self::wait_for_players(&shared_players, &required);

        Self::ensure_players_connected(&required, &connected, &outcome)?;

        for (seat, peer_id) in &seats {
            if *seat == local_seat {
                self.local_player_handle = Some(*seat);
                session = session
                    .add_player(PlayerType::Local, *seat)
                    .map_err(|e| format!("failed to add local player at seat {} : {}", seat, e))?;
            } else {
                self.current_remote_players
                    .get_or_insert_with(Vec::new)
                    .push(Address::Peer(*peer_id));
                self.remote_player_handles.push(*seat);
                session = session
                    .add_player(PlayerType::Remote(Address::Peer(*peer_id)), *seat)
                    .map_err(|e| format!("failed to add player at seat {} : {}", seat, e))?;
            }
        }

        if is_host {
            let mut handle = num_players;

            for peer_id in &configured_spectators {
                if connected.contains(peer_id) {
                    session = session
                        .add_player(PlayerType::Spectator(Address::Peer(*peer_id)), handle)
                        .map_err(|e| format!("failed to add spectator {} : {}", peer_id.0, e))?;
                    self.spectators_handles.push(handle);
                    self.registered_spectators.insert(peer_id.0);
                    handle += 1;
                } else {
                    self.queue_spectator(peer_id.0);
                }
            }

            let known: HashSet<PeerId> = required
                .iter()
                .chain(configured_spectators.iter())
                .cloned()
                .collect();

            for peer_id in connected.iter().filter(|peer_id| !known.contains(peer_id)) {
                info!(
                    "Peer {} is not in the lobby config, treating it as a late spectator",
                    peer_id.0
                );
                self.queue_spectator(peer_id.0);
            }
        } else {
            for peer_id in connected
                .iter()
                .filter(|peer_id| !required.contains(peer_id))
            {
                info!(
                    "Peer {} is not a player of this session, ignoring",
                    peer_id.0
                );
            }
        }

        let sess = session
            .start_p2p_session(channel)
            .map_err(|e| format!("failed to start session : {}", e))?;

        info!("Starting p2p session");

        self.session = Some(SessionType::P2P(sess));

        Ok(())
    }

    fn init_local(&mut self, mut config: AppConfig) -> Result<(), String> {
        let local = config.netplay.local_conf.take().unwrap();

        let remote_addr: SocketAddr = local
            .remote_addr
            .parse()
            .map_err(|e| format!("Can't parse remote addr : {}", e))?;
        let socket = UdpNonBlockingSocket::bind_to_port(local.port).unwrap();

        if local.player_draw == 0 {
            self.local_player_handle = Some(0);
            self.remote_player_handles = vec![1];
        } else {
            self.local_player_handle = Some(1);
            self.remote_player_handles = vec![0];
        }

        let session = Self::base_session_builder(&config)
            .with_num_players(2)
            .unwrap()
            .add_player(PlayerType::Local, self.local_player_handle.unwrap())
            .unwrap()
            .add_player(
                PlayerType::Remote(Address::Socket(remote_addr)),
                self.remote_player_handles[0],
            )
            .unwrap()
            .start_p2p_session(socket)
            .unwrap();

        info!("Starting local p2p session");

        self.session = Some(SessionType::P2P(session));

        Ok(())
    }

    fn init_test(&mut self, config: AppConfig) -> Result<(), String> {
        info!("Starting test session");

        let session: SyncTestSession<GGRSConfig> = SessionBuilder::new()
            .with_num_players(self.num_players)
            .unwrap()
            .with_check_distance(config.test.unwrap().check_distance as usize)
            .with_input_delay(config.input_delay as usize)
            .with_max_input_delay(config.max_input_delay.max(0) as usize)
            .start_synctest_session()
            .unwrap();

        self.local_player_handle = Some(0);
        self.remote_player_handles = (1..self.num_players).collect();

        self.session = Some(SessionType::Test(session));

        Ok(())
    }

    pub fn poll_remote(&mut self) -> Result<(), String> {
        self.drain_pending_spectators();

        let session_res = self.session();

        if let Some(mut session) = session_res {
            session.poll_remote();

            self.session = Some(session.retrieve());

            if self.host_gone {
                if let Some(SessionType::Spectate(session)) = self.session.as_ref() {
                    if session.frames_behind_host() == 0 {
                        set_netplay_disconnected(true);
                    }
                }
            }

            Ok(())
        } else {
            Err("poll_remote: No session found".to_string())
        }
    }

    pub fn is_synchronized(&mut self) -> bool {
        let session_res = self.session();

        if let Some(session) = session_res {
            let is_syncronized = session.is_synchronized();

            self.session = Some(session.retrieve());

            is_syncronized
        } else {
            false
        }
    }

    pub fn advance_frame(&mut self, input: Input) -> Result<(), String> {
        let session_res = self.session();

        if let Some(mut session) = session_res {
            if !self.is_spectator {
                if let None = self.local_player_handle {
                    return Err(format!("No local player handle"));
                }

                if self.remote_player_handles.is_empty() {
                    return Err(format!("No remote player handle"));
                }

                let local_handle = self.local_player_handle.unwrap();

                if let Err(e) = session.add_local_input(local_handle, input) {
                    return Err(format!("Couldn't added local input : {}", e));
                }

                if self.is_test {
                    for handle in (0..self.num_players).filter(|h| *h != local_handle) {
                        let seat_input = self
                            .test_inputs
                            .as_ref()
                            .and_then(|inputs| inputs.get(handle).copied())
                            .unwrap_or_default();

                        if let Err(e) = session.add_local_input(handle, seat_input) {
                            return Err(format!("Couldn't added test input : {}", e));
                        }
                    }
                }
            }

            if self.requests().is_empty() {
                match session.advance_frame() {
                    Ok(requests) => {
                        self.update_requests(requests);

                        self.session = Some(session.retrieve());

                        return Ok(());
                    }
                    Err(GgrsError::PredictionThreshold) => {
                        self.session = Some(session.retrieve());

                        return Err("PredictionThreshold".to_string());
                    }
                    Err(e) => {
                        self.session = Some(session.retrieve());

                        return Err(format!("GGRSError : {}", e.to_string()));
                    }
                };
            } else {
                self.session = Some(session.retrieve());

                return Err(
                    "Netplay request is not empty. Finish using request before advancing"
                        .to_string(),
                );
            }
        }

        Err("advance_frame: No session found".to_string())
    }

    pub fn events(&mut self) -> Vec<String> {
        let session_res = self.session();

        if let Some(mut session) = session_res {
            let events: Vec<String> = session.events(self);

            self.session = Some(session.retrieve());

            events
        } else {
            vec![]
        }
    }

    pub fn game_state(&self) -> GameState {
        self.game_state.clone()
    }

    pub unsafe fn reset_game_state(&mut self) {
        self.game_state.release();
    }

    pub fn requests(&self) -> Vec<NetplayRequest> {
        self.requests
            .iter()
            .map(|req| NetplayRequest::new(req))
            .collect()
    }

    pub fn update_requests(&mut self, requests: Vec<GgrsRequest<GGRSConfig>>) {
        self.requests = requests;
    }

    pub unsafe fn handle_save_game_state_request(
        &mut self,
        game_state: GameState,
    ) -> Result<(), String> {
        if !self.requests.is_empty() {
            let req = self.requests.first().unwrap();

            return match req {
                GgrsRequest::SaveGameState { cell, frame } => {
                    assert_eq!(self.game_state.frame(), *frame);

                    let buffer = bincode::serialize(&game_state.data()).unwrap();
                    let checksum = fletcher16(&buffer) as u128;
                    cell.save(*frame, Some(game_state.clone()), Some(checksum as u128));

                    self.game_state = game_state.clone();
                    self.game_state.update_frame(*frame);

                    self.requests.remove(0);

                    Ok(())
                }
                _ => {
                    let err = format!(
                    "The last request is not a save game state req, recheck the last request saved, was : {:#?}",self.requests()
                );
                    Err(err)
                }
            };
        }

        Err("Requests are empty".to_string())
    }

    pub fn handle_advance_frame_request(&mut self) -> Vec<Input> {
        if !self.requests.is_empty() {
            let req = self.requests.first().unwrap();

            return match req {
                GgrsRequest::AdvanceFrame { inputs } => {
                    self.game_state.add_frame();

                    let inputs: Vec<Input> = inputs
                        .iter()
                        .map(|(input, status)| {
                            return match *status {
                                InputStatus::Confirmed => *input,
                                InputStatus::Predicted => *input,
                                InputStatus::Disconnected => Input::disconnected(),
                            };
                        })
                        .collect();

                    self.requests.remove(0);

                    self.current_inputs = Some(inputs.clone());

                    inputs
                }
                _ => vec![],
            };
        }

        vec![]
    }

    pub unsafe fn handle_load_game_state_request(&mut self) -> Result<SafeBytes, String> {
        if !self.requests.is_empty() {
            let req = self.requests.first().unwrap();

            return match req {
                GgrsRequest::LoadGameState { cell, frame } => {
                    let to_load: GameState = cell
                        .load()
                        .expect("No data found when trying to load game state");
                    self.game_state = to_load;

                    self.game_state.update_frame(*frame);

                    self.requests.remove(0);

                    Ok(self.game_state.clone().data().to_safe_bytes())
                }
                _ => {
                    let err = format!(
                    "The last request is not a load game state request.The last request saved was : {:#?}",self.requests()
                );
                    Err(err)
                }
            };
        }

        Err("Requests are empty".to_string())
    }

    pub unsafe fn network_stats(
        &mut self,
        player_handle: i32,
        network_stats: *mut NetworkStats,
    ) -> Result<(), String> {
        let session_res = self.session();

        let requested = match player_handle {
            handle if handle >= 0 => Some(handle as usize),
            _ => self.remote_player_handles.first().copied(),
        };

        if let Some(mut session) = session_res {
            if let Some(remote_player_handle) = requested {
                let stats = session.net_stats(remote_player_handle);
                let str = format!("{:?}", stats);
                if let Ok(net) = stats {
                    // ggrs 0.13 removed kbps_sent, so keep the repr(C) layout for the C# side for now
                    (*network_stats) = NetworkStats::new(
                        net.send_queue_len,
                        net.ping,
                        0,
                        net.local_frames_behind,
                        net.remote_frames_behind,
                    );

                    self.session = Some(session.retrieve());

                    return Ok(());
                }
                self.session = Some(session.retrieve());

                return Err(str);
            }

            self.session = Some(session.retrieve());

            Err("No remote player handle found".to_string())
        } else {
            Err("network_stats : No session found".to_string())
        }
    }

    pub fn frames_ahead(&mut self) -> Result<i32, String> {
        let session_res = self.session();

        if let Some(mut session) = session_res {
            let frames_ahead = session.get_frames_ahead();

            self.session = Some(session.retrieve());

            Ok(frames_ahead)
        } else {
            Err("frames_ahead : No session found".to_string())
        }
    }

    /// Host only: registers a late spectator on the running P2P session
    pub fn add_spectator(&mut self, peer_id: &str) -> Result<(), String> {
        let uuid = Uuid::parse_str(peer_id)
            .map_err(|e| format!("add_spectator : invalid peer id '{}': {}", peer_id, e))?;

        if self.registered_spectators.contains(&uuid) {
            return Ok(());
        }

        if !crate::is_peer_connected(&uuid) {
            info!(
                "Late spectator {} has no WebRTC connection yet, deferring",
                uuid
            );
            self.queue_spectator(uuid);

            return Ok(());
        }

        self.add_spectator_now(uuid).map(|_| ())
    }

    fn queue_spectator(&mut self, uuid: Uuid) {
        if self.registered_spectators.contains(&uuid) || self.pending_spectators.contains(&uuid) {
            return;
        }

        self.pending_spectators.push(uuid);
    }

    fn add_spectator_now(&mut self, uuid: Uuid) -> Result<usize, String> {
        match self.session.as_mut() {
            Some(SessionType::P2P(session)) => {
                let handle = session
                    .add_spectator(Address::Peer(PeerId(uuid)))
                    .map_err(|e| e.to_string())?;
                self.spectators_handles.push(handle);
                self.registered_spectators.insert(uuid);
                Ok(handle)
            }
            Some(_) => Err("add_spectator : requires a P2P session".to_string()),
            None => Err("add_spectator : No session found".to_string()),
        }
    }

    fn drain_pending_spectators(&mut self) {
        if self.pending_spectators.is_empty() {
            return;
        }

        let ready: Vec<Uuid> = self
            .pending_spectators
            .iter()
            .filter(|uuid| crate::is_peer_connected(uuid))
            .cloned()
            .collect();

        if ready.is_empty() {
            return;
        }

        self.pending_spectators.retain(|uuid| !ready.contains(uuid));

        for uuid in ready {
            match self.add_spectator_now(uuid) {
                Ok(handle) => {
                    info!("Deferred late spectator {} added (handle {})", uuid, handle)
                }
                Err(e) => warn!(
                    "Deferred late spectator {} could not be added : {}",
                    uuid, e
                ),
            }
        }
    }

    /// Spectator only: how many confirmed frames the session still has to replay to reach the host
    pub fn frames_behind(&self) -> i32 {
        match self.session.as_ref() {
            Some(SessionType::Spectate(session)) => session.frames_behind_host() as i32,
            _ => 0,
        }
    }
}

fn fletcher16(data: &[u8]) -> u16 {
    let mut sum1: u16 = 0;
    let mut sum2: u16 = 0;

    for index in 0..data.len() {
        sum1 = (sum1 + data[index] as u16) % 255;
        sum2 = (sum2 + sum1) % 255;
    }

    (sum2 << 8) | sum1
}
