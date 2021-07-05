#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]
//! # Trovo API Client

mod auth;
#[cfg(feature = "chat")]
pub mod chat;
mod client;
mod entities;
mod errors;

pub use auth::*;
pub use client::*;
pub use entities::*;
pub use errors::*;

#[macro_use]
extern crate tracing;
