use crate::model::netplay_request::NetplayRequest;

#[repr(C)]
pub struct NetplayRequests {
    pub data: *const NetplayRequest,
    pub len: i32,
}

impl NetplayRequests {
    pub fn new(netplay_requests: Vec<NetplayRequest>) -> Self {
        let len = netplay_requests.len();
        let data = Box::into_raw(netplay_requests.into_boxed_slice()) as *const NetplayRequest;

        Self {
            data,
            len: len as i32,
        }
    }

    pub fn empty() -> Self {
        Self {
            data: std::ptr::null_mut(),
            len: 0,
        }
    }
}
