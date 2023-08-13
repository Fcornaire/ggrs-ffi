use std::sync::Mutex;

use crate::{neplay::Netplay, reset_netplay_instance};

pub trait MutexNetplayExtensions {
    unsafe fn ensure_not_poisoned(&self);
}

impl MutexNetplayExtensions for Mutex<Netplay> {
    unsafe fn ensure_not_poisoned(&self) {
        let res = match self.lock() {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        };

        if res.is_err() {
            reset_netplay_instance();
        }
    }
}
