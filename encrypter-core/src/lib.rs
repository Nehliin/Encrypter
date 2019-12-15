use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Protocol {
    pub from: String,
    pub to: String,
    pub message: String,
}
