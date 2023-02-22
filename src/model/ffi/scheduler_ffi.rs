use std::os::raw::c_char;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub struct SchedulerFFI {
    pub scheduler_actions: *mut *mut c_char,
    pub scheduler_actions_length: i32,
    pub scheduler_counters: *mut f32,
    pub scheduler_counters_length: i32,
    pub scheduler_start_counters: *mut i32,
    pub scheduler_start_counters_length: i32,
}
