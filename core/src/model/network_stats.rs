#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct NetworkStats {
    pub send_queue_len: u32,
    pub ping: u32,
    pub kbps_sent: u32,
    pub local_frames_behind: i32,
    pub remote_frames_behind: i32,
}

impl NetworkStats {
    pub fn new(
        send_queue_len: usize,
        ping: u128,
        kbps_sent: usize,
        local_frames_behind: i32,
        remote_frames_behind: i32,
    ) -> Self {
        Self {
            send_queue_len: send_queue_len as u32,
            ping: ping as u32,
            kbps_sent: kbps_sent as u32,
            local_frames_behind,
            remote_frames_behind,
        }
    }
}
