use crate::Status;

use super::unmanaged::safe_bytes::SafeBytes;

#[repr(C)]
pub struct ActionResult {
    data: SafeBytes,
    status: Status,
}

impl ActionResult {
    pub fn ok(data: SafeBytes) -> Self {
        Self {
            data,
            status: Status::ok(),
        }
    }

    pub fn ko(msg: String, data: SafeBytes) -> Self {
        Self {
            data,
            status: Status::ko(&msg),
        }
    }
}
