use core::slice;
use std::mem::forget;

use crate::model::game_state::GameState;

use super::player_ffi::PlayerFFI;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GameStateFFI {
    pub players: *mut PlayerFFI,
    pub players_len: i32,
    pub frame: i32,
}

impl GameStateFFI {
    pub unsafe fn to_model(&self, frame: i32) -> GameState {
        let players_ffi = {
            let slice = slice::from_raw_parts(self.players, self.players_len.try_into().unwrap());
            Vec::from(slice)
        };

        let players = players_ffi
            .iter()
            .map(|play_ffi| play_ffi.to_model())
            .collect();

        GameState::new(players, self.players_len, frame)
    }

    pub unsafe fn update(&mut self, gs: GameState) {
        self.frame = gs.frame();
        let players = gs.players();

        let mut players_ffi =
            slice::from_raw_parts(self.players, self.players_len as usize).to_vec();

        players_ffi
            .iter_mut()
            .enumerate()
            .for_each(|(ind, player)| {
                player.update(players[ind].clone());
            });

        let players_ffi_ptr = players_ffi.as_mut_ptr();

        forget(players_ffi); //TODO: Manual free

        std::ptr::write(&mut self.players, players_ffi_ptr);

        self.players_len = gs.players_len();
        self.frame = gs.frame();
    }
}
