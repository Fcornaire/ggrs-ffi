use serde::{Deserialize, Serialize};

use super::player::Player;

#[derive(Clone, Serialize, Deserialize)]
pub struct GameState {
    players: Vec<Player>,
    players_len: i32,
    frame: i32,
}

impl GameState {
    pub const fn new(players: Vec<Player>, players_len: i32, frame: i32) -> Self {
        Self {
            players,
            players_len,
            frame,
        }
    }

    pub fn frame(&self) -> i32 {
        self.frame
    }

    pub fn players(&self) -> Vec<Player> {
        self.players.clone()
    }

    pub fn players_len(&self) -> i32 {
        self.players_len
    }

    pub fn add_frame(&mut self) {
        self.frame += 1;
    }
}
