use core::slice;

use crate::model::{player::Player, state::GameState};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GameStateFFI {
    pub players: *const Player,
    pub players_len: usize,
}

impl GameStateFFI {
    pub unsafe fn to_model(&self, frame: i32) -> GameState {
        let players = {
            let slice = slice::from_raw_parts(self.players, self.players_len);
            Vec::from(slice)
        };

        GameState::new(players, frame)
    }
}
