use serde::{Deserialize, Serialize};

use super::player::Player;

#[derive(Clone, Serialize, Deserialize)]
pub struct GameState {
    players: Vec<Player>,
    frame: i32,
}

impl GameState {
    pub const fn new(players: Vec<Player>, frame: i32) -> Self {
        Self { players, frame }
    }

    pub fn frame(&self) -> i32 {
        self.frame
    }

    pub fn add_frame(&mut self) {
        self.frame += 1;
    }
}
