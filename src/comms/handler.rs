use crate::comms::message;
use serde_json::Value;

pub trait TryClone: std::marker::Sized {
    fn try_clone(&self) -> std::io::Result<Self>;
}

/// Default implementer of Handler
#[derive(Clone)]
pub struct DefaultHandler;

impl TryClone for DefaultHandler {
    fn try_clone(&self) -> std::io::Result<DefaultHandler> {
        Ok(self.clone())
    }
}

impl Handler for DefaultHandler {
    fn handle_text_msg(&mut self, msg: message::TextMessage) {
        println!("Text: {}", msg.text);
    }
    fn handle_request_client_id(&mut self, msg: message::RequestClientID) {}
    fn handle_request_client_id_response(&mut self, msg: message::RequestClientIDResponse) {}
}

/// Trait to define how to handle different types of messages.
///
/// # Example
///
/// ```
/// extern crate multiplayer;
/// use serde_json::json;
/// use serde_json;
/// use multiplayer::comms::message;
/// use multiplayer::comms::handler::{Handler, DefaultHandler};
///
/// let mut h = DefaultHandler;
/// let msg = json!({
///     "msg_type": "Text",
///     "data": {
///         "text": "Hello Handler!"
///     }
/// });
///
/// let msg = msg.to_string();
/// let buff = msg.into_bytes();
///
/// h.receive_json(&buff);
/// ```
pub trait Handler: TryClone {
    fn handle_text_msg(&mut self, msg: message::TextMessage) {}
    fn handle_request_client_id(&mut self, msg: message::RequestClientID) {}
    fn handle_request_client_id_response(&mut self, msg: message::RequestClientIDResponse) {}
    fn handle_login_status(&mut self, msg: message::LoginStatus) {}
    fn handle_request_join_game(&mut self, msg: message::RequestJoinGame) {}
    fn handle_request_join_game_response(&mut self, msg: message::RequestJoinGameResponse) {}

    fn receive_json(&mut self, buff: &Vec<u8>) {
        let v = self.parse_json(buff);
        let identifier = v.get("msg_type").unwrap();
        let data = v.get("data").unwrap();
        let data_string = serde_json::to_string(data).expect("Failed to convert data");

        println!("Received Json: {}", v);
        match identifier {
            Value::String(text) => {
                match text.as_str() {
                    message::TEXT_MESSAGE_IDENTIFIER => {
                        // handle text message
                        let msg: message::TextMessage = serde_json::from_str(data_string.as_str())
                            .expect("Failed to parse TextMessage");
                        self.handle_text_msg(msg);
                    }
                    message::REQUEST_CLIENT_ID_IDENTIFIER => {
                        // handle client id request
                        let msg: message::RequestClientID =
                            serde_json::from_str(data_string.as_str())
                                .expect("Failed to parse RequestClientID");
                        self.handle_request_client_id(msg);
                    }
                    message::REQUEST_CLIENT_ID_RESPONSE_IDENTIFIER => {
                        // handle client id request response
                        let msg: message::TextMessage = serde_json::from_str(data_string.as_str())
                            .expect("Failed to parse RequestClientIDResponse");
                        self.handle_text_msg(msg);
                    }
                    message::LOGIN_STATUS_IDENTIFIER => {
                        let msg: message::LoginStatus = serde_json::from_str(data_string.as_str())
                            .expect("Failed to parse LoginStatus");
                        self.handle_login_status(msg);
                    }
                    message::REQUEST_JOIN_GAME_IDENTIFIER => {
                        let msg: message::RequestJoinGame = serde_json::from_str(data_string.as_str())
                            .expect("Failed to parse RequestClientIDResponse");
                        self.handle_request_join_game(msg);
                    }
                    message::REQUEST_JOIN_GAME_RESPONSE_IDENTIFIER => {
                        let msg: message::RequestJoinGameResponse = serde_json::from_str(data_string.as_str())
                            .expect("Failed to parse RequestClientIDResponse");
                        self.handle_request_join_game_response(msg);
                    }
                    _ => println!("Unknown Message Identifier"),
                }
            }
            _ => println!("No Identifier Provided"),
        }
    }

    /// Returns a Value from a buffer.
    fn parse_json(&self, buff: &Vec<u8>) -> Value {
        let msg = buff
            .clone()
            .into_iter()
            .take_while(|&x| x != 0)
            .collect::<Vec<_>>();
        let string = String::from_utf8(msg).expect("Invlaid utf8 message");
        let v: Value = serde_json::from_str(string.as_str()).expect("Unable to convert to json");
        v
    }

    /// Checks if message identifier matches any of the IDENTIFIER constants.
    fn is_type(&self, buff: &Vec<u8>, id: &str) -> bool {
        let v = self.parse_json(buff);
        let identifier = v.get("msg_type").unwrap();
        let data = v.get("data").unwrap();
        let data_string = serde_json::to_string(data).expect("Failed to convert data");

        println!("Received Json: {}", v);
        match identifier {
            Value::String(text) => text == id,
            _ => false,
        }
    }
}
