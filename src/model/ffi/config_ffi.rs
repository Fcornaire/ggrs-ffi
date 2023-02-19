use core::slice;
use std::os::raw::{c_char, c_uchar};

use crate::model::{boolean::Boolean, config::Config};

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub struct ConfigFFI {
    remote_addr: *mut c_char,
    input_delay: i32,
    is_test_mode: Boolean,
    test_check_distance: i32,
}

impl ConfigFFI {
    pub unsafe fn to_model(&self) -> Config {
        let bytes = {
            assert!(!self.remote_addr.is_null());

            let len = libc::strlen(self.remote_addr as *const i8) as usize;
            let slice = slice::from_raw_parts(self.remote_addr as *const c_uchar, len);
            slice
        };
        let remote_addr = match std::str::from_utf8(bytes) {
            Ok(s) => String::from(s),
            Err(e) => panic!("Error while converting remote addr string UTF-8 : {}", e),
        };

        Config::new(
            remote_addr,
            self.input_delay,
            self.is_test_mode.is_on(),
            self.test_check_distance,
        )
    }
}
