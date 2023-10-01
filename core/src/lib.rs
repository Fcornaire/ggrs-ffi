use backtrace::Backtrace;
use exts::MutexNetplayExtensions;
use neplay::Netplay;
use once_cell::sync::{Lazy, OnceCell};
use std::{ffi::CString, mem::forget, os::raw::c_char, panic, sync::Mutex};
use tracing::error;

pub mod config;
pub mod core;
pub mod exts;
pub mod ffi;
pub mod model;
pub mod neplay;
pub mod session;
pub mod utils;

static mut NETPLAY_INSTANCE: OnceCell<Mutex<Netplay>> = OnceCell::new();
static NETPLAY_HAS_DISCONNECTED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));
static SHOULD_STOP_MATCHBOX_FUTURE: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

unsafe fn get_netplay_intance() -> &'static Mutex<Netplay> {
    let mutex = NETPLAY_INSTANCE.get_or_init(|| {
        tracing_subscriber::fmt()
            .compact()
            .with_thread_names(true)
            .with_target(false)
            .with_max_level(tracing::Level::INFO)
            .init();

        panic::set_hook(Box::new(|panic_info| {
            let backtrace = Backtrace::new();

            if let Some(s) = panic_info.payload().downcast_ref::<String>() {
                error!("[ggrs-ffi] panic occured {s:?}=> {:?}", backtrace);
            }
        }));

        Mutex::new(Netplay::new(None))
    });

    mutex.ensure_not_poisoned();

    mutex
}

unsafe fn reset_netplay_instance() {
    NETPLAY_INSTANCE.take();

    match NETPLAY_INSTANCE.set(Mutex::new(Netplay::new(None))) {
        Ok(_) => {}
        Err(_) => {}
    }
}

fn has_netplay_disconnected() -> bool {
    NETPLAY_HAS_DISCONNECTED.lock().unwrap().clone()
}

fn set_netplay_disconnected(disconnected: bool) {
    *NETPLAY_HAS_DISCONNECTED.lock().unwrap() = disconnected;
}

#[repr(C)]
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
