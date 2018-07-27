//! Entry point for new connected clients who communicate by using the BNet protocol.
//!
//! Clients must succesfully complete the handshake before the server allocates memory
//! for a new session.

use futures::future::{lazy, FutureResult};
use futures::prelude::*;
use slog;
use std::io;
use std::sync::{Arc, Mutex};
use tokio_codec::Decoder;
use tokio_tcp::TcpStream;

pub use self::error::*;
use protocol::bnet::frame::BNetCodec;
use server::lobby::ServerShared;

/// Maximum duration between accepting a new client and completing the handshake.
/// The connection is closed when the deadline expires.
const HANDSHAKE_COMPLETE_DEADLINE: u64 = 5;

/// Perform the BNet protocol handshake with the provided client.
pub fn handle_client(
    client: TcpStream,
    shared: Arc<Mutex<ServerShared>>,
    logger: slog::Logger,
) -> Result<impl Future<Item = (), Error = ()>, io::Error> {
    let peer = client.peer_addr()?;
    // Values provided to new logger instances must be Owned+Send.
    let peer_logger = logger.new(o!("peer" => format!("{:?}", peer)));
    trace!(peer_logger, "Client connected");

    let codec = BNetCodec::new().framed(client);

    Ok(lazy(|| -> FutureResult<(), ()> { unimplemented!() }))
}

fn handshake_operation() -> impl Future<Item = (), Error = HandshakeError> {
    lazy(|| -> FutureResult<(), HandshakeError> { unimplemented!() })
}

mod error {
    use protocol::bnet::frame::CodecError;

    #[derive(Debug, Fail)]
    /// Error type related to the handshake operation between client and server.
    pub enum HandshakeError {
        #[fail(display = "{}", _0)]
        /// Handshake failed due to malformed data.
        Codec(#[cause] CodecError),
    }

    // Implementation necessary as per constraint from Encoder::Error + Decoder::Error
    impl From<CodecError> for HandshakeError {
        fn from(x: CodecError) -> Self {
            HandshakeError::Codec(x)
        }
    }
}
