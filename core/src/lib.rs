use neplay::Netplay;
use once_cell::sync::{Lazy, OnceCell};
use std::{
    collections::HashSet,
    ffi::CString,
    os::raw::c_char,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex, MutexGuard,
    },
    thread::sleep,
    time::Duration,
};
use tracing::warn;

pub mod config;
pub mod core;
pub mod exts;
pub mod ffi;
pub mod model;
pub mod neplay;
pub mod session;

static NETPLAY_INSTANCE: Lazy<Mutex<Netplay>> = Lazy::new(|| {
    tracing_subscriber::fmt()
        .compact()
        .with_thread_names(true)
        .with_target(false)
        .with_max_level(tracing::Level::INFO)
        .init();

    Mutex::new(Netplay::new(None))
});
static NETPLAY_HAS_DISCONNECTED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));
static SHOULD_STOP_MATCHBOX_FUTURE: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

static CONNECTED_PEERS: Lazy<Mutex<HashSet<uuid::Uuid>>> = Lazy::new(|| Mutex::new(HashSet::new()));

pub fn set_connected_peers(peers: HashSet<uuid::Uuid>) {
    if let Ok(mut set) = CONNECTED_PEERS.lock() {
        *set = peers;
    }
}

pub fn is_peer_connected(peer: &uuid::Uuid) -> bool {
    CONNECTED_PEERS
        .lock()
        .map(|set| set.contains(peer))
        .unwrap_or(false)
}

pub fn get_runtime() -> &'static tokio::runtime::Runtime {
    static RUNTIME: OnceCell<tokio::runtime::Runtime> = OnceCell::new();

    RUNTIME.get_or_init(|| {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()
            .unwrap();
        runtime
    })
}

pub(crate) fn guard_netplay_instance() -> MutexGuard<'static, Netplay> {
    match NETPLAY_INSTANCE.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            warn!("Netplay instance was poisoned by a panic, resetting it");
            NETPLAY_INSTANCE.clear_poison();

            let mut guard = poisoned.into_inner();
            *guard = Netplay::new(None);

            guard
        }
    }
}

fn has_netplay_disconnected() -> bool {
    NETPLAY_HAS_DISCONNECTED.lock().unwrap().clone()
}

const POLLER_INTERVAL: Duration = Duration::from_millis(2);

static POLLER_ALIVE: AtomicBool = AtomicBool::new(false);
static POLLER_SHOULD_STOP: AtomicBool = AtomicBool::new(false);

fn start_background_poller() {
    POLLER_SHOULD_STOP.store(false, Ordering::SeqCst);

    if POLLER_ALIVE.swap(true, Ordering::SeqCst) {
        return;
    }

    let spawned = std::thread::Builder::new()
        .name("ggrs-poller".into())
        .spawn(|| {
            while !POLLER_SHOULD_STOP.load(Ordering::Relaxed) {
                let polled = guard_netplay_instance().poll_remote().is_ok();

                if !polled {
                    break;
                }

                std::thread::sleep(POLLER_INTERVAL);
            }

            POLLER_ALIVE.store(false, Ordering::SeqCst);
        });

    if spawned.is_err() {
        POLLER_ALIVE.store(false, Ordering::SeqCst);
    }
}

fn stop_background_poller() {
    POLLER_SHOULD_STOP.store(true, Ordering::SeqCst);

    for _ in 0..200 {
        if !POLLER_ALIVE.load(Ordering::SeqCst) {
            return;
        }

        std::thread::sleep(Duration::from_millis(5));
    }
}

static MATCHBOX_LOOP_ALIVE: AtomicBool = AtomicBool::new(false);

pub(crate) fn mark_matchbox_loop(alive: bool) {
    MATCHBOX_LOOP_ALIVE.store(alive, Ordering::SeqCst);
}

pub(crate) fn set_matchbox_stop(stop: bool) {
    let mut stp = SHOULD_STOP_MATCHBOX_FUTURE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    *stp = stop;
}

pub(crate) fn matchbox_stop_requested() -> bool {
    SHOULD_STOP_MATCHBOX_FUTURE
        .try_lock()
        .map(|stp| *stp)
        .unwrap_or(false)
}

pub(crate) fn stop_matchbox_loop() {
    set_matchbox_stop(true);

    for _ in 0..100 {
        if !MATCHBOX_LOOP_ALIVE.load(Ordering::SeqCst) {
            return;
        }

        sleep(Duration::from_millis(5));
    }
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
    fn new(is_ok: Bool, info: &str) -> Self {
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

    pub fn msg(msg: &str) -> Self {
        Self::new(Bool::True, msg)
    }

    pub fn ko(info: &str) -> Self {
        Self::new(Bool::False, info)
    }
}

#[repr(C)]
pub struct Events {
    pub data: *mut *mut c_char,
    pub len: i32,
    pub cap: i32,
}

impl Events {
    pub fn new(events: Vec<String>) -> Self {
        let c_strings: Vec<*mut c_char> = events
            .iter()
            .map(|s| {
                let s = CString::new(s.as_str()).unwrap();
                s.into_raw()
            })
            .collect();

        let len = c_strings.len();
        let data = Box::into_raw(c_strings.into_boxed_slice()) as *mut *mut c_char;

        Self {
            data,
            len: len as i32,
            cap: len as i32,
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
