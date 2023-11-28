// This is used for the kernel's service prelude module
#![allow(clippy::wildcard_imports)]

mod client;
mod protobuf;
mod server;

pub use client::ClientLayer;
pub use server::ServerLayer;
