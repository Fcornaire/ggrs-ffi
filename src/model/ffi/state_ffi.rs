use core::slice;

use crate::model::{player::Player, state::GameState};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GameStateFFI {
    pub players: *mut Player,
    pub players_len: i32,
    pub frame: i32,
}

impl GameStateFFI {
    pub unsafe fn to_model(&self, frame: i32) -> GameState {
        let players = {
            let slice = slice::from_raw_parts(self.players, self.players_len.try_into().unwrap());
            Vec::from(slice)
        };

        GameState::new(players, self.players_len, frame)
    }

    pub fn update(&mut self, gs: GameState) {
        self.frame = gs.frame();
        self.players = gs.players().as_mut_ptr();
        self.players_len = gs.players_len();
    }
}
