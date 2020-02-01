use async_std::net::SocketAddr;
use async_std::net::TcpStream;
use encrypter_core::Result;
use std::collections::hash_map::{Iter, Values};
use std::collections::HashMap;
use std::sync::Arc;

pub struct Peer {
    pub peer_id: String,
    pub tcp_stream: Arc<TcpStream>,
    pub public_key: [u8; 32],
}

impl Peer {
    pub fn new(peer_id: String, tcp_stream: Arc<TcpStream>, public_key: [u8; 32]) -> Self {
        Peer {
            peer_id,
            tcp_stream,
            public_key,
        }
    }

    pub fn get_addr(&self) -> std::io::Result<SocketAddr> {
        self.tcp_stream.as_ref().peer_addr()
    }
}

pub struct PeerSet {
    id_storage: HashMap<String, Peer>,
    ip_storage: HashMap<SocketAddr, String>,
}

impl PeerSet {
    pub fn new() -> Self {
        PeerSet {
            id_storage: HashMap::new(),
            ip_storage: HashMap::new(),
        }
    }

    pub fn insert(&mut self, peer: Peer) -> Result<()> {
        if let Ok(ip) = peer.get_addr() {
            self.ip_storage.insert(ip, peer.peer_id.clone());
            self.id_storage.insert(peer.peer_id.clone(), peer);
            Ok(())
        } else {
            Err("Could not insert peer: {:?}".into())
        }
    }

    pub fn remove_by_ip(&mut self, ip: &SocketAddr) -> bool {
        if let Some(id) = self.ip_storage.remove(ip) {
            if let Some(peer) = self.id_storage.remove(&id) {
                if let Ok(socket_ip) = peer.get_addr() {
                    info!("Removed peer {}, with ip: {}", peer.peer_id, socket_ip);
                    true
                } else {
                    error!("The peer mapped to ip {}, has no ip stored", ip);
                    false
                }
            } else {
                error!("IP mapping existed for ip: {}, but no peer was found", ip);
                false
            }
        } else {
            warn!("Peer not found for ip: {}", ip);
            false
        }
    }

    pub fn remove_by_id(&mut self, id: &str) -> bool {
        if let Some(peer) = self.id_storage.remove(id) {
            if let Ok(socket_ip) = peer.get_addr() {
                if let Some(socket_ip) = self.ip_storage.remove(&socket_ip) {
                    info!("Removed peer {}, with ip: {}", peer.peer_id, socket_ip);
                    true
                } else {
                    error!(
                        "ID mapping existed for id: {}, but no ip mapping was found",
                        peer.peer_id
                    );
                    false
                }
            } else {
                error!("The peer mapped to id {}, has no ip stored", id);
                false
            }
        } else {
            warn!("Peer not found with id: {}", id);
            false
        }
    }

    pub fn find_by_id(&self, id: &str) -> Option<&Peer> {
        self.id_storage.get(id)
    }

    pub fn values(&self) -> Values<String, Peer> {
        self.id_storage.values()
    }
    // TODO: maybe implement iterator trait to this datastructure
    pub fn iter(&self) -> Iter<String, Peer> {
        self.id_storage.iter()
    }
}
