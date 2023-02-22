use serde::{Deserialize, Serialize};

use crate::model::player_states::PlayerStates;

#[repr(C)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, PartialEq, Default)]
pub struct State {
    current_state: PlayerStates,
    previous_state: PlayerStates,
}

impl State {
    pub fn new(current_state: PlayerStates, previous_state: PlayerStates) -> Self {
        Self {
            current_state,
            previous_state,
        }
    }

    pub fn current_state(&self) -> PlayerStates {
        self.current_state.clone()
    }

    pub fn previous_state(&self) -> PlayerStates {
        self.previous_state.clone()
    }
}
