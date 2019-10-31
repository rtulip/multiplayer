use crate::comms::message;
use serde_json::Value;

pub trait Handler {

    fn handle_text_msg(&mut self, msg: message::TextMessage);
    fn handle_request_client_id(&mut self, msg: message::RequestClientID);
    fn handle_request_client_id_response(&mut self, msg: message::RequestClientIDResponse);

    fn receive_json(&mut self, buff: &Vec<u8>) {
        
        let msg = buff.clone().into_iter().take_while(|&x| x!= 0).collect::<Vec<_>>();
        let string = String::from_utf8(msg).expect("Invlaid utf8 message");

        let v: Value = serde_json::from_str(string.as_str()).expect("Unable to convert to json");
        let identifier = v.get("msg_type").unwrap();
        let data = v.get("data").unwrap();
        let data_string = serde_json::to_string(data).expect("Failed to convert data");

        println!("Received Json: {}", v);
        match identifier {
            Value::String(text) => {
                match text.as_str() {
                    message::TEXT_MESSAGE_IDENTIFIER => {
                        // handle text message
                        let msg: message::TextMessage = serde_json::from_str(data_string.as_str()).expect("Failed to parse TextMessage");
                        self.handle_text_msg(msg);
                    },
                    message::REQUEST_CLIENT_ID_IDENTIFIER => {
                        // handle client id request
                        let msg: message::RequestClientID = serde_json::from_str(data_string.as_str()).expect("Failed to parse RequestClientID");
                        self.handle_request_client_id(msg);
                    },
                    message::REQUEST_CLIENT_ID_RESPONSE_IDENTIFIER => {
                        // handle client id request response
                        let msg: message::TextMessage = serde_json::from_str(data_string.as_str()).expect("Failed to parse RequestClientIDResponse");
                        self.handle_text_msg(msg);
                    },
                    _ => println!("Unknown Message Identifier"),
                }
            },
            _ => println!("No Identifier Provided"),
        }
    }

}

