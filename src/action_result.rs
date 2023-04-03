use crate::Status;

#[repr(C)]
pub struct ActionResult<T> {
    data: T,
    status: Status,
}

impl<T: Default> ActionResult<T> {
    pub fn ok(data: T) -> Self {
        Self {
            data: data,
            status: Status::ok(),
        }
    }

    pub fn ko(msg: String) -> Self {
        Self {
            data: T::default(),
            status: Status::ko(Box::leak(msg.into_boxed_str())),
        }
    }
}
