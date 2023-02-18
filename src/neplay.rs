use ggrs::{GGRSEvent, GGRSRequest, InputStatus, P2PSession, SyncTestSession};

use crate::{
    model::{input::Input, netplay_request::NetplayRequest, state::GameState},
    GGRSConfig, Status,
};

pub struct Netplay {
    session: Option<*mut P2PSession<GGRSConfig>>,
    session_test: Option<*mut SyncTestSession<GGRSConfig>>,
    requests: Vec<GGRSRequest<GGRSConfig>>,
    game_state: GameState,
    skip_frames: u32,
}

impl Netplay {
    pub const fn new(
        session: Option<*mut P2PSession<GGRSConfig>>,
        session_test: Option<*mut SyncTestSession<GGRSConfig>>,
    ) -> Self {
        Self {
            session,
            session_test,
            requests: vec![],
            game_state: GameState::new(vec![], 0, 0),
            skip_frames: 0,
        }
    }

    pub fn session(&self) -> Option<*mut P2PSession<GGRSConfig>> {
        self.session
    }

    pub unsafe fn events(&mut self) -> Vec<&'static str> {
        let session = self.session.take().unwrap();

        let mut events: Vec<&'static str> = vec![];

        for (_, event) in (*session).events().enumerate() {
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
                    self.update_skip_frames(skip_frames + 1);

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

        self.minus_skip_frames();

        self.update_session(session);

        events
    }

    pub fn session_test(&self) -> Option<*mut SyncTestSession<GGRSConfig>> {
        self.session_test
    }

    pub fn skip_frames(&self) -> u32 {
        self.skip_frames
    }

    pub fn update_skip_frames(&mut self, skip_frames: u32) {
        self.skip_frames += skip_frames;
    }

    pub fn minus_skip_frames(&mut self) {
        if self.skip_frames > 0 {
            self.skip_frames -= 1;
        }
    }

    pub fn game_state(&self) -> GameState {
        self.game_state.clone()
    }

    pub fn update_session(&mut self, session: *mut P2PSession<GGRSConfig>) {
        self.session = Some(session);
    }

    pub fn update_session_test(&mut self, session_test: *mut SyncTestSession<GGRSConfig>) {
        self.session_test = Some(session_test);
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
