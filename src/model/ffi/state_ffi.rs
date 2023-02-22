use crate::model::player_states::PlayerStates;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub struct StateFFI {
    pub current_state: PlayerStates,
    pub previous_state: PlayerStates,
}
