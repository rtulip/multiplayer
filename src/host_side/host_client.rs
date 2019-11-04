use crate::comms::handler::{Handler, TryClone};
use crate::comms::message;
use crate::errors::InputHandleError;
use crate::threading::dispatcher::Dispatcher;
use std::net::{Shutdown, TcpStream};

#[derive(Clone, Copy)]
pub enum HostClientState {
    Waiting,
    InQueue,
    InGame,
}

pub struct HostClient {
    pub dispatch: Dispatcher,
    pub socket: TcpStream,
    pub state: HostClientState,
}

impl HostClient {
    pub fn new(ip: &str, dispatch: Dispatcher) -> HostClient {
        let socket = TcpStream::connect(ip).expect("Unable to connect to server");
        let state = HostClientState::Waiting;
        HostClient {
            dispatch,
            socket,
            state,
        }
    }
}

impl TryClone for HostClient {
    fn try_clone(&self) -> std::io::Result<HostClient> {
        Ok(HostClient {
            dispatch: self.dispatch.clone(),
            socket: self.socket.try_clone()?,
            state: self.state.clone(),
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

    fn handle_login_status(&mut self, msg: message::LoginStatus) {
        if msg.success {
            let socket_clone = self.socket.try_clone().expect("Failed to clone socket");
            let dispatch_clone = self.dispatch.clone();
            self.dispatch
                .execute_loop(move || match read_input_line("Enter a Message") {
                    Ok(msg) => {
                        let mut s_clone = socket_clone.try_clone().expect("Unable to clone stream");
                        dispatch_clone.execute(move || {
                            parse_text(msg, &mut s_clone);
                        });
                        Ok(())
                    }
                    Err(e) => Err(e),
                });
        } else {
            println!("Failed to login successfully!");
            self.socket
                .shutdown(Shutdown::Both)
                .expect("Shutdown call failed");
        }
    }

    fn handle_request_join_game_response(&mut self, msg: message::RequestJoinGameResponse) {
        println!("In Queue. Waiting for {} player(s)", msg.waiting_for);
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

fn parse_text(string: String, mut socket: &mut TcpStream) {
    if string.starts_with("/join") {
        println!("Entering Queue");
        let msg = message::RequestJoinGame;
        message::send_json(msg, &mut socket);
    } else if string.starts_with("/quit") {
        println!("Logging Out")
    } else {
        let msg = message::TextMessage::new(string);
        message::send_json(msg, &mut socket);
    }
}
