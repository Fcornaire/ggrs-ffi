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
const SAMPLE_WINDOW: usize = 8;
const HALF_MILLISECOND: Duration = Duration::from_micros(500);
const PACKET_LEN: usize = 1 + std::mem::size_of::<u64>();

const PING: u8 = 0;
const PONG: u8 = 1;

static EPOCH: Lazy<Instant> = Lazy::new(Instant::now);
static PING_MEASUREMENT: Lazy<Mutex<Option<PingMeasurement>>> = Lazy::new(|| Mutex::new(None));

type Samples = Arc<Mutex<HashMap<Uuid, VecDeque<Duration>>>>;

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

/// Median round trip
pub fn rtt(peer: &Uuid) -> i32 {
    let measurement = PING_MEASUREMENT.lock().unwrap_or_else(|p| p.into_inner());

    let Some(measurement) = measurement.as_ref() else {
        return -1;
    };

    let samples = measurement
        .samples
        .lock()
        .unwrap_or_else(|p| p.into_inner());

    match samples.get(peer) {
        Some(window) if !window.is_empty() => {
            let mut sorted: Vec<Duration> = window.iter().copied().collect();
            sorted.sort_unstable();

            let median = sorted[sorted.len() / 2];

            (median + HALF_MILLISECOND).as_millis() as i32
        }
        _ => -1,
    }
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

            for peer in peers {
                let _ = channel.try_send(encode(PING, EPOCH.elapsed().as_micros() as u64), peer);
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
            let rtt = EPOCH.elapsed().saturating_sub(Duration::from_micros(stamp));
            let mut samples = samples.lock().unwrap_or_else(|p| p.into_inner());
            let window = samples.entry(peer.0).or_default();

            if window.len() == SAMPLE_WINDOW {
                window.pop_front();
            }

            window.push_back(rtt);
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
