use serde::{Deserialize, Serialize};

use super::{boolean::Boolean, player_state::PlayerStates, vector2f::Vector2f};

#[repr(C)]
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Player {
    position: Vector2f,
    position_counter: Vector2f,
    wall_stick_max: f32,
    speed: Vector2f,
    state: PlayerStates,
    jump_buffer_counter: f32,
    scheduler_actions: Vec<String>,
    scheduler_counters: Vec<f32>,
    scheduler_start_counters: Vec<i32>,
    auto_move: i32,
    aiming: Boolean,
    index: i32,
}

impl Player {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn builder() -> PlayerBuilder {
        PlayerBuilder::default()
    }

    pub fn position(&self) -> Vector2f {
        self.position
    }

    pub fn wall_stick_max(&self) -> f32 {
        self.wall_stick_max
    }

    pub fn position_counter(&self) -> Vector2f {
        self.position_counter
    }

    pub fn speed(&self) -> Vector2f {
        self.speed
    }

    pub fn state(&self) -> PlayerStates {
        self.state
    }

    pub fn jump_buffer_counter(&self) -> f32 {
        self.jump_buffer_counter
    }

    pub fn scheduler_counters(&self) -> Vec<f32> {
        self.scheduler_counters.clone()
    }

    pub fn scheduler_start_counters(&self) -> Vec<i32> {
        self.scheduler_start_counters.clone()
    }

    pub fn scheduler_actions(&self) -> Vec<String> {
        self.scheduler_actions.clone()
    }

    pub fn auto_move(&self) -> i32 {
        self.auto_move
    }

    pub fn aiming(&self) -> Boolean {
        self.aiming
    }

    pub fn index(&self) -> i32 {
        self.index
    }
}

#[derive(Default)]
pub struct PlayerBuilder {
    player: Player,
}

impl PlayerBuilder {
    pub fn new() -> PlayerBuilder {
        PlayerBuilder {
            player: Player::new(),
        }
    }

    pub fn position(mut self, position: Vector2f) -> PlayerBuilder {
        self.player.position = position;
        self
    }

    pub fn position_counter(mut self, position_counter: Vector2f) -> PlayerBuilder {
        self.player.position_counter = position_counter;
        self
    }

    pub fn wall_stick_max(mut self, wall_stick_max: f32) -> PlayerBuilder {
        self.player.wall_stick_max = wall_stick_max;
        self
    }

    pub fn speed(mut self, speed: Vector2f) -> PlayerBuilder {
        self.player.speed = speed;
        self
    }

    pub fn state(mut self, state: PlayerStates) -> PlayerBuilder {
        self.player.state = state;
        self
    }

    pub fn jump_buffer_counter(mut self, jump_buffer_counter: f32) -> PlayerBuilder {
        self.player.jump_buffer_counter = jump_buffer_counter;
        self
    }

    pub fn scheduler_actions(mut self, scheduler_actions: Vec<String>) -> PlayerBuilder {
        self.player.scheduler_actions = scheduler_actions;
        self
    }

    pub fn scheduler_counters(mut self, scheduler_counters: Vec<f32>) -> PlayerBuilder {
        self.player.scheduler_counters = scheduler_counters;
        self
    }

    pub fn scheduler_start_counters(mut self, scheduler_start_counters: Vec<i32>) -> PlayerBuilder {
        self.player.scheduler_start_counters = scheduler_start_counters;
        self
    }

    pub fn auto_move(mut self, auto_move: i32) -> PlayerBuilder {
        self.player.auto_move = auto_move;
        self
    }

    pub fn aiming(mut self, aiming: Boolean) -> PlayerBuilder {
        self.player.aiming = aiming;
        self
    }

    pub fn index(mut self, index: i32) -> PlayerBuilder {
        self.player.index = index;
        self
    }

    pub fn build(self) -> Player {
        self.player
    }
}
