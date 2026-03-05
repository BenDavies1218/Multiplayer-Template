//! Common utilities for the avian_3d_character example
//!
//! This module contains shared infrastructure for building client and server applications.

pub mod shared;
pub mod cli;
#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "server")]
pub mod server;
