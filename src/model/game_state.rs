use serde::{Deserialize, Serialize};

use super::{
    arrow::Arrow, chest::Chest, ffi::session_ffi::SessionFFI, pickup::Pickup, player::Player,
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GameState {
    players: Vec<Player>,
    players_len: i32,
    arrows: Vec<Arrow>,
    arrows_len: i32,
    chests: Vec<Chest>,
    chests_len: i32,
    pickups: Vec<Pickup>,
    pickups_len: i32,
    session: SessionFFI,
    frame: i32,
}

impl GameState {
    pub const fn new(
        players: Vec<Player>,
        players_len: i32,
        arrows: Vec<Arrow>,
        arrows_len: i32,
        chests: Vec<Chest>,
        chests_len: i32,
        pickups: Vec<Pickup>,
        pickups_len: i32,
        session: SessionFFI,
        frame: i32,
    ) -> Self {
        Self {
            players,
            players_len,
            arrows,
            arrows_len,
            session,
            chests,
            chests_len,
            pickups,
            pickups_len,
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

    pub fn chests(&self) -> Vec<Chest> {
        self.chests.clone()
    }

    pub fn chests_len(&self) -> i32 {
        self.chests_len
    }

    pub fn pickups(&self) -> Vec<Pickup> {
        self.pickups.clone()
    }

    pub fn pickups_len(&self) -> i32 {
        self.pickups_len
    }

    pub fn session(&self) -> SessionFFI {
        self.session.clone()
    }

    pub fn add_frame(&mut self) {
        self.frame += 1;
    }

    pub fn swap_players(&mut self) {
        self.players.swap(0, 1);

        self.players[0].update_index(0);
        self.players[1].update_index(1);
    }

    pub fn update_remaining_player_index(&mut self) {
        self.players[0].update_index(0);
    }
}
