//! Items for configuring and building an HS lobby server.
//!
//! A lobby server is the program responsible for authenticating players
//! and responding to in-game activities.

use slog;
use std::net::SocketAddr;
use tokio_tcp::TcpListener;

use log;

// Re-export all types defined within the error submodule (see below)
pub use self::error::*;

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
pub struct LobbyServer {
    listener: TcpListener,
    logger: slog::Logger,
}

impl LobbyServer {
    /// Constructs a new handle from the provided configuration.
    pub fn with(config: &ServerConfig) -> Result<Self, BindError> {
        let ServerConfig {
            bind_address,
            bind_fallback,
            logger,
        } = config;
        let listener = Self::try_tcp_bind(bind_address, bind_fallback)?;
        Ok(Self {
            listener,
            logger: logger.clone(),
        })
    }

    /// Attempt binding to the provided address.
    fn try_tcp_bind(
        address: &SocketAddr,
        config: &BindRetryConfig,
    ) -> Result<TcpListener, BindError> {
        let max_retries = config.max_retries as usize;
        let attempt_port_begin = address.port();

        let mut try_address = address.clone();
        let mut listener = None;

        for _ in 0..=max_retries {
            let bind_result = TcpListener::bind(&try_address);

            match (bind_result, max_retries) {
                (Ok(l), _) => {
                    listener = Some(l);
                    break;
                }
                (Err(e), 0) => Err(BindError::Io(e))?,
                (Err(_), _) => {
                    let current_port = try_address.port();
                    let next_port = current_port.checked_add(1).ok_or(BindError::PortOverflow)?;
                    try_address.set_port(next_port);
                }
            }
        }

        listener.ok_or(BindError::ExhaustedRetries(
            config.max_retries,
            attempt_port_begin,
        ))
    }
}

mod error {
    use std::io;

    #[derive(Debug, Fail)]
    /// Error type related to binding a server to a specific address and port.
    pub enum BindError {
        #[fail(display = "Exhausted allowed amount of retries ({}) starting from port {}", _0, _1)]
        /// Failure to bind after the allowed amount of retries.
        ExhaustedRetries(u8, u16),

        #[fail(display = "Calculation for the next port overflowed")]
        /// Failure to bind due to port number overflow.
        PortOverflow,

        #[fail(display = "{}", _0)]
        /// Failure to bind due to some input/output related error.
        Io(#[cause] io::Error),
    }
}
