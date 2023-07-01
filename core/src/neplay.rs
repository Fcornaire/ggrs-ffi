use flate2::read::GzDecoder;
use futures::{select, FutureExt};
use futures_timer::Delay;
use matchbox_socket::{PeerId, PeerState, WebRtcSocket};
use serde_json::Value;
use std::fs;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::runtime::Runtime;

use ggrs::{
    DesyncDetection, GGRSError, GGRSRequest, InputStatus, PlayerType, SessionBuilder,
    SyncTestSession, UdpNonBlockingSocket,
};

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
    SHOULD_STOP_MATCHBOX_FUTURE,
};

pub struct Netplay {
    pub local_player_handle: Option<usize>,
    pub remote_player_handle: Option<usize>,
    session: Option<SessionType>,
    is_test: bool,
    requests: Vec<GGRSRequest<GGRSConfig>>,
    game_state: GameState,
    current_inputs: Option<Vec<Input>>,
    // logger: TcpStream,
}

impl Netplay {
    pub fn new(session: Option<SessionType>) -> Self {
        Self {
            local_player_handle: None,
            remote_player_handle: None,
            session,
            is_test: false,
            requests: vec![],
            game_state: GameState::empty(),
            current_inputs: Some(vec![]),
            // logger: TcpStream::connect("127.0.0.1:8080").expect("Failed to connect to logger"),
        }
    }

    pub fn local_player_handle(&self) -> i32 {
        match self.local_player_handle {
            Some(handle) => handle as i32,
            None => -1,
        }
    }

    pub fn remote_player_handle(&self) -> i32 {
        match self.remote_player_handle {
            Some(handle) => handle as i32,
            None => -1,
        }
    }

    pub fn reset(&mut self) -> Result<(), String> {
        let mut session = self.session();

        session.disconnect_all(self).unwrap();

        self.local_player_handle = None;
        self.remote_player_handle = None;
        self.requests.clear();
        self.game_state = GameState::empty();
        self.current_inputs = Some(vec![]);
        self.is_test = false;
        self.session = None;

        let mut stp = SHOULD_STOP_MATCHBOX_FUTURE.lock().unwrap();
        *stp = true;

        Ok(())
    }

    // pub fn logger(&mut self) -> &mut TcpStream {
    //     &mut self.logger
    // }

    pub fn session(&mut self) -> Box<dyn Session<GGRSConfig>> {
        let session = self.session.take();

        match (session, self.is_test) {
            (Some(SessionType::P2P(p2p)), false) => Box::new(p2p),
            (Some(SessionType::Test(test)), true) => Box::new(test),
            _ => panic!("Wrong call!"),
        }
    }

    pub unsafe fn init(&mut self, config: AppConfig) -> Result<(), String> {
        let mut session = SessionBuilder::<GGRSConfig>::new()
            .with_num_players(2)
            .with_input_delay(config.input_delay as usize)
            .with_disconnect_timeout(Duration::from_secs(5));

        self.is_test = config.is_test();

        if let Some(server) = config.netplay.server {
            let (mut socket, future_msg) = WebRtcSocket::new_ggrs(server.room_url.unwrap());

            let channel = socket.take_channel(0).unwrap();

            {
                let mut stp = SHOULD_STOP_MATCHBOX_FUTURE.lock().unwrap();
                *stp = false;
            }

            let shared_players: Arc<Mutex<Vec<PlayerType<PeerId>>>> = Arc::new(Mutex::new(vec![]));
            let clone_for_thread = shared_players.clone();

            std::thread::spawn(move || {
                let rt = Runtime::new().unwrap();

                rt.block_on(async {
                    let loop_fut = future_msg.fuse();
                    futures::pin_mut!(loop_fut);

                    let timeout = Delay::new(Duration::from_millis(5));
                    futures::pin_mut!(timeout);

                    let mut should_stop = false;

                    while !should_stop {
                        {
                            match SHOULD_STOP_MATCHBOX_FUTURE.try_lock() {
                                Ok(stp) => {
                                    should_stop = *stp;
                                }
                                Err(_) => {}
                            }
                        }

                        for (_, state) in socket.update_peers() {
                            match state {
                                PeerState::Connected => {}
                                PeerState::Disconnected => {
                                    let mut can_go = false;

                                    while !can_go {
                                        match SHOULD_STOP_MATCHBOX_FUTURE.try_lock() {
                                            Ok(mut stp) => {
                                                *stp = true;
                                                can_go = true;
                                            }
                                            Err(_) => {}
                                        }
                                    }

                                    break;
                                }
                            }
                        }

                        match clone_for_thread.lock() {
                            Ok(mut players) => {
                                *players = socket.players();
                            }
                            Err(_) => {}
                        }

                        select! {
                            // Restart this loop every 10ms
                            _ = (&mut timeout).fuse() => {
                                timeout.reset(Duration::from_millis(10));
                            }

                            // Or break if the message loop ends (disconnected, closed, etc.)
                            _ = &mut loop_fut => {
                                let mut can_go = false;

                                    while !can_go {
                                        match SHOULD_STOP_MATCHBOX_FUTURE.try_lock() {
                                            Ok(mut stp) => {
                                                *stp = true;
                                                can_go = true;
                                            }
                                            Err(_) => {}
                                        }
                                    }
                                break;
                            }
                        }
                    }
                });
            });

            let mut players = vec![];
            let clone = shared_players.clone();

            while players.len() < 2 {
                match clone.try_lock() {
                    Ok(pl) => {
                        players = pl.clone();
                    }
                    Err(_) => {}
                }

                std::thread::sleep(Duration::from_millis(60));
            }

            for (i, player) in players.into_iter().enumerate() {
                let mut pl: PlayerType<Address> = PlayerType::Local;

                match player {
                    PlayerType::Local => {
                        self.local_player_handle = Some(i);
                        pl = PlayerType::Local;
                    }
                    PlayerType::Remote(peer_id) => {
                        self.remote_player_handle = Some(i);
                        pl = PlayerType::Remote(Address::Peer(peer_id));
                    }
                    _ => {}
                }

                session = session.add_player(pl, i).expect("failed to add player");
            }

            let sess = session
                .start_p2p_session(channel)
                .expect("failed to start session");

            self.session = Some(SessionType::P2P(sess));
            return Ok(());
        }

        if let Some(local) = config.netplay.local {
            match local.remote_addr.parse::<SocketAddr>() {
                Ok(socket) => {
                    let remote_addr: SocketAddr = socket;
                    let local_port = local.port;
                    let socket = UdpNonBlockingSocket::bind_to_port(local_port).unwrap();

                    if local.player_draw == 0 {
                        self.local_player_handle = Some(0);
                        self.remote_player_handle = Some(1);
                    } else {
                        self.local_player_handle = Some(1);
                        self.remote_player_handle = Some(0);
                    }

                    let session = SessionBuilder::<GGRSConfig>::new()
                        .with_num_players(2)
                        .add_player(PlayerType::Local, self.local_player_handle.unwrap())
                        .unwrap()
                        .add_player(
                            PlayerType::Remote(Address::Socket(remote_addr)),
                            self.remote_player_handle.unwrap(),
                        )
                        .unwrap()
                        .with_input_delay(config.input_delay as usize)
                        .with_disconnect_timeout(Duration::from_secs(5))
                        .with_desync_detection_mode(DesyncDetection::On { interval: 500 })
                        .start_p2p_session(socket)
                        .unwrap();

                    self.session = Some(SessionType::P2P(session));
                    return Ok(());
                }
                Err(e) => return Err(format!("Can't parse remote addr : {}", e)),
            }
        }

        if config.is_test() {
            let session: SyncTestSession<GGRSConfig> = SessionBuilder::new()
                .with_num_players(2)
                .with_check_distance(config.test.unwrap().check_distance as usize)
                .with_input_delay(config.input_delay as usize)
                .start_synctest_session()
                .unwrap();

            self.local_player_handle = Some(0);
            self.remote_player_handle = Some(1);

            self.session = Some(SessionType::Test(session));
            return Ok(());
        }

        Err("Not suitable configuration (Test, local or matchbox server) found".to_string())
    }

    pub fn poll_remote(&mut self) -> Result<(), String> {
        let mut session = self.session();

        session.poll_remote();

        self.session = Some(session.retrieve());

        Ok(())
    }

    pub fn advance_frame(&mut self, input: Input) -> Result<(), String> {
        let mut session = self.session();

        if let None = self.local_player_handle {
            return Err(format!("No local player handle"));
        }

        if let None = self.remote_player_handle {
            return Err(format!("No remote player handle"));
        }

        if let Err(e) = session.add_local_input(self.local_player_handle.unwrap(), input) {
            return Err(format!("Couldn't added local input : {}", e));
        }

        if self.is_test {
            let res: Result<(), GGRSError>;

            // let mut rng = rand::thread_rng();

            // let rand = rng.gen_range(0..10);

            // if rand % 2 == 0 {
            if self.game_state.frame() % 120 > 60 {
                res = session.add_local_input(self.remote_player_handle.unwrap(), Input::default());
            } else {
                res = session.add_local_input(self.remote_player_handle.unwrap(), Input::default());
                //we don't care on test mode
            }

            if let Err(e) = res {
                return Err(format!("Couldn't added test input : {}", e));
            }
        }

        if self.requests().is_empty() {
            match session.advance_frame() {
                Ok(requests) => {
                    self.update_requests(requests);

                    self.session = Some(session.retrieve());

                    return Ok(());
                }
                Err(GGRSError::PredictionThreshold) => {
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
                "Netplay request is not empty. Finish using request before advancing".to_string(),
            );
        }
    }

    pub fn events(&mut self) -> Vec<&'static str> {
        let mut session = self.session();
        let events: Vec<&'static str> = session.events(self);

        self.session = Some(session.retrieve());

        events
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

    pub fn update_requests(&mut self, requests: Vec<GGRSRequest<GGRSConfig>>) {
        self.requests = requests;
    }

    pub unsafe fn handle_save_game_state_request(
        &mut self,
        game_state: GameState,
    ) -> Result<(), String> {
        if !self.requests.is_empty() {
            let req = self.requests.first().unwrap();

            return match req {
                GGRSRequest::SaveGameState { cell, frame } => {
                    assert_eq!(self.game_state.frame(), *frame);

                    let buffer = bincode::serialize(&game_state.data()).unwrap();
                    let checksum = fletcher16(&buffer) as u128;
                    cell.save(*frame, Some(game_state.clone()), Some(checksum as u128));

                    // self.logger
                    //     .write(
                    //         &format!("Rust: checksum at frame {frame} : {}", checksum.to_string())
                    //             .as_bytes(),
                    //     )
                    //     .unwrap();

                    self.game_state = game_state.clone();
                    self.game_state.update_frame(*frame);

                    self.requests.remove(0);

                    if self.is_test {
                        self.debug();
                    }

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

    fn debug(&self) {
        let data = self.game_state.clone().data().bytes();
        let mut gz = GzDecoder::new(&*data);
        let mut s = String::new();
        gz.read_to_string(&mut s).unwrap();

        //Mostly for debug purpose, need refacto
        let v: Value = serde_json::from_str(&s).unwrap();
        let gs = serde_json::to_string_pretty(&v).unwrap();

        let mut file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open("gs.json")
            .expect("Unable to open");

        file.write_all(gs.as_bytes()).expect("Unable to write data");
    }

    pub fn handle_advance_frame_request(&mut self) -> Vec<Input> {
        if !self.requests.is_empty() {
            let req = self.requests.first().unwrap();

            return match req {
                GGRSRequest::AdvanceFrame { inputs } => {
                    self.game_state.add_frame();

                    let inputs: Vec<Input> = inputs
                        .iter()
                        .map(|(input, status)| {
                            return match *status {
                                InputStatus::Confirmed => *input,
                                InputStatus::Predicted => *input,
                                InputStatus::Disconnected => Input::default(),
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

    pub unsafe fn handle_load_game_state_request(&mut self) -> Result<GameState, String> {
        if !self.requests.is_empty() {
            let req = self.requests.first().unwrap();

            return match req {
                GGRSRequest::LoadGameState { cell, frame } => {
                    let to_load: GameState = cell
                        .load()
                        .expect("No data found when trying to load game state");
                    self.game_state = to_load;

                    self.game_state.update_frame(*frame);

                    self.requests.remove(0);

                    Ok(self.game_state.clone())
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

    pub unsafe fn network_stats(&mut self, network_stats: *mut NetworkStats) -> Result<(), String> {
        let mut session = self.session();

        if let Some(remote_player_handle) = self.remote_player_handle {
            let stats = session.net_stats(remote_player_handle);
            let str = format!("{:?}", stats);
            if let Ok(net) = stats {
                (*network_stats) = NetworkStats::new(
                    net.send_queue_len,
                    net.ping,
                    net.kbps_sent,
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
    }

    pub fn frames_ahead(&mut self) -> Result<i32, String> {
        let mut session = self.session();

        let frames_ahead = session.get_frames_ahead();

        self.session = Some(session.retrieve());

        Ok(frames_ahead)
    }
}

unsafe impl Send for Netplay {}

fn fletcher16(data: &[u8]) -> u16 {
    let mut sum1: u16 = 0;
    let mut sum2: u16 = 0;

    for index in 0..data.len() {
        sum1 = (sum1 + data[index] as u16) % 255;
        sum2 = (sum2 + sum1) % 255;
    }

    (sum2 << 8) | sum1
}
