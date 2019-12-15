use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub enum Protocol {
    From(String),
    To(String),
    Message(String),
}
