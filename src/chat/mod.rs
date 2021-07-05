//! # Chat
//!
//! Connect to Trovo chat via websockets

mod client;
mod entities;
mod error;
mod socket;

pub use entities::*;
pub use error::*;
pub use socket::*;
