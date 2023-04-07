use serde::{Deserialize, Serialize};

use crate::core::unmanaged::{safe_bytes::SafeBytes, unmanaged_bytes::UnmanagedBytes};

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct GameState {
    // players: Vec<Player>,
    // players_len: i32,
    // arrows: Vec<Arrow>,
    // arrows_len: i32,
    // chests: Vec<Chest>,
    // chests_len: i32,
    // pickups: Vec<Pickup>,
    // pickups_len: i32,
    // session: SessionFFI,
    data: UnmanagedBytes,
    frame: i32,
}

impl GameState {
    // pub const fn new(
    //     players: Vec<Player>,
    //     players_len: i32,
    //     arrows: Vec<Arrow>,
    //     arrows_len: i32,
    //     chests: Vec<Chest>,
    //     chests_len: i32,
    //     pickups: Vec<Pickup>,
    //     pickups_len: i32,
    //     session: SessionFFI,
    //     frame: i32,
    // ) -> Self {
    //     Self {
    //         players,
    //         players_len,
    //         arrows,
    //         arrows_len,
    //         session,
    //         chests,
    //         chests_len,
    //         pickups,
    //         pickups_len,
    //         frame,
    //     }
    // }

    pub fn new(safe_bytes: SafeBytes) -> Self {
        Self {
            data: UnmanagedBytes::new(safe_bytes),
            frame: 0,
        }
    }

    pub fn empty() -> Self {
        Self {
            data: UnmanagedBytes::empty(),
            frame: 0,
        }
    }

    pub fn data(&self) -> UnmanagedBytes {
        self.data.clone()
    }

    pub unsafe fn release(&mut self) {
        // self.data.to_safe_bytes().release();
        self.data = UnmanagedBytes::empty();
    }

    pub fn frame(&self) -> i32 {
        self.frame
    }

    pub fn update_frame(&mut self, frame: i32) {
        self.frame = frame;
    }

    // pub fn players(&self) -> Vec<Player> {
    //     self.players.clone()
    // }

    // pub fn players_len(&self) -> i32 {
    //     self.players_len
    // }

    // pub fn arrows(&self) -> Vec<Arrow> {
    //     self.arrows.clone()
    // }

    // pub fn arrows_len(&self) -> i32 {
    //     self.arrows_len
    // }

    // pub fn chests(&self) -> Vec<Chest> {
    //     self.chests.clone()
    // }

    // pub fn chests_len(&self) -> i32 {
    //     self.chests_len
    // }

    // pub fn pickups(&self) -> Vec<Pickup> {
    //     self.pickups.clone()
    // }

    // pub fn pickups_len(&self) -> i32 {
    //     self.pickups_len
    // }

    // pub fn session(&self) -> SessionFFI {
    //     self.session.clone()
    // }

    pub fn add_frame(&mut self) {
        self.frame += 1;

        // let mut file = fs::OpenOptions::new()
        //     .write(true)
        //     .append(true)
        //     .create(true)
        //     .open("add.txt")
        //     .expect("Unable to open");

        // file.write_all(self.frame.to_string().as_bytes())
        //     .expect("Unable to write data");
    }

    // pub fn swap_players(&mut self) {
    //     self.players.swap(0, 1);

    //     self.players[0].update_index(0);
    //     self.players[1].update_index(1);
    // }

    // pub fn update_remaining_player_index(&mut self) {
    //     self.players[0].update_index(0);
    // }
}
