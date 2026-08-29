use std::sync::Mutex;

use ggrs::PlayerType;
use matchbox_socket::{PeerId, WebRtcSocket};

use crate::{neplay::Netplay, reset_netplay_instance};

// matchbox_socket's `ggrs` feature helper, that feature compiles against crates.io ggrs
// Don't work with our fork's
// Late spectate is merged
// Can delete delete this and re enable the feature when released
pub trait WebRtcSocketGgrsExtensions {
    fn players(&mut self) -> Vec<PlayerType<PeerId>>;
}

impl WebRtcSocketGgrsExtensions for WebRtcSocket {
    fn players(&mut self) -> Vec<PlayerType<PeerId>> {
        let Some(our_id) = self.id() else {
            return vec![PlayerType::Local];
        };

        let mut ids: Vec<_> = self
            .connected_peers()
            .chain(std::iter::once(our_id))
            .collect();
        ids.sort();

        ids.into_iter()
            .map(|id| {
                if id == our_id {
                    PlayerType::Local
                } else {
                    PlayerType::Remote(id)
                }
            })
            .collect()
    }
}

pub trait MutexNetplayExtensions {
    unsafe fn ensure_not_poisoned(&self);
}

impl MutexNetplayExtensions for Mutex<Netplay> {
    unsafe fn ensure_not_poisoned(&self) {
        if self.is_poisoned() {
            reset_netplay_instance();
        }
    }
}
