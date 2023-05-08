use flate2::read::GzDecoder;
use rand::Rng;
use serde_json::Value;
use std::fs;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::time::Duration;

use ggrs::{
    GGRSError, GGRSRequest, InputStatus, PlayerType, SessionBuilder, SyncTestSession,
    UdpNonBlockingSocket,
};

use crate::model::player_draw::PlayerDraw;
use crate::{
    model::{
        ffi::config_ffi::ConfigFFI, game_state::GameState, input::Input,
        netplay_request::NetplayRequest, network_stats::NetworkStats,
    },
    session::{Session, SessionType},
    GGRSConfig,
};

pub struct Netplay {
    session: Option<SessionType>,
    is_test: bool,
    player_draw: PlayerDraw,
    requests: Vec<GGRSRequest<GGRSConfig>>,
    game_state: GameState,
    current_inputs: Option<Vec<Input>>,
}

impl Netplay {
    pub fn new(session: Option<SessionType>) -> Self {
        Self {
            session,
            is_test: false,
            player_draw: PlayerDraw::Unkown,
            requests: vec![],
            game_state: GameState::empty(),
            current_inputs: Some(vec![]),
        }
    }

    pub fn session(&mut self) -> Box<dyn Session> {
        let session = self.session.take();

        match (session, self.is_test) {
            (Some(SessionType::P2P(p2p)), false) => Box::new(p2p),
            (Some(SessionType::Test(test)), true) => Box::new(test),
            _ => panic!("Wrong call!"),
        }
    }

    pub unsafe fn init(&mut self, config_ffi: *mut ConfigFFI) -> Result<(), String> {
        let config = (*config_ffi).to_model();

        match config.remote_addr().parse::<SocketAddr>() {
            Ok(socket) => {
                let remote_addr: SocketAddr = socket;
                let local_port = config.port();
                let socket = UdpNonBlockingSocket::bind_to_port(local_port).unwrap();

                self.is_test = config.is_test_mode();
                self.player_draw = config.player_draw();

                if !config.is_test_mode() {
                    let session = SessionBuilder::<GGRSConfig>::new()
                        .with_num_players(2)
                        .add_player(PlayerType::Local, 0)
                        .unwrap()
                        .add_player(PlayerType::Remote(remote_addr), 1)
                        .unwrap()
                        .with_input_delay(config.input_delay() as usize)
                        .with_disconnect_timeout(Duration::from_secs(15))
                        .start_p2p_session(socket)
                        .unwrap();

                    self.session = Some(SessionType::P2P(session));
                    Ok(())
                } else {
                    let session: SyncTestSession<GGRSConfig> = SessionBuilder::new()
                        .with_num_players(2)
                        .with_check_distance(config.test_check_distance() as usize)
                        .with_input_delay(config.input_delay() as usize)
                        .start_synctest_session()
                        .unwrap();

                    self.session = Some(SessionType::Test(session));
                    Ok(())
                }
            }
            Err(e) => Err(format!("Can't parse remote addr : {}", e)),
        }
    }

    pub fn poll_remote(&mut self) -> Result<(), String> {
        let mut session = self.session();

        session.poll_remote();

        self.session = Some(session.retrieve());

        Ok(())
    }

    pub fn advance_frame(&mut self, input: Input) -> Result<(), String> {
        let mut session = self.session();

        if let Err(e) = session.add_local_input(0, input) {
            return Err(format!("Couldn't added local input : {}", e));
        }

        if self.is_test {
            let res: Result<(), GGRSError>;

            // let mut rng = rand::thread_rng();

            // let rand = rng.gen_range(0..10);

            // if rand % 2 == 0 {
            if self.game_state.frame() % 120 > 60 {
                res = session.add_local_input(1, Input::default());
            } else {
                res = session.add_local_input(1, Input::default()); //we don't care on test mode
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
                    cell.save(*frame, Some(game_state.clone()), Some(checksum));

                    self.game_state = game_state.clone();
                    self.game_state.update_frame(*frame);

                    self.requests.remove(0);

                    let inputs = match self.current_inputs.take() {
                        Some(inp) => inp,
                        None => vec![],
                    };

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

        let stats = session.net_stats(1);
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

        Err(str)
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
