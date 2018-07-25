//! Items for configuring and building an HS lobby server.
//!
//! A lobby server is the program responsible for authenticating players
//! and responding to in-game activities.

// TODO: Attribute can be removed when https://github.com/idanarye/rust-typed-builder/pull/4
// is merged.
#![allow(missing_docs)]

use slog;
use std::net::SocketAddr;

use log;

#[derive(Debug, Default, Clone, Copy)]
/// Object for defining how a socket binding failure must be resolved.
pub struct BindRetryConfig {
    /// Allowed amount of retries before returning an error.
    ///
    /// This error will terminate all library activity.
    max_retries: u8,
    /// Switch for allowing to bind to the next port.
    ///
    /// The next port is calculated as current_port + 1. An error is returned
    /// If the result would overflow regardless of retry count.
    try_next_port: bool,
}

#[derive(Debug, TypedBuilder)]
/// Object for configuring a [`LobbyServer`].
///
/// Construction is handled through the [typed-builder crate][], which enforces
/// a valid configuration at during compilation.
/// Doublecheck the spelling of your configuration if compilation unexpectedly
/// fails while mentioning this type.
///
/// # See also
/// [`ServerConfig::builder`]
///
/// [typed-builder crate]: https://github.com/idanarye/rust-typed-builder
pub struct ServerConfig {
    /// A combination of IP-address and port for the server to bind on.
    bind_address: SocketAddr,
    /// Controls how a binding failure must be resolved.
    bind_fallback: BindRetryConfig,

    #[default = "log::default_logger()"]
    /// Root logger instance, used for handling runtime information throughout this
    /// library.
    logger: slog::Logger,
}

#[derive(Debug)]
/// Handle for lobby server activities.
///
/// An instance of this object must be scheduled on an asynchronous runtime. TODO!
pub struct LobbyServer {}

impl LobbyServer {
    /// Constructs a new handle from the provided configuration.
    pub fn with(_config: &ServerConfig) -> Self {
        unimplemented!()
    }
}
