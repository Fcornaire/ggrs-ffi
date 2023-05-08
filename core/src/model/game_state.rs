use serde::{Deserialize, Serialize};

use crate::core::unmanaged::{safe_bytes::SafeBytes, unmanaged_bytes::UnmanagedBytes};

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct GameState {
    data: UnmanagedBytes,
    frame: i32,
}

impl GameState {
    pub fn new(safe_bytes: SafeBytes) -> Self {
        Self {
            data: UnmanagedBytes::new(safe_bytes),
            frame: 0,
        }
    }

    pub fn empty() -> Self {
        Self {
            data: UnmanagedBytes::empty(),
            frame: 0,
        }
    }

    pub fn data(&self) -> UnmanagedBytes {
        self.data.clone()
    }

    pub unsafe fn release(&mut self) {
        self.data = UnmanagedBytes::empty();
    }

    pub fn frame(&self) -> i32 {
        self.frame
    }

    pub fn update_frame(&mut self, frame: i32) {
        self.frame = frame;
    }

    pub fn add_frame(&mut self) {
        self.frame += 1;
    }
}
