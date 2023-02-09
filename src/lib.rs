use ggrs::Config;
use model::input::Input;
use neplay::Netplay;
use serde::{Deserialize, Serialize};
use std::{ffi::CString, mem::forget, net::SocketAddr, os::raw::c_char};

pub mod ffi;
pub mod model;
pub mod neplay;

static mut NETPLAY: Netplay = Netplay::new(None);

#[derive(Clone, Serialize, Deserialize)]
#[repr(C)]
pub struct State {
    pub frame: i32,
    pub num_players: usize,
    pub positions: Vec<(f32, f32)>,
    pub velocities: Vec<(f32, f32)>,
    pub rotations: Vec<f32>,
}

#[derive(Debug)]
pub struct GGRSConfig;
impl Config for GGRSConfig {
    type Input = Input; // Copy + Clone + PartialEq + bytemuck::Pod + bytemuck::Zeroable
    type State = State; // Clone
    type Address = SocketAddr; // Clone + PartialEq + Eq + Hash
}

#[repr(u8)]
enum Bool {
    False = 0,
    True = 1,
}

#[repr(C)]
pub struct Status {
    is_ok: Bool,
    info: *mut c_char,
}

impl Status {
    fn new(is_ok: Bool, info: &'static str) -> Self {
        let c_str = CString::new(info).unwrap();

        Self {
            is_ok,
            info: c_str.into_raw(),
        }
    }

    pub fn ok() -> Self {
        Self::new(Bool::True, "OK")
    }

    pub fn ko(info: &'static str) -> Self {
        Self::new(Bool::False, info)
    }
}

#[repr(C)]
pub struct Events {
    pub data: *mut *mut c_char,
    pub len: usize,
    pub cap: usize,
}

impl Events {
    pub fn new(events: Vec<&str>) -> Self {
        let mut c_strings: Vec<*mut c_char> = events
            .iter()
            .map(|s| {
                let s = CString::new(*s).unwrap();
                s.into_raw()
            })
            .collect();

        let c_strings_ptr = c_strings.as_mut_ptr();

        forget(c_strings);

        Self {
            data: c_strings_ptr,
            len: events.len(),
            cap: events.capacity(),
        }
    }

    pub fn empty() -> Self {
        Self {
            data: std::ptr::null_mut(),
            len: 0,
            cap: 0,
        }
    }
}
