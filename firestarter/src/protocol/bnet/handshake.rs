//! Entry point for new connected clients who communicate by using the BNet protocol.
//!
//! Clients must succesfully complete the handshake before the server allocates memory
//! for a new session.

use futures::future::{lazy, FutureResult};
use futures::prelude::*;
use slog;
use std::sync::{Arc, Mutex};
use tokio_tcp::TcpStream;

use server::lobby::ServerShared;

/// Perform the BNet protocol handshake with the provided client.
pub fn handle_client(
    _client: TcpStream,
    _shared: Arc<Mutex<ServerShared>>,
    _logger: slog::Logger,
) -> impl Future<Item = (), Error = ()> {
    lazy(|| -> FutureResult<(), ()> { unimplemented!() })
}
