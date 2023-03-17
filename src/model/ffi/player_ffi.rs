use core::slice;

use crate::{
    model::{
        bool_ffi::BoolFFI, dodge_slide::DodgeSlide, player::Player,
        player_arrows_inventory::PlayerArrowsInventory, scheduler::Scheduler, state::State,
        vector2f::Vector2f,
    },
    utils::{
        char_c_array_to_vec_string, copy_vec_float_to_float_array_c, copy_vec_int_to_int_array_c,
        copy_vec_string_to_char_c_array,
    },
};

use super::{dodge_slide::DodgeSlideFFI, scheduler_ffi::SchedulerFFI, state_ffi::StateFFI};

#[repr(C)]
#[derive(Clone, Debug)]
pub struct PlayerFFI {
    pub is_dead: BoolFFI,
    pub position: Vector2f,
    pub position_counter: Vector2f,
    pub facing: i32,
    pub arrows_inventory: PlayerArrowsInventory,
    pub wall_stick_max: f32,
    pub speed: Vector2f,
    pub flap_gravity: f32,
    pub can_hyper: BoolFFI,
    pub state: StateFFI,
    pub jump_buffer_counter: f32,
    pub dodge_end_counter: f32,
    pub dodge_stall_counter: f32,
    pub jump_grace_counter: f32,
    pub dodge_slide: DodgeSlideFFI,
    pub dodge_cooldown: BoolFFI,
    pub scheduler: SchedulerFFI,
    pub auto_move: i32,
    pub aiming: BoolFFI,
    pub can_var_jump: BoolFFI,
    pub on_ground: BoolFFI,
    pub duck_slip_counter: f32,
    pub index: i32,
}

impl PlayerFFI {
    pub unsafe fn to_model(&self) -> Player {
        assert!(!self.scheduler.scheduler_actions.is_null());
        assert!(!self.scheduler.scheduler_counters.is_null());
        assert!(!self.scheduler.scheduler_start_counters.is_null());

        let scheduler_actions = char_c_array_to_vec_string(
            self.scheduler.scheduler_actions,
            self.scheduler.scheduler_actions_length as usize,
        );
        let scheduler_counters = {
            let slice = slice::from_raw_parts(
                self.scheduler.scheduler_counters,
                self.scheduler.scheduler_counters_length.try_into().unwrap(),
            );
            Vec::from(slice)
        };
        let scheduler_start_counters = {
            let slice = slice::from_raw_parts(
                self.scheduler.scheduler_start_counters,
                self.scheduler
                    .scheduler_start_counters_length
                    .try_into()
                    .unwrap(),
            );
            Vec::from(slice)
        };

        Player::builder()
            .is_dead(self.is_dead)
            .position(self.position)
            .position_counter(self.position_counter)
            .facing(self.facing)
            .arrows_inventory(self.arrows_inventory)
            .wall_stick_max(self.wall_stick_max)
            .speed(self.speed)
            .flap_gravity(self.flap_gravity)
            .can_hyper(self.can_hyper)
            .state(State::new(
                self.state.current_state,
                self.state.previous_state,
            ))
            .dodge_end_counter(self.dodge_end_counter)
            .dodge_stall_counter(self.dodge_stall_counter)
            .jump_grace_counter(self.jump_grace_counter)
            .dodge_slide(DodgeSlide::new(
                self.dodge_slide.is_dodge_sliding,
                self.dodge_slide.was_dodge_sliding,
            ))
            .dodge_cooldown(self.dodge_cooldown)
            .jump_buffer_counter(self.jump_buffer_counter)
            .scheduler(Scheduler::new(
                scheduler_actions,
                scheduler_counters,
                scheduler_start_counters,
            ))
            .auto_move(self.auto_move)
            .aiming(self.aiming)
            .can_var_jump(self.can_var_jump)
            .on_ground(self.on_ground)
            .duck_slip_counter(self.duck_slip_counter)
            .index(self.index)
            .build()
    }

    pub fn update(&mut self, player: Player) {
        self.is_dead = player.is_dead();
        self.position = player.position();
        self.position_counter = player.position_counter();
        self.facing = player.facing();
        self.arrows_inventory = player.arrows_inventory();
        self.wall_stick_max = player.wall_stick_max();
        self.speed = player.speed();
        self.flap_gravity = player.flap_gravity();
        self.can_hyper = player.can_hyper();
        self.state.current_state = player.state().current_state();
        self.state.previous_state = player.state().previous_state();
        self.dodge_end_counter = player.dodge_end_counter();
        self.dodge_stall_counter = player.dodge_stall_counter();
        self.jump_grace_counter = player.jump_grace_counter();
        self.dodge_slide.is_dodge_sliding = player.dodge_slide().is_dodge_sliding();
        self.dodge_slide.was_dodge_sliding = player.dodge_slide().was_dodge_sliding();
        self.jump_buffer_counter = player.jump_buffer_counter();
        self.dodge_cooldown = player.dodge_cooldown();

        copy_vec_string_to_char_c_array(
            &player.scheduler().scheduler_actions(),
            self.scheduler.scheduler_actions,
        );
        self.scheduler.scheduler_actions_length =
            player.scheduler().scheduler_actions().len() as i32;

        copy_vec_float_to_float_array_c(
            &player.scheduler().scheduler_counters(),
            self.scheduler.scheduler_counters,
        );
        self.scheduler.scheduler_counters_length =
            player.scheduler().scheduler_counters().len() as i32;

        copy_vec_int_to_int_array_c(
            &player.scheduler().scheduler_start_counters(),
            self.scheduler.scheduler_start_counters,
        );
        self.scheduler.scheduler_start_counters_length =
            player.scheduler().scheduler_start_counters().len() as i32;

        self.auto_move = player.auto_move();
        self.aiming = player.aiming();
        self.can_var_jump = player.can_var_jump();
        self.on_ground = player.on_ground();
        self.duck_slip_counter = player.duck_slip_counter();
        self.index = player.index();
    }
}
