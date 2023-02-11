use ggrs::{GGRSRequest, InputStatus, P2PSession};

use crate::{
    model::{input::Input, netplay_request::NetplayRequest, state::GameState},
    GGRSConfig, Status,
};

pub struct Netplay {
    session: Option<*mut P2PSession<GGRSConfig>>,
    requests: Vec<GGRSRequest<GGRSConfig>>,
    game_state: GameState,
}

impl Netplay {
    pub const fn new(session: Option<*mut P2PSession<GGRSConfig>>) -> Self {
        Self {
            session,
            requests: vec![],
            game_state: GameState::new(vec![], 0),
        }
    }

    pub fn session(&self) -> Option<*mut P2PSession<GGRSConfig>> {
        self.session
    }

    pub fn game_state(&self) -> GameState {
        self.game_state.clone()
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

    pub fn handle_save_game_state_request(&mut self, gs: Option<GameState>) -> Status {
        if !self.requests.is_empty() {
            let req = self.requests.first().unwrap();

            return match req {
                GGRSRequest::SaveGameState { cell, frame } => {
                    assert_eq!(self.game_state.frame(), *frame);

                    if gs.is_some() {
                        let buffer = bincode::serialize(&gs).unwrap();
                        let checksum = fletcher16(&buffer) as u128;
                        let clone = gs.clone();
                        cell.save(*frame, clone, Some(checksum));
                    }

                    self.requests.remove(0);

                    Status::ok()
                }
                _ => {
                    let t = format!(
                    "The last request is not a save game state req, recheck the last request saved, was : {:#?}",self.requests()
                );
                    Status::ko(Box::leak(t.into_boxed_str()))
                }
            };
        }

        Status::ko("Requests are empty")
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

                    inputs
                }
                _ => vec![],
            };
        }

        vec![]
    }

    pub fn handle_load_game_state_request(&mut self) -> Status {
        if !self.requests.is_empty() {
            let req = self.requests.first().unwrap();

            return match req {
                GGRSRequest::LoadGameState { cell, frame: _ } => {
                    let to_load: GameState = cell.load().expect("No data found.");
                    self.game_state = to_load;

                    //TODO: Send new load GS instead of status

                    self.requests.remove(0);

                    Status::ok()
                }
                _ => {
                    let t = format!(
                    "The last request is not a load game state req, recheck the last request saved, was : {:#?}",self.requests()
                );
                    Status::ko(Box::leak(t.into_boxed_str()))
                }
            };
        }

        Status::ko("Requests are empty")
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
