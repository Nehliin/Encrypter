use serde::{Deserialize, Serialize};

pub const ID_MAX_SIZE: usize = 32 - std::mem::size_of::<String>();
pub const MESSAGE_MAX_SIZE: usize = 256 - std::mem::size_of::<String>();
pub const MESSAGE_PACKET_SIZE: usize = 32 + 32 + 256;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Message {
    pub from: String,
    pub to: String,
    pub content: String,
}
#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub enum Protocol {
    Message(Message),
    NewConnection(String),
    RemoveConnection,
    PeerList(Vec<String>),
}
