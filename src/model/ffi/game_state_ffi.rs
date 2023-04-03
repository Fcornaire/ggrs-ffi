use core::slice;

use crate::model::{
    arrow::Arrow, chest::Chest, game_state::GameState, pickup::Pickup, player::Player,
};

use super::{
    arrow_ffi::ArrowFFI, chests_ffi::ChestFFI, pickup_ffi::PickupFFI, player_ffi::PlayerFFI,
    session_ffi::SessionFFI,
};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GameStateFFI {
    pub players: *mut PlayerFFI,
    pub players_len: i32,
    pub arrows: *mut ArrowFFI,
    pub arrows_len: i32,
    pub chests: *mut ChestFFI,
    pub chests_len: i32,
    pub pickups: *mut PickupFFI,
    pub pickups_len: i32,
    pub session: SessionFFI,
    pub frame: i32,
}

impl GameStateFFI {
    pub unsafe fn to_model(&self, frame: i32) -> GameState {
        //TODO: refacto
        let players_ffi = {
            let slice = slice::from_raw_parts(self.players, self.players_len.try_into().unwrap());
            Vec::from(slice)
        };

        let arrows_ffi =
            slice::from_raw_parts(self.arrows, self.arrows_len.try_into().unwrap()).to_vec();

        let chests_ffi =
            slice::from_raw_parts(self.chests, self.chests_len.try_into().unwrap()).to_vec();

        let pickups_ffi =
            slice::from_raw_parts(self.pickups, self.pickups_len.try_into().unwrap()).to_vec();

        let players: Vec<Player> = players_ffi
            .iter()
            .filter(|player_ffi| !player_ffi.is_empty_player())
            .map(|play_ffi| play_ffi.to_model())
            .collect();

        let arrows: Vec<Arrow> = arrows_ffi
            .iter()
            .filter(|arrow_ffi| !arrow_ffi.is_empty_arrow())
            .map(|arrow_ffi| arrow_ffi.to_model())
            .collect();

        let chests: Vec<Chest> = chests_ffi
            .iter()
            .filter(|chest_ffi| !chest_ffi.is_empty())
            .map(|chest_ffi| chest_ffi.to_model())
            .collect();

        let pickups: Vec<Pickup> = pickups_ffi
            .iter()
            .filter(|pickup_ffi| !pickup_ffi.is_empty())
            .map(|pickup_ffi| pickup_ffi.to_model())
            .collect();

        GameState::new(
            players.clone(),
            players.clone().len().try_into().unwrap(),
            arrows.clone(),
            arrows.clone().len().try_into().unwrap(),
            chests.clone(),
            chests.clone().len().try_into().unwrap(),
            pickups.clone(),
            pickups.clone().len().try_into().unwrap(),
            self.session,
            frame,
        )
    }

    pub unsafe fn update(&mut self, gs: GameState) {
        self.frame = gs.frame();
        let players = gs.players();
        let arrows = gs.arrows();
        let chests = gs.chests();
        let pickups = gs.pickups();

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

        if chests.len() > 0 {
            let chests_ffi =
                slice::from_raw_parts_mut(self.chests, self.chests_len.try_into().unwrap());

            for i in 0..chests.len() {
                chests_ffi[i].update(chests[i].clone());
            }
        }

        if pickups.len() > 0 {
            let pickups_ffi =
                slice::from_raw_parts_mut(self.pickups, self.pickups_len.try_into().unwrap());

            for i in 0..pickups.len() {
                pickups_ffi[i].update(pickups[i].clone());
            }
        }

        self.players_len = gs.players_len();
        self.arrows_len = gs.arrows_len();
        self.chests_len = gs.chests_len();
        self.pickups_len = gs.pickups_len();
        self.session = gs.session();
        self.frame = gs.frame();
    }
}
