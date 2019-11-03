use crate::comms::handler::{Handler, TryClone};
use crate::comms::message;
use crate::errors::InputHandleError;
use crate::threading::dispatcher::Dispatcher;
use std::net::TcpStream;

pub struct HostClient {
    pub dispatch: Dispatcher,
    pub socket: TcpStream,
}

impl HostClient {
    pub fn new(ip: &str, dispatch: Dispatcher) -> HostClient {
        let socket = TcpStream::connect(ip).expect("Unable to connect to server");
        HostClient { dispatch, socket }
    }
}

impl TryClone for HostClient {
    fn try_clone(&self) -> std::io::Result<HostClient> {
        Ok(HostClient {
            dispatch: self.dispatch.clone(),
            socket: self.socket.try_clone()?,
        })
    }
}

impl Handler for HostClient {
    fn handle_text_msg(&mut self, msg: message::TextMessage) {
        println!("Received A Text Message: {}", msg.text);
    }

    fn handle_request_client_id(&mut self, msg: message::RequestClientID) {
        println!("Received a Request for Client ID");
        let mut socket_clone = self.socket.try_clone().expect("Failed to clone socket");
        self.dispatch.execute(move || {
            let id = read_input_line("Enter your ID:").expect("Error Reading Client ID from stdin");
            let response = message::RequestClientIDResponse { id };
            message::send_json(response, &mut socket_clone);
        })
    }
}

fn read_input_line(prompt: &str) -> Result<String, InputHandleError> {
    let mut msg = String::new();
    println!("{}", prompt);
    match std::io::stdin().read_line(&mut msg) {
        Ok(_buff_size) => return Ok(msg),
        Err(_) => Err(InputHandleError),
    }
}
