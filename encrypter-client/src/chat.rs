use crate::network::PRIVATE_KEY;
use x25519_dalek::{PublicKey, SharedSecret};

pub(crate) struct Chat {
    pub shared_key: SharedSecret,
    pub messages: Vec<String>,
}

impl Chat {
    pub fn new(public_key: [u8; 32]) -> Self {
        Chat {
            shared_key: PRIVATE_KEY.diffie_hellman(&PublicKey::from(public_key)),
            messages: Vec::new(),
        }
    }

    pub fn change_key(&mut self, public_key: [u8; 32]) {
        self.shared_key = PRIVATE_KEY.diffie_hellman(&PublicKey::from(public_key));
    }
}
