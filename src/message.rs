use serde::{Deserialize, Serialize};

pub const MSG_SIZE: usize = 4096;

#[derive(Deserialize, Serialize)]
pub struct TextMessage {
    msg_type: String,
    text_utf8: Vec<u8>,
}