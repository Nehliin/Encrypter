use serde::{Deserialize, Serialize};
use x25519_dalek::SharedSecret;

pub const ID_MAX_SIZE: usize = 32 - std::mem::size_of::<String>();
pub const MESSAGE_MAX_SIZE: usize = 256 - std::mem::size_of::<String>();
pub const MESSAGE_PACKET_SIZE: usize = 32 + 32 + 256;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct EncryptedMessage(Message);

impl EncryptedMessage {
    pub fn create(mut message: Message, shared_key: &SharedSecret) -> Self {
        // encrypted
        unsafe {
            message
                .content
                .as_bytes_mut()
                .iter_mut()
                .zip(shared_key.as_bytes().iter().cycle())
                .for_each(|(mb, kb)| *mb ^= kb);
        }
        message.content = String::from_utf8_lossy(message.content.as_bytes()).to_string();
        EncryptedMessage(message)
    }
    pub fn get_message(self) -> Message {
        self.0
    }

    pub fn get_info(&self) -> (&String, &String) {
        (&self.0.from, &self.0.to)
    }
}
#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Message {
    pub from: String,
    pub to: String,
    pub content: String,
}
#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub enum Protocol {
    Message(EncryptedMessage),
    NewConnection(String, [u8; 32]),
    RemoveConnection,
    PeerList(Vec<(String, [u8; 32])>),
}
