use serde::{Deserialize, Serialize};

pub const MSG_SIZE: usize = 4096;

#[derive(Deserialize, Serialize)]
pub struct TextMessage {
    msg_type: String,
    text_utf8: String,
}

impl TextMessage{

    pub fn new<S: Into<String>>(text: S) -> TextMessage{

        TextMessage{
            msg_type: "Text".to_owned(),
            text_utf8: text.into(),
        }

    }

}