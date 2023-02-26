use serde::{Deserialize, Serialize};

use super::{
    bool_ffi::BoolFFI, dodge_slide::DodgeSlide, player_arrows_Inventory::PlayerArrowsInventory,
    scheduler::Scheduler, state::State, vector2f::Vector2f,
};

#[repr(C)]
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Player {
    position: Vector2f,
    position_counter: Vector2f,
    arrows_inventory: PlayerArrowsInventory,
    wall_stick_max: f32,
    speed: Vector2f,
    can_hyper: BoolFFI,
    state: State,
    jump_buffer_counter: f32,
    dodge_end_counter: f32,
    dodge_stall_counter: f32,
    jump_grace_counter: f32,
    dodge_slide: DodgeSlide,
    dodge_cooldown: BoolFFI,
    scheduler: Scheduler,
    auto_move: i32,
    aiming: BoolFFI,
    can_var_jump: BoolFFI,
    on_ground: BoolFFI,
    duck_slip_counter: f32,
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

    pub fn arrows_inventory(&self) -> PlayerArrowsInventory {
        self.arrows_inventory
    }

    pub fn speed(&self) -> Vector2f {
        self.speed
    }

    pub fn can_hyper(&self) -> BoolFFI {
        self.can_hyper
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn jump_buffer_counter(&self) -> f32 {
        self.jump_buffer_counter
    }

    pub fn dodge_end_counter(&self) -> f32 {
        self.dodge_end_counter
    }

    pub fn jump_grace_counter(&self) -> f32 {
        self.jump_grace_counter
    }

    pub fn dodge_slide(&self) -> DodgeSlide {
        self.dodge_slide
    }

    pub fn dodge_stall_counter(&self) -> f32 {
        self.dodge_stall_counter
    }

    pub fn dodge_cooldown(&self) -> BoolFFI {
        self.dodge_cooldown
    }

    pub fn scheduler(&self) -> Scheduler {
        self.scheduler.clone()
    }

    pub fn auto_move(&self) -> i32 {
        self.auto_move
    }

    pub fn aiming(&self) -> BoolFFI {
        self.aiming
    }

    pub fn can_var_jump(&self) -> BoolFFI {
        self.can_var_jump
    }

    pub fn on_ground(&self) -> BoolFFI {
        self.on_ground
    }

    pub fn duck_slip_counter(&self) -> f32 {
        self.duck_slip_counter
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

    pub fn arrows_inventory(mut self, arrows_inventory: PlayerArrowsInventory) -> PlayerBuilder {
        self.player.arrows_inventory = arrows_inventory;
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

    pub fn can_hyper(mut self, can_hyper: BoolFFI) -> PlayerBuilder {
        self.player.can_hyper = can_hyper;
        self
    }

    pub fn state(mut self, state: State) -> PlayerBuilder {
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

    pub fn jump_grace_counter(mut self, jump_grace_counter: f32) -> PlayerBuilder {
        self.player.jump_grace_counter = jump_grace_counter;
        self
    }

    pub fn dodge_slide(mut self, dodge_slide: DodgeSlide) -> PlayerBuilder {
        self.player.dodge_slide = dodge_slide;
        self
    }

    pub fn dodge_stall_counter(mut self, dodge_stall_counter: f32) -> PlayerBuilder {
        self.player.dodge_stall_counter = dodge_stall_counter;
        self
    }

    pub fn dodge_cooldown(mut self, dodge_cooldown: BoolFFI) -> PlayerBuilder {
        self.player.dodge_cooldown = dodge_cooldown;
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

    pub fn aiming(mut self, aiming: BoolFFI) -> PlayerBuilder {
        self.player.aiming = aiming;
        self
    }

    pub fn can_var_jump(mut self, can_var_jump: BoolFFI) -> PlayerBuilder {
        self.player.can_var_jump = can_var_jump;
        self
    }

    pub fn on_ground(mut self, on_ground: BoolFFI) -> PlayerBuilder {
        self.player.on_ground = on_ground;
        self
    }

    pub fn duck_slip_counter(mut self, duck_slip_counter: f32) -> PlayerBuilder {
        self.player.duck_slip_counter = duck_slip_counter;
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
