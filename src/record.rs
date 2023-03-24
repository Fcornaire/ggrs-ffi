use std::{fs, io::Write};

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::model::{ffi::player_draw::PlayerDraw, game_state::GameState, input::Input};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Frame {
    game_frame: i32,
    inputs: Vec<Input>,
    state: GameState,
}

impl Frame {
    pub fn new(game_frame: i32, inputs: Vec<Input>, state: GameState) -> Self {
        Self {
            game_frame,
            inputs,
            state,
        }
    }

    pub fn game_frame(&self) -> i32 {
        self.game_frame
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Record {
    frames: Vec<Frame>,
}

impl Record {
    pub fn new() -> Self {
        Self { frames: vec![] }
    }

    pub fn add_frame(
        &mut self,
        frame: i32,
        inputs: Vec<Input>,
        game_state: GameState,
        player_draw: PlayerDraw,
    ) {
        let mut inputs_swap = inputs.clone();
        let mut game_state_swap = game_state.clone();

        //Swap players index for P2 to match p1 export
        if player_draw == PlayerDraw::Player2 {
            if game_state_swap.players().len() >= 2 {
                game_state_swap.swap_players();
            } else {
                game_state_swap.update_remaining_player_index();
            }

            if inputs_swap.len() == 2 {
                inputs_swap.swap(0, 1);
            }
        }

        let frame: Frame = Frame::new(frame, inputs_swap, game_state_swap);

        self.frames.push(frame);
    }

    pub fn frames(&self) -> Vec<Frame> {
        self.frames.clone()
    }

    pub fn remove_predicted_frames(&mut self, last_confirmed_frames: i32) {
        self.frames
            .retain(|fr| fr.game_frame <= last_confirmed_frames);
    }

    pub fn export(&self) {
        let rec = serde_json::to_string_pretty(&self.frames).unwrap();

        let now = OffsetDateTime::now_utc();

        let file_name = format!(
            "{}-{}_{}{}{}.json",
            "Replay",
            now.date(),
            now.hour(),
            now.minute(),
            now.second()
        );

        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open("./Replays/".to_string() + &file_name)
            .expect("Unable to open");

        file.write_all(rec.as_bytes())
            .expect("Unable to write data");
    }
}
