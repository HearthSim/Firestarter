//! Items for configuring and building an HS lobby server.
//!
//! A lobby server is the program responsible for authenticating players
//! and responding to in-game activities.

use futures::prelude::*;
use slog;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::runtime;
use tokio_executor as executor;
use tokio_tcp::TcpListener;

use log;
use protocol::bnet;

// Re-export all types defined within the error submodule (see below)
pub use self::error::*;

/// Amount of time to pause accepting new clients when an I/O error is returned
/// by the listening socket.
const _DEFAULT_ERROR_TIMEOUT: Duration = Duration::from_millis(100);

/// Maximum amount of clients to accept.
/// This value is set to make sure we don't deplete all system resources.
///
/// There is a hard limit on the amount of connections a system can handle, which
/// depends on your OS.
const _DEFAULT_MAX_CONNECTIONS: usize = 1000;

#[derive(Debug, Default, Clone, Copy, TypedBuilder)]
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
/// Object for sending commands to a running server.
pub struct ServerHandle {
    _inner: (),
}

#[derive(Debug)]
/// Handle for lobby server activities.
///
/// An instance of this object must be scheduled on an asynchronous runtime. TODO!
pub struct LobbyServer {
    listener: TcpListener,
    config: ServerConfig,
}

impl LobbyServer {
    /// Starts the server.
    ///
    /// This method sets up a Tokio runtime and executes the server task.
    /// This method only returns AFTER all tasks have been completed and/or dropped.
    pub fn run(self) {
        // Do NOT ignore the handle, because it could contain communication channels.
        // Channel receivers/senders return an error when the other side is dropped which
        // *could* prematurely finish the future task.
        let (_handle, task) = self.split();
        runtime::run(task);
    }

    /// Split this object into a control-handle and a future.
    ///
    /// The future is a task which executes the activities of a lobby server. This task
    /// must be scheduled on your Tokio runtime.
    /// The handle can be used to interact with the task (=server) while it's running.
    pub fn split(self) -> (ServerHandle, impl Future<Item = (), Error = ()>) {
        let LobbyServer { listener, config } = self;
        let ServerConfig { logger, .. } = config;

        let handle = ServerHandle { _inner: () };
        let shared = Arc::new(Mutex::new(ServerShared {}));
        let err_logger = logger.clone();

        let task = listener
            .incoming()
            .for_each(move |client| {
                let task_build_result =
                    bnet::handshake::handle_client(client, shared.clone(), logger.clone());

                match task_build_result {
                    Ok(task) => executor::spawn(task),
                    Err(e) => info!(logger, "Handshake task creation failed"; "error" => ?e),
                };

                Ok(())
            })
            .map_err(move |e| error!(err_logger, "Server loop ended with error!"; "error" => ?e));

        (handle, task)
    }

    /// Constructs a new handle from the provided configuration.
    pub fn with(config: ServerConfig) -> Result<Self, BindError> {
        let listener = Self::try_tcp_bind(&config.bind_address, &config.bind_fallback)?;
        Ok(Self { listener, config })
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

#[derive(Debug, Default)]
/// Structure containing data accessible to each client handler.
pub struct ServerShared {}

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
