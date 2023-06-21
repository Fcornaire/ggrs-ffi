use std::mem::forget;

use crate::model::netplay_request::NetplayRequest;

#[repr(C)]
pub struct NetplayRequests {
    pub data: *const NetplayRequest,
    pub len: usize,
}

impl NetplayRequests {
    pub fn new(netplay_requests: Vec<NetplayRequest>) -> Self {
        let len = netplay_requests.len();
        let clone = netplay_requests.clone();
        let requests = clone.as_ptr();

        forget(requests); //TODO: swith to box
        forget(clone);

        Self {
            data: requests,
            len,
        }
    }

    pub fn empty() -> Self {
        Self {
            data: std::ptr::null_mut(),
            len: 0,
        }
    }
}
