use futures::prelude::*;
use futures::Stream;
use slog;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio_codec::Framed;
use tokio_tcp::TcpStream;

use protocol::bnet::frame::{BNetCodec, BNetPacket};
use protocol::bnet::router::{ClientSharedData, RoutingLogistic};
use protocol::bnet::session::{ClientHash, ClientSession, SessionError};
use rpc::transport::{Request, Response};
use server::lobby::ServerShared;

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
    ) -> impl Future<Item = LightWeightSession, Error = SessionError> {
        let codec = self.codec.take().unwrap();
        codec
            .send(response.unwrap())
            .map_err(|error| error.into())
            .and_then(|stream| {
                self.reinstall_codec(stream);
                Ok(self)
            })
    }

    /// Transforms the current lightweight session in a complete user session.
    ///
    /// This transformation comes with big allocations.
    pub fn into_full_session(
        self,
        server_shared: Arc<Mutex<ServerShared>>,
    ) -> impl Future<Item = (), Error = SessionError> {
        let LightWeightSession {
            address,
            codec,
            logger,
        } = self;
        let client_id = ClientHash::from_socket_address(address);
        let shared_data = ClientSharedData::new(client_id, server_shared, logger);
        //
        let codec = codec.expect("Codec contract invalid");
        let router = RoutingLogistic::default_handlers(shared_data, codec);
        let session_future = ClientSession::new(address, router);
        session_future
    }
}
