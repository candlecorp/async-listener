use std::fmt::{self, Display};

#[derive(Debug)]
pub enum AsyncListenerError {
    NetworkError(String),
    FileSystemError(String),
}

impl Display for AsyncListenerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AsyncListenerError::NetworkError(msg) => write!(f, "Network port error: {}", msg),
            AsyncListenerError::FileSystemError(msg) => write!(f, "File system error: {}", msg),
        }
    }
}

impl AsyncListenerError {
    pub fn network_error(msg: &str) -> Self {
        AsyncListenerError::NetworkError(msg.to_owned())
    }

    pub fn file_system_error(msg: &str) -> Self {
        AsyncListenerError::FileSystemError(msg.to_owned())
    }
}

impl From<notify::Error> for AsyncListenerError {
    fn from(err: notify::Error) -> Self {
        AsyncListenerError::FileSystemError(err.to_string())
    }
}

impl From<std::io::Error> for AsyncListenerError {
    fn from(err: std::io::Error) -> Self {
        AsyncListenerError::NetworkError(err.to_string())
    }
}

impl From<&std::io::Error> for AsyncListenerError {
    fn from(err: &std::io::Error) -> Self {
        AsyncListenerError::NetworkError(err.to_string())
    }
}
