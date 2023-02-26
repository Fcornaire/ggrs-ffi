use serde::{Deserialize, Serialize};

use super::{arrow::Arrow, player::Player};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GameState {
    players: Vec<Player>,
    players_len: i32,
    arrows: Vec<Arrow>,
    arrows_len: i32,
    frame: i32,
}

impl GameState {
    pub const fn new(
        players: Vec<Player>,
        players_len: i32,
        arrows: Vec<Arrow>,
        arrows_len: i32,
        frame: i32,
    ) -> Self {
        Self {
            players,
            players_len,
            arrows,
            arrows_len,
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

    pub fn arrows(&self) -> Vec<Arrow> {
        self.arrows.clone()
    }

    pub fn arrows_len(&self) -> i32 {
        self.arrows_len
    }

    pub fn add_frame(&mut self) {
        self.frame += 1;
    }
}
