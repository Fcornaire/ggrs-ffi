use std::slice;

#[repr(C)]
#[derive(Debug)]
pub struct SafeBytes {
    ptr: *mut u8,
    size: usize,
}

impl SafeBytes {
    pub fn new(ptr: *mut u8, size: usize) -> Self {
        Self { ptr, size }
    }

    pub unsafe fn slice(&self) -> &[u8] {
        slice::from_raw_parts(self.ptr, self.size)
    }

    pub unsafe fn release(self) {
        if self.ptr.is_null() {
            return;
        }

        unsafe {
            drop(Box::from_raw(self.ptr));
        }
    }
}
