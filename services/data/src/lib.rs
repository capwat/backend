use cfg_if::cfg_if;

cfg_if! {
  if #[cfg(feature = "grpc")] {
    mod client;
    pub use client::DataServiceClient;
  }
}

mod r#impl;

pub mod config;
pub mod db;

pub use r#impl::{DataService, DataServiceInitError};
