use std::{
    collections::HashSet,
    sync::mpsc,
    time::{Duration, Instant},
};

use futures::{future::BoxFuture, select, FutureExt};
use ggrs::{Message, NonBlockingSocket, PlayerType};
use matchbox_socket::{PeerId, WebRtcChannel, WebRtcSocket};
use tracing::{info, warn};

use crate::{
    config::ggrs_config::Address,
    exts::WebRtcSocketGgrsExtensions,
    relay::{self, RelayChannel, RelaySocket, RelayStatus},
};

const RELEASE_RELAY_AFTER: Duration = Duration::from_secs(2);

pub trait PeerSocket: Send {
    /// False once the socket is closed
    fn update_peers(&mut self) -> bool;
    fn connected_peers(&self) -> Vec<PeerId>;
    fn players(&mut self) -> Vec<PlayerType<PeerId>>;
}

impl PeerSocket for WebRtcSocket {
    fn update_peers(&mut self) -> bool {
        self.try_update_peers().is_ok()
    }

    fn connected_peers(&self) -> Vec<PeerId> {
        WebRtcSocket::connected_peers(self).collect()
    }

    fn players(&mut self) -> Vec<PlayerType<PeerId>> {
        WebRtcSocketGgrsExtensions::players(self)
    }
}

impl PeerSocket for RelaySocket {
    fn update_peers(&mut self) -> bool {
        RelaySocket::update_peers(self);
        true
    }

    fn connected_peers(&self) -> Vec<PeerId> {
        RelaySocket::connected_peers(self)
    }

    fn players(&mut self) -> Vec<PlayerType<PeerId>> {
        RelaySocket::players(self)
    }
}

pub struct RoutedSocket {
    direct: WebRtcSocket,
    relay: RelaySocket,
    direct_peers: HashSet<PeerId>,
    relay_peers: HashSet<PeerId>,
    direct_only_peers: HashSet<PeerId>,
    routes: mpsc::Sender<HashSet<PeerId>>,
    relay_until_direct: Option<PeerId>,
    direct_since: Option<Instant>,
}

impl RoutedSocket {
    /// Leaves the relay once that peer stayed reachable directly
    fn release_relay_if_direct(&mut self) {
        let Some(peer) = self.relay_until_direct else {
            return;
        };

        if !self.direct_peers.contains(&peer) {
            self.direct_since = None;
            return;
        }

        let since = *self.direct_since.get_or_insert_with(Instant::now);

        if since.elapsed() >= RELEASE_RELAY_AFTER {
            info!("Direct connection to {} is up, leaving the relay", peer.0);
            self.relay.release();
            self.relay_until_direct = None;
        }
    }
}

impl PeerSocket for RoutedSocket {
    fn update_peers(&mut self) -> bool {
        let is_direct_open = PeerSocket::update_peers(&mut self.direct);
        self.relay.update_peers();

        let direct_peers: HashSet<PeerId> = self.direct.connected_peers().collect();

        if direct_peers != self.direct_peers {
            let _ = self.routes.send(direct_peers.clone());
            self.direct_peers = direct_peers;
        }

        self.release_relay_if_direct();

        is_direct_open
    }

    fn connected_peers(&self) -> Vec<PeerId> {
        let direct = self
            .direct_peers
            .iter()
            .filter(|peer| !self.relay_peers.contains(*peer))
            .copied();
        let relayed = self
            .relay
            .connected_peers()
            .into_iter()
            .filter(|peer| !self.direct_only_peers.contains(peer));

        direct
            .chain(relayed)
            .collect::<HashSet<PeerId>>()
            .into_iter()
            .collect()
    }

    fn players(&mut self) -> Vec<PlayerType<PeerId>> {
        std::iter::once(PlayerType::Local)
            .chain(self.connected_peers().into_iter().map(PlayerType::Remote))
            .collect()
    }
}

pub struct RoutedChannel {
    direct: WebRtcChannel,
    relay: RelayChannel,
    relay_peers: HashSet<PeerId>,
    direct_only_peers: HashSet<PeerId>,
    direct_peers: HashSet<PeerId>,
    routes: mpsc::Receiver<HashSet<PeerId>>,
    encoded_message: Vec<u8>,
}

impl RoutedChannel {
    fn send_packet(&mut self, packet: &[u8], peer: PeerId) -> bool {
        if let Some(direct_peers) = self.routes.try_iter().last() {
            self.direct_peers = direct_peers;
        }

        let is_direct = self.direct_peers.contains(&peer);

        if self.relay_peers.contains(&peer) {
            return self.relay.send_packet(packet, peer);
        }

        if !is_direct
            && !self.direct_only_peers.contains(&peer)
            && self.relay.send_packet(packet, peer)
        {
            return true;
        }

        is_direct && self.direct.try_send(Box::from(packet), peer).is_ok()
    }

    fn receive_packets(&mut self) -> Vec<(PeerId, Box<[u8]>)> {
        let mut packets = self.direct.receive();
        packets.extend(self.relay.receive_packets());

        packets
    }
}

pub enum PeerChannel {
    WebRtc(WebRtcChannel),
    Relay(RelayChannel),
    Routed(Box<RoutedChannel>),
}

impl PeerChannel {
    pub fn send_packet(&mut self, packet: &[u8], peer: PeerId) -> bool {
        match self {
            PeerChannel::WebRtc(channel) => channel.try_send(Box::from(packet), peer).is_ok(),
            PeerChannel::Relay(channel) => channel.send_packet(packet, peer),
            PeerChannel::Routed(channel) => channel.send_packet(packet, peer),
        }
    }

    pub fn receive_packets(&mut self) -> Vec<(PeerId, Box<[u8]>)> {
        match self {
            PeerChannel::WebRtc(channel) => channel.receive(),
            PeerChannel::Relay(channel) => channel.receive_packets(),
            PeerChannel::Routed(channel) => channel.receive_packets(),
        }
    }
}

impl NonBlockingSocket<Address> for PeerChannel {
    fn send_to(&mut self, msg: &Message, addr: &Address) {
        match self {
            PeerChannel::WebRtc(channel) => NonBlockingSocket::send_to(channel, msg, addr),
            PeerChannel::Relay(channel) => NonBlockingSocket::send_to(channel, msg, addr),
            PeerChannel::Routed(channel) => NonBlockingSocket::send_to(channel.as_mut(), msg, addr),
        }
    }

    fn receive_all_messages(&mut self) -> Vec<(Address, Message)> {
        match self {
            PeerChannel::WebRtc(channel) => NonBlockingSocket::receive_all_messages(channel),
            PeerChannel::Relay(channel) => NonBlockingSocket::receive_all_messages(channel),
            PeerChannel::Routed(channel) => {
                NonBlockingSocket::receive_all_messages(channel.as_mut())
            }
        }
    }
}

impl NonBlockingSocket<Address> for RoutedChannel {
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

pub type Connection = BoxFuture<'static, Result<(), String>>;

pub struct Transport {
    pub socket: Box<dyn PeerSocket>,
    pub channel: PeerChannel,
    pub connection: Connection,
}

pub fn open(room_url: String, relay_url: Option<String>) -> Result<Transport, String> {
    match relay_url {
        Some(relay_url) => {
            let (socket, channel, connection) = relay::connect(relay_url)?;

            Ok(Transport {
                socket: Box::new(socket),
                channel: PeerChannel::Relay(channel),
                connection,
            })
        }
        None => {
            let (socket, channel, connection) = open_webrtc(room_url)?;

            Ok(Transport {
                socket: Box::new(socket),
                channel: PeerChannel::WebRtc(channel),
                connection,
            })
        }
    }
}

/// Opens both paths
pub fn open_routed(
    room_url: String,
    relay_url: String,
    relay_peers: HashSet<PeerId>,
    direct_only_peers: HashSet<PeerId>,
    relay_until_direct: Option<PeerId>,
) -> Result<(Transport, RelayStatus), String> {
    let (direct, direct_channel, direct_connection) = open_webrtc(room_url)?;
    let (relay, relay_channel, relay_connection) = relay::connect(relay_url)?;
    let (routes, routes_receiver) = mpsc::channel();
    let status = relay.status();

    let transport = Transport {
        socket: Box::new(RoutedSocket {
            direct,
            relay,
            direct_peers: HashSet::new(),
            relay_peers: relay_peers.clone(),
            direct_only_peers: direct_only_peers.clone(),
            routes,
            relay_until_direct,
            direct_since: None,
        }),
        channel: PeerChannel::Routed(Box::new(RoutedChannel {
            direct: direct_channel,
            relay: relay_channel,
            relay_peers,
            direct_only_peers,
            direct_peers: HashSet::new(),
            routes: routes_receiver,
            encoded_message: Vec::new(),
        })),
        connection: follow_primary(direct_connection, relay_connection).boxed(),
    };

    Ok((transport, status))
}

fn open_webrtc(room_url: String) -> Result<(WebRtcSocket, WebRtcChannel, Connection), String> {
    let (mut socket, message_loop) = WebRtcSocket::new_unreliable(room_url);
    let channel = socket
        .take_channel(0)
        .map_err(|e| format!("take_channel : {:?}", e))?;
    let connection = message_loop
        .map(|result| result.map_err(|e| e.to_string()))
        .boxed();

    Ok((socket, channel, connection))
}

async fn follow_primary(primary: Connection, secondary: Connection) -> Result<(), String> {
    let mut primary = primary.fuse();
    let mut secondary = secondary.fuse();

    loop {
        select! {
            result = primary => return result,
            result = secondary => match result {
                Ok(()) => info!("Secondary connection closed"),
                Err(e) => warn!("Secondary connection closed : {e}"),
            },
        }
    }
}
