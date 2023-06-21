use core::fmt;
use std::net::SocketAddr;

use ggrs::{Config, Message, UdpNonBlockingSocket};
use matchbox_socket::{Packet, PeerId, WebRtcChannel};

use crate::model::{game_state::GameState, input::Input};

#[derive(Debug)]
pub struct GGRSConfig;
impl Config for GGRSConfig {
    type Input = Input; // Copy + Clone + PartialEq + bytemuck::Pod + bytemuck::Zeroable
    type State = GameState; // Clone
    type Address = Address; // Clone + PartialEq + Eq + Hash
}

#[derive(Hash, Debug, Clone, PartialEq, Eq)]
pub enum Address {
    Socket(SocketAddr),
    Peer(PeerId),
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Address::Socket(socket_addr) => {
                write!(f, "{}:{}", socket_addr.ip(), socket_addr.port())
            }
            Address::Peer(peer_id) => {
                write!(f, "{}", peer_id.0)
            }
        }
    }
}

fn build_packet(msg: &Message) -> Packet {
    bincode::serialize(&msg).unwrap().into_boxed_slice()
}

fn deserialize_packet(message: (PeerId, Packet)) -> (PeerId, Message) {
    (message.0, bincode::deserialize(&message.1).unwrap())
}

impl ggrs::NonBlockingSocket<Address> for WebRtcChannel {
    fn send_to(&mut self, msg: &Message, addr: &Address) {
        match addr {
            Address::Socket(_) => panic!("Cannot send to socket address, use a peer id instead"),
            Address::Peer(peer_id) => self.send(build_packet(msg), *peer_id),
        }
    }

    fn receive_all_messages(&mut self) -> Vec<(Address, Message)> {
        self.receive()
            .into_iter()
            .map(deserialize_packet)
            .map(|x| (Address::Peer(x.0), x.1))
            .collect::<Vec<(Address, Message)>>()
    }
}

impl ggrs::NonBlockingSocket<Address> for UdpNonBlockingSocket {
    fn send_to(&mut self, msg: &Message, address: &Address) {
        match address {
            Address::Socket(addr) => self.send_to(msg, addr),
            Address::Peer(_) => {
                panic!("Cannot send to peer id, use a socket address instead")
            }
        }
    }

    fn receive_all_messages(&mut self) -> Vec<(Address, Message)> {
        self.receive_all_messages()
            .into_iter()
            .map(|x| (Address::Socket(x.0), x.1))
            .collect::<Vec<(Address, Message)>>()
    }
}
