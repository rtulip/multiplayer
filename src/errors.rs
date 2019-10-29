use std::error;
use std::fmt;
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct ClientDisconnectError{
    pub addr: SocketAddr,
}

impl fmt::Display for ClientDisconnectError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Client {} Disconnected", self.addr)
    }
}

impl error::Error for ClientDisconnectError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    } 
}

type Result<T> = std::result::Result<T, ClientDisconnectError>;
