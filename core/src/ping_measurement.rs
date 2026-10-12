use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, MutexGuard,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

use matchbox_socket::PeerId;
use once_cell::sync::Lazy;
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    get_runtime,
    transport::{self, PeerChannel, Transport},
};

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
static PING_MEASUREMENTS: Lazy<Mutex<PingMeasurements>> =
    Lazy::new(|| Mutex::new(PingMeasurements::default()));

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

#[derive(Default)]
struct PingMeasurements {
    direct: Option<PingMeasurement>,
    relay: Option<PingMeasurement>,
}

impl PingMeasurements {
    fn get(&self, is_relay: bool) -> Option<&PingMeasurement> {
        if is_relay {
            self.relay.as_ref()
        } else {
            self.direct.as_ref()
        }
    }

    fn get_mut(&mut self, is_relay: bool) -> &mut Option<PingMeasurement> {
        if is_relay {
            &mut self.relay
        } else {
            &mut self.direct
        }
    }
}

fn lock_ping_measurements() -> MutexGuard<'static, PingMeasurements> {
    PING_MEASUREMENTS.lock().unwrap_or_else(|p| p.into_inner())
}

pub fn start(url: String, is_relay: bool) -> Result<(), String> {
    stop(is_relay);

    let stop_flag = Arc::new(AtomicBool::new(false));
    let samples: Samples = Arc::new(Mutex::new(HashMap::new()));

    let thread = {
        let stop_flag = stop_flag.clone();
        let samples = samples.clone();

        std::thread::Builder::new()
            .name(thread_name(is_relay).into())
            .spawn(move || run(url, is_relay, stop_flag, samples))
            .map_err(|e| format!("Failed to spawn ping measurement : {}", e))?
    };

    *lock_ping_measurements().get_mut(is_relay) = Some(PingMeasurement {
        stop: stop_flag,
        samples,
        thread,
    });

    Ok(())
}

pub fn stop(is_relay: bool) {
    let measurement = lock_ping_measurements().get_mut(is_relay).take();

    if let Some(measurement) = measurement {
        measurement.stop.store(true, Ordering::SeqCst);
        let _ = measurement.thread.join();
    }
}

pub fn stats(peer: &Uuid, is_relay: bool) -> PingStats {
    let measurements = lock_ping_measurements();

    let Some(measurement) = measurements.get(is_relay) else {
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

fn thread_name(is_relay: bool) -> &'static str {
    if is_relay {
        "relay-ping-measurement"
    } else {
        "ping-measurement"
    }
}

fn label(is_relay: bool) -> &'static str {
    if is_relay {
        "Relay ping measurement"
    } else {
        "Ping measurement"
    }
}

fn run(url: String, is_relay: bool, stop: Arc<AtomicBool>, samples: Samples) {
    let label = label(is_relay);
    info!("{label} started");

    let relay_url = is_relay.then(|| url.clone());

    let Transport {
        mut socket,
        mut channel,
        connection,
    } = match transport::open(url, relay_url) {
        Ok(transport) => transport,
        Err(e) => {
            warn!("{label} could not start : {e}");
            return;
        }
    };

    let connection = get_runtime().spawn(connection);
    let mut connected: HashSet<PeerId> = HashSet::new();
    let mut next_ping = Instant::now();

    while !stop.load(Ordering::Relaxed) {
        if connection.is_finished() || !socket.update_peers() {
            warn!("{label} signaling closed");
            break;
        }

        for (peer, packet) in channel.receive_packets() {
            handle_packet(&mut channel, peer, &packet, &samples);
        }

        if Instant::now() >= next_ping {
            next_ping = Instant::now() + PING_INTERVAL;

            let peers: HashSet<PeerId> = socket.connected_peers().into_iter().collect();
            let now = INSTANT.elapsed();
            let stamp = now.as_micros() as u64;
            let mut samples = samples.lock().unwrap_or_else(|p| p.into_inner());

            for peer in peers.difference(&connected) {
                info!("{label} connected to {}", peer.0);
            }

            for peer in connected.difference(&peers) {
                info!("{label} lost {}", peer.0);
                samples.remove(&peer.0);
            }

            connected = peers;

            for peer in &connected {
                let sent = channel.send_packet(&encode(PING, stamp), *peer);

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

    connection.abort();

    info!("{label} stopped");
}

fn handle_packet(channel: &mut PeerChannel, peer: PeerId, packet: &[u8], samples: &Samples) {
    let Some((kind, stamp)) = decode(packet) else {
        return;
    };

    match kind {
        PING => {
            channel.send_packet(&encode(PONG, stamp), peer);
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
