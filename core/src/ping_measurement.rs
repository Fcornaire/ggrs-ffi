use std::{
    collections::{HashMap, VecDeque},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

use matchbox_socket::{PeerId, PeerState, WebRtcSocket};
use once_cell::sync::Lazy;
use tracing::{info, warn};
use uuid::Uuid;

use crate::get_runtime;

const POLL_INTERVAL: Duration = Duration::from_millis(2);
const PING_INTERVAL: Duration = Duration::from_millis(500);
const SAMPLE_WINDOW: usize = 24;
const LOSS_TIMEOUT: Duration = Duration::from_millis(1500);
const SPIKE_PERCENTILE: usize = 90;
const HALF_MILLISECOND: Duration = Duration::from_micros(500);
const PACKET_LEN: usize = 1 + std::mem::size_of::<u64>();

const PING: u8 = 0;
const PONG: u8 = 1;

static INSTANT: Lazy<Instant> = Lazy::new(Instant::now);
static PING_MEASUREMENT: Lazy<Mutex<Option<PingMeasurement>>> = Lazy::new(|| Mutex::new(None));

type Samples = Arc<Mutex<HashMap<Uuid, PeerSamples>>>;

#[derive(Default)]
struct PeerSamples {
    outcomes: VecDeque<Option<Duration>>,
    pending: VecDeque<u64>,
}

impl PeerSamples {
    fn record(&mut self, outcome: Option<Duration>) {
        if self.outcomes.len() == SAMPLE_WINDOW {
            self.outcomes.pop_front();
        }

        self.outcomes.push_back(outcome);
    }

    fn expire_pending(&mut self, now: Duration) {
        while let Some(&stamp) = self.pending.front() {
            if now.saturating_sub(Duration::from_micros(stamp)) < LOSS_TIMEOUT {
                break;
            }

            self.pending.pop_front();
            self.record(None);
        }
    }
}

#[repr(C)]
pub struct PingStats {
    pub rtt: i32,   // median
    pub spike: i32, // slow rtt
    pub loss_percent: i32,
    pub samples: i32,
}

impl PingStats {
    pub fn none() -> Self {
        Self {
            rtt: -1,
            spike: 0,
            loss_percent: 0,
            samples: 0,
        }
    }
}

struct PingMeasurement {
    stop: Arc<AtomicBool>,
    samples: Samples,
    thread: JoinHandle<()>,
}

pub fn start(room_url: String) -> Result<(), String> {
    stop();

    let stop_flag = Arc::new(AtomicBool::new(false));
    let samples: Samples = Arc::new(Mutex::new(HashMap::new()));

    let thread = {
        let stop_flag = stop_flag.clone();
        let samples = samples.clone();

        std::thread::Builder::new()
            .name("ping-measurement".into())
            .spawn(move || run(room_url, stop_flag, samples))
            .map_err(|e| format!("Failed to spawn ping measurement : {}", e))?
    };

    *PING_MEASUREMENT.lock().unwrap_or_else(|p| p.into_inner()) = Some(PingMeasurement {
        stop: stop_flag,
        samples,
        thread,
    });

    Ok(())
}

pub fn stop() {
    let measurement = PING_MEASUREMENT
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .take();

    if let Some(measurement) = measurement {
        measurement.stop.store(true, Ordering::SeqCst);
        let _ = measurement.thread.join();
    }
}

pub fn stats(peer: &Uuid) -> PingStats {
    let measurement = PING_MEASUREMENT.lock().unwrap_or_else(|p| p.into_inner());

    let Some(measurement) = measurement.as_ref() else {
        return PingStats::none();
    };

    let samples = measurement
        .samples
        .lock()
        .unwrap_or_else(|p| p.into_inner());

    let Some(peer_samples) = samples.get(peer) else {
        return PingStats::none();
    };

    let outcomes = &peer_samples.outcomes;

    if outcomes.is_empty() {
        return PingStats::none();
    }

    let mut received: Vec<Duration> = outcomes.iter().flatten().copied().collect();
    received.sort_unstable();

    let lost = outcomes.len() - received.len();
    let loss_percent = (lost * 100 / outcomes.len()) as i32;

    if received.is_empty() {
        return PingStats {
            loss_percent,
            samples: outcomes.len() as i32,
            ..PingStats::none()
        };
    }

    let median = received[received.len() / 2];
    let slow = received[(received.len() * SPIKE_PERCENTILE / 100).min(received.len() - 1)];

    PingStats {
        rtt: to_millis(median),
        spike: to_millis(slow - median),
        loss_percent,
        samples: outcomes.len() as i32,
    }
}

fn to_millis(duration: Duration) -> i32 {
    (duration + HALF_MILLISECOND).as_millis() as i32
}

fn run(room_url: String, stop: Arc<AtomicBool>, samples: Samples) {
    info!("Ping measurement started");

    let (mut socket, message_loop) = WebRtcSocket::new_unreliable(room_url);
    let message_loop = get_runtime().spawn(message_loop);
    let mut next_ping = Instant::now();

    while !stop.load(Ordering::Relaxed) {
        match socket.try_update_peers() {
            Ok(changes) => {
                for (peer, state) in changes {
                    match state {
                        PeerState::Connected => info!("Ping measurement connected to {}", peer.0),
                        PeerState::Disconnected => {
                            info!("Ping measurement lost {}", peer.0);
                            samples
                                .lock()
                                .unwrap_or_else(|p| p.into_inner())
                                .remove(&peer.0);
                        }
                    }
                }
            }
            Err(_) => {
                warn!("Ping measurement signaling closed");
                break;
            }
        }

        let channel = socket.channel_mut(0);

        for (peer, packet) in channel.receive() {
            handle_packet(channel, peer, &packet, &samples);
        }

        if Instant::now() >= next_ping {
            next_ping = Instant::now() + PING_INTERVAL;

            let peers: Vec<PeerId> = socket.connected_peers().collect();
            let channel = socket.channel_mut(0);
            let now = INSTANT.elapsed();
            let stamp = now.as_micros() as u64;
            let mut samples = samples.lock().unwrap_or_else(|p| p.into_inner());

            for peer in peers {
                let sent = channel.try_send(encode(PING, stamp), peer).is_ok();

                if let Some(peer_samples) = samples.get_mut(&peer.0) {
                    peer_samples.expire_pending(now);

                    if sent {
                        peer_samples.pending.push_back(stamp);
                    }
                }
            }
        }

        std::thread::sleep(POLL_INTERVAL);
    }

    socket.close();
    message_loop.abort();

    info!("Ping measurement stopped");
}

fn handle_packet(
    channel: &mut matchbox_socket::WebRtcChannel,
    peer: PeerId,
    packet: &[u8],
    samples: &Samples,
) {
    let Some((kind, stamp)) = decode(packet) else {
        return;
    };

    match kind {
        PING => {
            let _ = channel.try_send(encode(PONG, stamp), peer);
        }
        PONG => {
            let rtt = INSTANT
                .elapsed()
                .saturating_sub(Duration::from_micros(stamp));
            let mut samples = samples.lock().unwrap_or_else(|p| p.into_inner());
            let peer_samples = samples.entry(peer.0).or_default();

            match peer_samples
                .pending
                .iter()
                .position(|pending| *pending == stamp)
            {
                Some(index) => {
                    peer_samples.pending.remove(index);
                    peer_samples.record(Some(rtt));
                }
                None if peer_samples.outcomes.is_empty() => peer_samples.record(Some(rtt)),
                None => {}
            }
        }
        _ => {}
    }
}

//Stamp so that we dont have to deal with clock divergence across computer
fn encode(kind: u8, stamp: u64) -> Box<[u8]> {
    let mut packet = Vec::with_capacity(PACKET_LEN);
    packet.push(kind);
    packet.extend_from_slice(&stamp.to_le_bytes());

    packet.into_boxed_slice()
}

fn decode(packet: &[u8]) -> Option<(u8, u64)> {
    if packet.len() != PACKET_LEN {
        return None;
    }

    let stamp = u64::from_le_bytes(packet[1..PACKET_LEN].try_into().ok()?);

    Some((packet[0], stamp))
}
