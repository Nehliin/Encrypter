use aes_soft::block_cipher_trait::generic_array::GenericArray;
use aes_soft::block_cipher_trait::BlockCipher;
use aes_soft::Aes256;
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
        // encrypt the message
        let key = GenericArray::from_slice(shared_key.as_bytes());
        let cipher = Aes256::new(&key);

        message
            .content
            .as_mut_slice()
            .chunks_exact_mut(16)
            .for_each(|mut chunk| {
                cipher.encrypt_block(GenericArray::from_mut_slice(&mut chunk));
            });

        let last_block = &mut [0u8; 16];
        message
            .content
            .as_slice()
            .chunks_exact(16)
            .remainder()
            .iter()
            .zip(last_block.iter_mut())
            .for_each(|(chunk_byte, lblock_byte)| *lblock_byte = *chunk_byte);
        cipher.encrypt_block(GenericArray::from_mut_slice(last_block));
        let message_lenght = message.content.len();
        let mut encrypted_message: Vec<u8> =
            message.content[..message_lenght - (message_lenght % 16)].to_vec();
        encrypted_message.extend_from_slice(last_block);
        message.content = encrypted_message;
        EncryptedMessage(message)
    }
    pub fn decrypt_message(mut self, shared_key: &SharedSecret) -> Message {
        let key = GenericArray::from_slice(shared_key.as_bytes());
        let cipher = Aes256::new(&key);
        self.0
            .content
            .as_mut_slice()
            .chunks_exact_mut(16)
            .for_each(|mut chunk| {
                cipher.decrypt_block(GenericArray::from_mut_slice(&mut chunk));
            });
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
    pub content: Vec<u8>,
}
#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub enum Protocol {
    Message(EncryptedMessage),
    NewConnection(String, [u8; 32]),
    RemoveConnection,
    PeerList(Vec<(String, [u8; 32])>),
}
