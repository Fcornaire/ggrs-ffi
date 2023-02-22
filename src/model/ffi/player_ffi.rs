use core::slice;
use std::mem::forget;

use crate::{
    model::{
        boolean::Boolean, player::Player, player_state::PlayerStates, scheduler::Scheduler,
        vector2f::Vector2f,
    },
    utils::{
        char_c_array_to_vec_string, copy_vec_float_to_float_array_c, copy_vec_int_to_int_array_c,
        copy_vec_string_to_char_c_array,
    },
};

use super::scheduler_ffi::SchedulerFFI;

#[repr(C)]
#[derive(Clone, Debug)]
pub struct PlayerFFI {
    pub position: Vector2f,
    pub position_counter: Vector2f,
    pub wall_stick_max: f32,
    pub speed: Vector2f,
    pub state: PlayerStates,
    pub jump_buffer_counter: f32,
    pub dodge_end_counter: f32,
    pub dodge_stall_counter: f32,
    pub scheduler: SchedulerFFI,
    pub auto_move: i32,
    pub aiming: Boolean,
    pub index: i32,
}

impl PlayerFFI {
    pub unsafe fn to_model(&self) -> Player {
        assert!(!self.scheduler.scheduler_actions.is_null());
        assert!(!self.scheduler.scheduler_counters.is_null());
        assert!(!self.scheduler.scheduler_start_counters.is_null());

        let scheduler_actions = char_c_array_to_vec_string(self.scheduler.scheduler_actions);
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
            .position(self.position)
            .position_counter(self.position_counter)
            .wall_stick_max(self.wall_stick_max)
            .speed(self.speed)
            .state(self.state)
            .dodge_end_counter(self.dodge_end_counter)
            .dodge_stall_counter(self.dodge_stall_counter)
            .jump_buffer_counter(self.jump_buffer_counter)
            .scheduler(Scheduler::new(
                scheduler_actions,
                scheduler_counters,
                scheduler_start_counters,
            ))
            .auto_move(self.auto_move)
            .aiming(self.aiming)
            .index(self.index)
            .build()
    }

    pub fn update(&mut self, player: Player) {
        self.position = player.position();
        self.position_counter = player.position_counter();
        self.wall_stick_max = player.wall_stick_max();
        self.speed = player.speed();
        self.state = player.state();
        self.dodge_end_counter = player.dodge_end_counter();
        self.dodge_stall_counter = player.dodge_stall_counter();
        self.jump_buffer_counter = player.jump_buffer_counter();

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
        self.index = player.index();
    }
}