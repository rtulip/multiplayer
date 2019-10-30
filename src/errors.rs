use std::error;
use std::fmt;
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct ClientDisconnectError{
    pub addr: SocketAddr,
}

#[derive(Debug, Clone)]
pub struct InputHandleError;

#[derive(Debug, Clone)]
pub struct UnexpectedError;

impl fmt::Display for ClientDisconnectError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Client {} Disconnected", self.addr)
    }
}

impl fmt::Display for InputHandleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Problem with handling input!")
    }
}

impl fmt::Display for UnexpectedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unexpected Error!")
    }
}

impl error::Error for ClientDisconnectError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    } 
}

impl error::Error for InputHandleError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    } 
}

impl error::Error for UnexpectedError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    } 
}

pub type ConnectionStatus = std::result::Result<(), ClientDisconnectError>;
pub type ExpectedSuccess = std::result::Result<(), UnexpectedError>;
