use serde::{Deserialize, Serialize};


pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
#[derive(Deserialize, Serialize, Debug)]
pub struct Protocol {
    pub from: String,
    pub to: String,
    pub message: String,
}
