//! Entry point for new connected clients who communicate by using the BNet protocol.
//!
//! Clients must succesfully complete the handshake before the server allocates memory
//! for a new session.

use futures::future::lazy;
use futures::prelude::*;
use slog;
use std::io;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio_codec::Decoder;
use tokio_tcp::TcpStream;
use tokio_timer::Deadline;

use protocol::bnet::frame::BNetCodec;
use protocol::bnet::session::LightWeightSession;
use protocol::bnet::session::SessionError;
use rpc::transport::Response;
use server::lobby::ServerShared;

pub use self::error::*;

/// Maximum duration between accepting a new client and completing the handshake.
/// The connection is closed when the deadline expires.
const HANDSHAKE_DURATION_DEADLINE: u64 = 5;

/// Perform the BNet protocol handshake with the provided client.
pub fn handle_client(
    client: TcpStream,
    shared: Arc<Mutex<ServerShared>>,
    logger: slog::Logger,
) -> Result<impl Future<Item = (), Error = ()>, io::Error> {
    let peer_addr = client.peer_addr()?;
    // Values provided to new logger instances must be Owned+Send, so a String is created
    // from the peer address.
    let peer_logger = logger.new(o!("peer" => format!("{:?}", peer_addr)));
    let handshake_logger = peer_logger.clone();
    let handler_logger = peer_logger.clone();
    trace!(peer_logger, "Client connected");

    let codec = BNetCodec::new().framed(client);
    let session = LightWeightSession::new(peer_addr, codec, peer_logger);
    let handshake = handshake_operation(session);

    // Wrap the handshake procedure in a deadline. The client connection is closed when the
    // deadline passes. All allocated resources are cleaned up as well.
    let handshake_deadline = Instant::now() + Duration::from_secs(HANDSHAKE_DURATION_DEADLINE);
    let handshake = Deadline::new(handshake, handshake_deadline);
    let handshake = handshake
        .map_err(|deadline_err| match deadline_err.into_inner() {
            Some(handshake_error) => handshake_error,
            _ => SessionError::Timeout,
        })
        // 'inspect_err' is not available on 'Future' because it wasn't backported. It's available
        // on 'Stream' so don't get confused!
        .map_err(move |error| {
            warn!(handshake_logger, "Handshake failed"; "reason" => %error);
            error
        })
        // The full session is a future itself. It will only complete when asked or errored (including timeout).
        .and_then(|session| session.into_full_session())
        .map_err(
            move |error| error!(handler_logger, "Client handler returned error"; "error" => ?error),
        )
        .map(|_| ());
    Ok(handshake)
}

fn handshake_operation(
    session: LightWeightSession,
) -> impl Future<Item = LightWeightSession, Error = HandshakeError> {
    use service::bnet::connection_service::ConnectionService;

    session
        .read_request()
        .and_then(|(session, request)| {
            ConnectionService::connect_direct(session, &request)
                .map_err(Into::into)
                .map(move |(session, response_bytes)| {
                    let response_packet = Response::from_request(request, response_bytes);
                    (session, response_packet)
                })
        })
        .and_then(|(session, response)| session.send_response(response))
        .inspect(|session| trace!(session.logger(), "Handshake was successful"))
}

mod error {
    use protocol::bnet::session::SessionError;
    // Just shortcut the type until it's meaningful to have specific errors
    // for handshake situations.

    /// Error thrown during handshaking with new clients.
    pub type HandshakeError = SessionError;
}
