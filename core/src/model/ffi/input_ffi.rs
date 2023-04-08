use std::mem::forget;

use crate::model::input::Input;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub struct Inputs {
    pub data: *const Input,
    pub len: usize,
}

impl Inputs {
    pub fn new(inputs: Vec<Input>) -> Self {
        let len = inputs.len();
        let clone = inputs.clone();
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
