use async_tungstenite::tungstenite::{self, protocol::CloseFrame};
use std::{error::Error, fmt::Display};

/// Errors that can happen with authenticated requests
#[derive(Debug)]
pub enum ChatConnectError {
    /// Error connecting to socket
    WebSocket(tungstenite::Error),

    /// Error serialising or deserialising entities
    Serde(serde_json::Error),

    /// The websocket closed before we could connect
    SocketClosed,
}

impl From<tungstenite::Error> for ChatConnectError {
    fn from(error: tungstenite::Error) -> Self {
        Self::WebSocket(error)
    }
}

impl From<serde_json::Error> for ChatConnectError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serde(error)
    }
}

impl Display for ChatConnectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WebSocket(e) => e.fmt(f),
            Self::Serde(e) => e.fmt(f),
            Self::SocketClosed => write!(f, "socket closed"),
        }
    }
}

impl Error for ChatConnectError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::WebSocket(e) => Some(e),
            Self::Serde(e) => Some(e),
            Self::SocketClosed => None,
        }
    }
}

/// Errors that can happen with authenticated requests
#[derive(Debug)]
pub enum ChatMessageStreamError {
    /// Error connecting to socket
    WebSocket(tungstenite::Error),

    /// Error serialising or deserialising entities
    Serde(serde_json::Error),

    /// The socket was closed by the server
    SocketClosed(Option<CloseFrame<'static>>),

    /// The server never responsed to our pings
    PingTimeout,
}

impl From<tungstenite::Error> for ChatMessageStreamError {
    fn from(error: tungstenite::Error) -> Self {
        Self::WebSocket(error)
    }
}

impl From<serde_json::Error> for ChatMessageStreamError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serde(error)
    }
}

impl Display for ChatMessageStreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WebSocket(e) => e.fmt(f),
            Self::Serde(e) => e.fmt(f),
            Self::SocketClosed(Some(frame)) => {
                write!(f, "socket was closed: {}", frame)
            }
            Self::SocketClosed(None) => {
                write!(f, "socket was closed")
            }
            Self::PingTimeout => {
                write!(f, "server stopped responding to pings")
            }
        }
    }
}

impl Error for ChatMessageStreamError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::WebSocket(e) => Some(e),
            Self::Serde(e) => Some(e),
            Self::SocketClosed(_) => None,
            Self::PingTimeout => None,
        }
    }
}
