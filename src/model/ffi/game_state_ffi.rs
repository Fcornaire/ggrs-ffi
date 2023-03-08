use core::slice;

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
            .filter(|arrow_ffi| !arrow_ffi.is_empty_arrow())
            .map(|arrow_ffi| arrow_ffi.to_model())
            .collect();

        GameState::new(
            players,
            self.players_len,
            arrows.clone(),
            arrows.clone().len().try_into().unwrap(),
            frame,
        )
    }

    pub unsafe fn update(&mut self, gs: GameState) {
        self.frame = gs.frame();
        let players = gs.players();
        let arrows = gs.arrows();

        let players_ffi =
            slice::from_raw_parts_mut(self.players, self.players_len.try_into().unwrap());

        for i in 0..players.len() {
            players_ffi[i].update(players[i].clone());
        }

        if arrows.len() > 0 {
            let arrows_ffi =
                slice::from_raw_parts_mut(self.arrows, self.arrows_len.try_into().unwrap());

            for i in 0..arrows.len() {
                arrows_ffi[i].update(arrows[i].clone());
            }
        }

        self.players_len = gs.players_len();
        self.arrows_len = gs.arrows_len();
        self.frame = gs.frame();
    }
}
