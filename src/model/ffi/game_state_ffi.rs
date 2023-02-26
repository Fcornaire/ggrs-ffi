use core::slice;
use std::{fs, io::Write, mem::forget};

use crate::model::{arrow::Arrow, game_state::GameState};

use super::{arrow_ffi::ArrowFFI, player_ffi::PlayerFFI};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GameStateFFI {
    pub players: *mut PlayerFFI,
    pub players_len: i32,
    pub arrows: *mut ArrowFFI,
    pub arrows_len: i32,
    pub frame: i32,
}

impl GameStateFFI {
    pub unsafe fn to_model(&self, frame: i32) -> GameState {
        let players_ffi = {
            let slice = slice::from_raw_parts(self.players, self.players_len.try_into().unwrap());
            Vec::from(slice)
        };

        let arrows_ffi =
            slice::from_raw_parts(self.arrows, self.arrows_len.try_into().unwrap()).to_vec();

        let players = players_ffi
            .iter()
            .map(|play_ffi| play_ffi.to_model())
            .collect();

        let arrows: Vec<Arrow> = arrows_ffi
            .iter()
            .map(|arrow_ffi| arrow_ffi.to_model())
            .collect();

        GameState::new(players, self.players_len, arrows, self.arrows_len, frame)
    }

    pub unsafe fn update(&mut self, gs: GameState) {
        self.frame = gs.frame();
        let players = gs.players();
        let arrows = gs.arrows();

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

        if arrows.len() > 0 {
            let mut arrows_ffi =
                slice::from_raw_parts(self.arrows, self.arrows_len as usize).to_vec();

            arrows_ffi.iter_mut().enumerate().for_each(|(ind, arrow)| {
                arrow.update(arrows[ind].clone());
            });

            let arrows_ffi_ptr = arrows_ffi.as_mut_ptr();
            forget(arrows_ffi_ptr); //TODO: Manual free

            std::ptr::write(&mut self.arrows, arrows_ffi_ptr);
        }

        self.players_len = gs.players_len();
        self.arrows_len = gs.arrows_len();
        self.frame = gs.frame();
    }
}
