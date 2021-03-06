//! Module with types that represent a client session.

use futures::prelude::*;
use slog;
use std::net::SocketAddr;
use tokio_codec::Framed;
use tokio_tcp::TcpStream;

use protocol::bnet::frame::{BNetCodec, BNetPacket};
use rpc::transport::{Request, Response};

pub use self::error::*;

#[derive(Debug)]
/// A lightweight session is the smallest allocation necessary to handle a newly connected
/// client.
///
/// This session also has the bare minimum of functionality to cummunicate with the connected
/// client. This session type is of use during handshake procedures.
pub struct LightWeightSession {
    address: SocketAddr,
    codec: Option<Framed<TcpStream, BNetCodec>>,
    logger: slog::Logger,
}

impl LightWeightSession {
    /// Creates a new session object.
    pub fn new(
        address: SocketAddr,
        codec: Framed<TcpStream, BNetCodec>,
        logger: slog::Logger,
    ) -> Self {
        let codec = Some(codec);
        Self {
            address,
            codec,
            logger,
        }
    }

    /// Retrieve the address endpoint of the client.
    pub fn address(&self) -> &SocketAddr {
        &self.address
    }

    /// Retrieve a specialized logger for this session.
    pub fn logger(&self) -> &slog::Logger {
        &self.logger
    }

    fn reinstall_codec(&mut self, codec: Framed<TcpStream, BNetCodec>) {
        self.codec = Some(codec);
    }

    /// Read a request from the client.
    pub fn read_request(
        mut self,
    ) -> impl Future<Item = (Self, Request<BNetPacket>), Error = SessionError> {
        let codec = self.codec.take().unwrap();
        codec.into_future()
    		// The codec is dropped on error!
    		.map_err(|(error, _)| error.into())
    		.and_then(|(packet_opt, stream)| {
    			let packet = packet_opt.ok_or(SessionError::ClientDisconnect)?;
    			Ok((packet, stream))
    		})
    		.and_then(move |(packet, stream)| {
    			// Parse a request from the packet.
    			let request = packet.try_as_request().map_err(|_| SessionError::MissingRequest)?;
    			self.reinstall_codec(stream);
    			Ok((self, request))
    		})
    }

    /// Sends the provided response to the client.
    ///
    /// This method is designed for request->response communication. There will be reduced performance
    /// if sending a batch of responses one by one through this method.
    pub fn send_response(
        mut self,
        response: Response<BNetPacket>,
    ) -> impl Future<Item = Self, Error = SessionError> {
        let codec = self.codec.take().unwrap();
        codec
            .send(response.into_inner())
            .map_err(|error| error.into())
            .and_then(|stream| {
                self.reinstall_codec(stream);
                Ok(self)
            })
    }

    /// Transforms the current lightweight session in a complete user session.
    ///
    /// This transformation comes with big allocations.
    pub fn into_full_session(self) -> ClientSession {
        unimplemented!()
    }
}

#[derive(Debug)]
/// A complete user session.
///
/// This structure contains the necessary data to properly communicate with a specific client.
pub struct ClientSession {}

impl Future for ClientSession {
    type Item = ();
    type Error = SessionError;

    fn poll(&mut self) -> Poll<(), SessionError> {
        unimplemented!()
    }
}

mod error {
    use protocol::bnet::frame::CodecError;
    use rpc::system::RPCError;

    #[derive(Debug, Fail)]
    /// Error type related to a client session.
    pub enum SessionError {
        #[fail(display = "Client disconnected")]
        /// Session failure because of client disconnect.
        ClientDisconnect,

        #[fail(display = "Client didn't send expected request")]
        /// Session failure because of faulty client communications.
        MissingRequest,

        #[fail(display = "A passed deadline triggered a timeout on the connection")]
        /// The client did not responsd (properly) within the set deadline.
        Timeout,

        #[fail(display = "{}", _0)]
        /// Session failure due to malformed data.
        Codec(#[cause] CodecError),

        #[fail(display = "{}", _0)]
        /// Session failure due to a service error.
        RPC(#[cause] RPCError),
    }

    // Usability improvement
    impl From<CodecError> for SessionError {
        fn from(x: CodecError) -> Self {
            SessionError::Codec(x)
        }
    }

    // Usability improvement
    impl From<RPCError> for SessionError {
        fn from(x: RPCError) -> Self {
            SessionError::RPC(x)
        }
    }
}
