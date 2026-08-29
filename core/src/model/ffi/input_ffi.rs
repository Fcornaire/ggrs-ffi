use crate::model::input::Input;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub struct Inputs {
    pub data: *const Input,
    pub len: i32,
}

impl Inputs {
    pub fn new(inputs: Vec<Input>) -> Self {
        let len = inputs.len();
        let data = Box::into_raw(inputs.into_boxed_slice()) as *const Input;

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
