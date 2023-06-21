use lazy_static::lazy_static;
use neplay::Netplay;
use std::{
    ffi::CString,
    mem::forget,
    os::raw::c_char,
    sync::{Arc, Mutex},
};

pub mod config;
pub mod core;
pub mod ffi;
pub mod model;
pub mod neplay;
pub mod session;
pub mod utils;

lazy_static! {
    pub static ref NETPLAY: Mutex<Netplay> = Mutex::new(Netplay::new(None));
    pub static ref SHOULD_STOP_MATCHBOX_FUTURE: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}

#[repr(u8)]
enum Bool {
    False = 0,
    True = 1,
}

impl Bool {
    pub fn is_true(&self) -> bool {
        match self {
            Bool::True => true,
            Bool::False => false,
        }
    }
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

    pub fn is_ok(&self) -> bool {
        self.is_ok.is_true()
    }

    pub fn ok() -> Self {
        Self::new(Bool::True, "OK")
    }

    pub fn msg(msg: &'static str) -> Self {
        Self::new(Bool::True, msg)
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
