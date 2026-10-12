use std::{
    collections::HashMap,
    io::ErrorKind,
    net::{IpAddr, SocketAddr, ToSocketAddrs, UdpSocket},
    sync::{Arc, Mutex, MutexGuard},
    time::{Duration, Instant},
};

use async_tungstenite::{
    async_std::connect_async,
    tungstenite::{http::Uri, Bytes, Message as WsMessage},
};
use futures::{channel::oneshot, future::Fuse, select, FutureExt, StreamExt};
use futures_timer::Delay;
use ggrs::{Message, NonBlockingSocket, PlayerType};
use matchbox_socket::PeerId;
use serde::Deserialize;
use tracing::{info, warn};
use uuid::Uuid;

use crate::{config::ggrs_config::Address, transport::Connection};

const PACKET_HELLO: u8 = 1;
const PACKET_GAME: u8 = 2;
const TOKEN_SIZE: usize = 8;
const RECEIVE_BUFFER_SIZE: usize = 65_536;

const HELLO_INTERVAL_RETRY: Duration = Duration::from_millis(250);
const HELLO_KEEP_ALIVE: Duration = Duration::from_secs(5);
const PING_KEEP_ALIVE: Duration = Duration::from_secs(10);
const REFUSED_RETRY: Duration = Duration::from_secs(3);

const PAYLOAD_WHOLE: u8 = 0;
const PAYLOAD_PIECE: u8 = 1;
const PIECE_HEADER_SIZE: usize = 4;
const PIECE_MAX_BYTES: usize = 1200;
const MESSAGE_MAX_PIECES: usize = 128;
const UNFINISHED_MESSAGE_TIMEOUT: Duration = Duration::from_millis(500);

#[derive(Debug, Deserialize)]
enum ConnectionEvent {
    Welcome { token: u64, udp_port: u16 },
    UdpConfirmed {},
    PeerConnected { peer: String, id: u8 },
    PeerDisconnected { peer: String },
    SpectatorsFull {},
}

#[derive(Default)]
struct RelayState {
    token: Option<u64>,
    relay_addr: Option<SocketAddr>,
    is_confirmed: bool,
    is_refused: bool,
    is_released: bool,
    ids_by_peer: HashMap<PeerId, u8>,
    peers_by_id: HashMap<u8, PeerId>,
}

type ArcRelayState = Arc<Mutex<RelayState>>;

fn lock_relay_state(shared: &ArcRelayState) -> MutexGuard<'_, RelayState> {
    shared
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub struct RelaySocket {
    state: ArcRelayState,
    udp: UdpSocket,
    next_hello: Instant,
    release: Option<oneshot::Sender<()>>,
}

#[derive(Clone)]
pub struct RelayStatus(ArcRelayState);

impl RelayStatus {
    pub fn is_refused(&self) -> bool {
        lock_relay_state(&self.0).is_refused
    }
}

pub struct RelayChannel {
    state: ArcRelayState,
    udp: UdpSocket,
    receive_buffer: Vec<u8>,
    outgoing_buffer: Vec<u8>,
    encoded_message: Vec<u8>,
    next_message_id: u16,
    reassembly: Reassembly,
}

struct PartialMessage {
    message_id: u16,
    started: Instant,
    pieces: Vec<Option<Vec<u8>>>,
    missing: usize,
}

#[derive(Default)]
struct Reassembly {
    partials: HashMap<PeerId, PartialMessage>,
}

impl Reassembly {
    /// Returns the message once its last piece arrived
    fn accept(&mut self, peer: PeerId, piece: &[u8]) -> Option<Vec<u8>> {
        let (header, chunk) = piece.split_at_checked(PIECE_HEADER_SIZE)?;
        let message_id = u16::from_le_bytes([header[0], header[1]]);
        let index = header[2] as usize;
        let count = header[3] as usize;

        if !(2..=MESSAGE_MAX_PIECES).contains(&count)
            || index >= count
            || chunk.len() > PIECE_MAX_BYTES
        {
            return None;
        }

        let should_restart = match self.partials.get(&peer) {
            Some(partial) => {
                partial.message_id != message_id
                    && ((message_id.wrapping_sub(partial.message_id) as i16) > 0
                        || partial.started.elapsed() > UNFINISHED_MESSAGE_TIMEOUT)
            }
            None => true,
        };

        if should_restart {
            self.partials.insert(
                peer,
                PartialMessage {
                    message_id,
                    started: Instant::now(),
                    pieces: vec![None; count],
                    missing: count,
                },
            );
        }

        let partial = self.partials.get_mut(&peer)?;

        if partial.message_id != message_id || partial.pieces.len() != count {
            return None;
        }

        if partial.pieces[index].is_none() {
            partial.pieces[index] = Some(chunk.to_vec());
            partial.missing -= 1;
        }

        if partial.missing > 0 {
            return None;
        }

        self.partials
            .remove(&peer)
            .map(|partial| partial.pieces.into_iter().flatten().flatten().collect())
    }
}

pub fn connect(connection_url: String) -> Result<(RelaySocket, RelayChannel, Connection), String> {
    let relay_ip = resolve_host(&connection_url)?;

    let udp = UdpSocket::bind("0.0.0.0:0").map_err(|e| format!("relay : udp bind : {e}"))?;
    udp.set_nonblocking(true)
        .map_err(|e| format!("relay : udp non blocking : {e}"))?;
    let channel_udp = udp
        .try_clone()
        .map_err(|e| format!("relay : udp clone : {e}"))?;

    let state = ArcRelayState::default();
    let (release, released) = oneshot::channel();
    let connection = run_connection(connection_url, state.clone(), relay_ip, released).boxed();

    let socket = RelaySocket {
        state: state.clone(),
        udp,
        next_hello: Instant::now(),
        release: Some(release),
    };
    let channel = RelayChannel::new(state, channel_udp);

    Ok((socket, channel, connection))
}

fn resolve_host(connection_url: &str) -> Result<IpAddr, String> {
    let uri: Uri = connection_url
        .parse()
        .map_err(|e| format!("relay : invalid url '{connection_url}' : {e}"))?;
    let host = uri
        .host()
        .ok_or_else(|| format!("relay : no host in '{connection_url}'"))?;

    (host, 0)
        .to_socket_addrs()
        .map_err(|e| format!("relay : cannot resolve {host} : {e}"))?
        .map(|addr| addr.ip())
        .find(IpAddr::is_ipv4)
        .ok_or_else(|| format!("relay : no ipv4 address for {host}"))
}

async fn run_connection(
    url: String,
    relay_state: ArcRelayState,
    relay_ip: IpAddr,
    released: oneshot::Receiver<()>,
) -> Result<(), String> {
    let mut released = released.fuse();
    let mut should_join = true;

    while should_join {
        let is_released = run_websocket(&url, &relay_state, relay_ip, &mut released).await?;
        should_join = !is_released && lock_relay_state(&relay_state).is_refused;

        if should_join {
            select! {
                _ = Delay::new(REFUSED_RETRY).fuse() => {}
                _ = released => should_join = false,
            }
        }
    }

    Ok(())
}

async fn run_websocket(
    url: &str,
    relay_state: &ArcRelayState,
    relay_ip: IpAddr,
    released: &mut Fuse<oneshot::Receiver<()>>,
) -> Result<bool, String> {
    let (mut websocket, _) = connect_async(url)
        .await
        .map_err(|e| format!("relay connection failed : {e}"))?;
    let mut keep_alive = Delay::new(PING_KEEP_ALIVE).fuse();
    let mut is_open = true;
    let mut is_released = false;

    while is_open {
        let mut should_ping = false;

        select! {
            message = websocket.next().fuse() => match message {
                Some(Ok(WsMessage::Text(text))) => handle_event(text.as_str(), relay_state, relay_ip),
                Some(Ok(WsMessage::Close(_))) | None => is_open = false,
                Some(Ok(_)) => {}
                Some(Err(e)) => return Err(format!("relay connection closed : {e}")),
            },
            _ = keep_alive => should_ping = true,
            _ = &mut *released => {
                is_released = true;
                is_open = false;
            }
        }

        if should_ping {
            websocket
                .send(WsMessage::Ping(Bytes::new()))
                .await
                .map_err(|e| format!("relay connection closed : {e}"))?;
            keep_alive = Delay::new(PING_KEEP_ALIVE).fuse();
        }
    }

    if is_released {
        let _ = websocket.close(None).await;
    }

    Ok(is_released)
}

fn handle_event(text: &str, relay_state: &ArcRelayState, relay_ip: IpAddr) {
    let event: ConnectionEvent = match serde_json::from_str(text) {
        Ok(event) => event,
        Err(e) => {
            warn!("Ignoring relay event {text} : {e}");
            return;
        }
    };

    let mut state = lock_relay_state(relay_state);

    if state.is_released {
        return;
    }

    match event {
        ConnectionEvent::Welcome { token, udp_port } => {
            state.token = Some(token);
            state.relay_addr = Some(SocketAddr::new(relay_ip, udp_port));
            state.is_refused = false;
            info!("Relay joined");
        }
        ConnectionEvent::UdpConfirmed {} => {
            state.is_confirmed = true;
            info!("Relay reachable over udp");
        }
        ConnectionEvent::PeerConnected { peer, id } => match Uuid::parse_str(&peer) {
            Ok(uuid) => {
                state.ids_by_peer.insert(PeerId(uuid), id);
                state.peers_by_id.insert(id, PeerId(uuid));
                info!("Relay peer {peer} connected");
            }
            Err(e) => warn!("Ignoring relay peer '{peer}' : {e}"),
        },
        ConnectionEvent::PeerDisconnected { peer } => {
            if let Ok(uuid) = Uuid::parse_str(&peer) {
                if let Some(id) = state.ids_by_peer.remove(&PeerId(uuid)) {
                    state.peers_by_id.remove(&id);
                }
            }
            info!("Relay peer {peer} disconnected");
        }
        ConnectionEvent::SpectatorsFull {} => {
            state.is_refused = true;
            info!("Relay has no spectator slot left");
        }
    }
}

impl RelaySocket {
    /// Keeps our udp address known to the relay
    pub fn update_peers(&mut self) {
        let now = Instant::now();

        if now < self.next_hello {
            return;
        }

        let (token, relay_addr, is_confirmed) = {
            let relay_state = lock_relay_state(&self.state);
            (
                relay_state.token,
                relay_state.relay_addr,
                relay_state.is_confirmed,
            )
        };

        let (Some(token), Some(relay_addr)) = (token, relay_addr) else {
            return;
        };

        let mut hello = [0u8; 1 + TOKEN_SIZE];
        hello[0] = PACKET_HELLO;
        hello[1..].copy_from_slice(&token.to_le_bytes());

        if let Err(e) = self.udp.send_to(&hello, relay_addr) {
            warn!("Relay hello failed : {e}");
        }

        let duration = if is_confirmed {
            HELLO_KEEP_ALIVE
        } else {
            HELLO_INTERVAL_RETRY
        };

        self.next_hello = now + duration;
    }

    pub fn connected_peers(&self) -> Vec<PeerId> {
        let relay_state = lock_relay_state(&self.state);

        if !relay_state.is_confirmed {
            return vec![];
        }

        relay_state.ids_by_peer.keys().copied().collect()
    }

    pub fn players(&self) -> Vec<PlayerType<PeerId>> {
        std::iter::once(PlayerType::Local)
            .chain(self.connected_peers().into_iter().map(PlayerType::Remote))
            .collect()
    }

    pub fn status(&self) -> RelayStatus {
        RelayStatus(self.state.clone())
    }

    pub fn release(&mut self) {
        if let Some(release) = self.release.take() {
            let _ = release.send(());
        }

        *lock_relay_state(&self.state) = RelayState {
            is_released: true,
            ..RelayState::default()
        };
    }
}

impl RelayChannel {
    fn new(state: ArcRelayState, udp: UdpSocket) -> Self {
        Self {
            state,
            udp,
            receive_buffer: vec![0u8; RECEIVE_BUFFER_SIZE],
            outgoing_buffer: Vec::with_capacity(RECEIVE_BUFFER_SIZE),
            encoded_message: Vec::with_capacity(RECEIVE_BUFFER_SIZE),
            next_message_id: 0,
            reassembly: Reassembly::default(),
        }
    }

    fn receive_messages(&mut self) -> Option<(usize, SocketAddr)> {
        match self.udp.recv_from(&mut self.receive_buffer) {
            Ok(received) => Some(received),
            Err(e)
                if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::ConnectionReset =>
            {
                None
            }
            Err(e) => {
                warn!("Relay receive failed : {e}");
                None
            }
        }
    }

    pub fn send_packet(&mut self, packet: &[u8], peer: PeerId) -> bool {
        let (token, relay_addr, dest) = {
            let relay_state = lock_relay_state(&self.state);
            (
                relay_state.token,
                relay_state.relay_addr,
                relay_state.ids_by_peer.get(&peer).copied(),
            )
        };

        let (Some(token), Some(relay_addr), Some(dst)) = (token, relay_addr, dest) else {
            return false;
        };

        let count = packet.len().div_ceil(PIECE_MAX_BYTES);

        if count > MESSAGE_MAX_PIECES {
            warn!(
                "Dropped relay message of {} bytes to peer {}",
                packet.len(),
                peer.0
            );
            return false;
        }

        let message_id = self.next_message_id;

        if count > 1 {
            self.next_message_id = self.next_message_id.wrapping_add(1);
        }

        for (index, chunk) in packet.chunks(PIECE_MAX_BYTES).enumerate() {
            self.outgoing_buffer.clear();
            self.outgoing_buffer.push(PACKET_GAME);
            self.outgoing_buffer.extend_from_slice(&token.to_le_bytes());
            self.outgoing_buffer.push(dst);

            if count == 1 {
                self.outgoing_buffer.push(PAYLOAD_WHOLE);
            } else {
                self.outgoing_buffer.push(PAYLOAD_PIECE);
                self.outgoing_buffer
                    .extend_from_slice(&message_id.to_le_bytes());
                self.outgoing_buffer.push(index as u8);
                self.outgoing_buffer.push(count as u8);
            }

            self.outgoing_buffer.extend_from_slice(chunk);

            if let Err(e) = self.udp.send_to(&self.outgoing_buffer, relay_addr) {
                if e.kind() != ErrorKind::WouldBlock {
                    warn!("Dropped relay packet to peer {} : {e}", peer.0);
                }
                return false;
            }
        }

        true
    }

    pub fn receive_packets(&mut self) -> Vec<(PeerId, Box<[u8]>)> {
        let mut received = vec![];

        let Some(relay_addr) = lock_relay_state(&self.state).relay_addr else {
            return received;
        };

        while let Some((len, from)) = self.receive_messages() {
            if from != relay_addr {
                continue;
            }

            let Some((&PACKET_GAME, rest)) = self.receive_buffer[..len].split_first() else {
                continue;
            };

            let Some((&src, payload)) = rest.split_first() else {
                continue;
            };

            let Some(peer) = lock_relay_state(&self.state).peers_by_id.get(&src).copied() else {
                continue;
            };

            let packet = match payload.split_first() {
                Some((&PAYLOAD_WHOLE, bytes)) => Some(Box::from(bytes)),
                Some((&PAYLOAD_PIECE, piece)) => self
                    .reassembly
                    .accept(peer, piece)
                    .map(Vec::into_boxed_slice),
                _ => None,
            };

            if let Some(packet) = packet {
                received.push((peer, packet));
            }
        }

        received
    }
}

impl NonBlockingSocket<Address> for RelayChannel {
    fn send_to(&mut self, msg: &Message, addr: &Address) {
        let Address::Peer(peer) = addr else {
            panic!("Cannot send to socket address")
        };

        let mut encoded_message = std::mem::take(&mut self.encoded_message);
        encoded_message.clear();

        if bincode::serialize_into(&mut encoded_message, msg).is_ok() {
            self.send_packet(&encoded_message, *peer);
        }

        self.encoded_message = encoded_message;
    }

    fn receive_all_messages(&mut self) -> Vec<(Address, Message)> {
        self.receive_packets()
            .into_iter()
            .filter_map(|(peer, packet)| {
                let msg = bincode::deserialize::<Message>(&packet).ok()?;

                Some((Address::Peer(peer), msg))
            })
            .collect()
    }
}
