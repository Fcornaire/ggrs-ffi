use serde::{Deserialize, Serialize};

use super::{
    boolean::Boolean, player_state::PlayerStates, scheduler::Scheduler, vector2f::Vector2f,
};

#[repr(C)]
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Player {
    position: Vector2f,
    position_counter: Vector2f,
    wall_stick_max: f32,
    speed: Vector2f,
    state: PlayerStates,
    jump_buffer_counter: f32,
    dodge_end_counter: f32,
    dodge_stall_counter: f32,
    scheduler: Scheduler,
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

    pub fn dodge_end_counter(&self) -> f32 {
        self.dodge_end_counter
    }

    pub fn dodge_stall_counter(&self) -> f32 {
        self.dodge_stall_counter
    }

    pub fn scheduler(&self) -> Scheduler {
        self.scheduler.clone()
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

    pub fn dodge_end_counter(mut self, dodge_end_counter: f32) -> PlayerBuilder {
        self.player.dodge_end_counter = dodge_end_counter;
        self
    }

    pub fn dodge_stall_counter(mut self, dodge_stall_counter: f32) -> PlayerBuilder {
        self.player.dodge_stall_counter = dodge_stall_counter;
        self
    }

    pub fn scheduler(mut self, scheduler: Scheduler) -> PlayerBuilder {
        self.player.scheduler = scheduler;
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
