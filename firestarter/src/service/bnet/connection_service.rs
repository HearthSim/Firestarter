//! Service handling RPC requests/responses that manipulate the connection between
//! client and server.

use futures::future::{lazy, FutureResult};
use futures::prelude::*;

use protocol::bnet::frame::BNetPacket;
use protocol::bnet::session::LightWeightSession;
use rpc::system::RPCError;
use rpc::transport::{Request, Response};

#[derive(Debug, Default)]
/// Service handling RPC requests/responses that manipulate the connection between
/// client and server.
///
/// See the module documentation for more information.
pub struct ConnectionService {}

#[allow(missing_docs)]
#[repr(u32)]
#[derive(Debug, Copy, Clone)]
/// Addressable methods for this service.
pub enum Methods {
    Connect = 1,
    Bind = 2,
    Echo = 3,
    ForceDisconnect = 4,
    KeepAlive = 5,
    Encrypt = 6,
    RequestDisconnect = 7,
}

impl ConnectionService {
    /// See [`Methods::Connect`]
    pub const METHOD_CONNECT: Methods = Methods::Connect;
    /// See [`Methods::Bind`]
    pub const METHOD_BIND: Methods = Methods::Bind;
    /// See [`Methods::Echo`]
    pub const METHOD_ECHO: Methods = Methods::Echo;
    /// See [`Methods::ForceDisconnect`]
    pub const METHOD_FORCE_DISCONNECT: Methods = Methods::ForceDisconnect;
    /// See [`Methods::KeepAlive`]
    pub const METHOD_KEEP_ALIVE: Methods = Methods::KeepAlive;
    /// See [`Methods::Encrypt`]
    pub const METHOD_ENCRYPT: Methods = Methods::Encrypt;
    /// See [`Methods::RequestDisconnect`]
    pub const METHOD_REQUEST_DISCONNECT: Methods = Methods::RequestDisconnect;

    /// Handles a direct connect request without going through routing and service handling.
    ///
    /// This method can be used to directly handshake with a client, without side-effects.
    pub fn connect_direct(
        session: LightWeightSession,
        request: Request<BNetPacket>,
    ) -> impl Future<Item = (LightWeightSession, Response<BNetPacket>), Error = RPCError> {
        lazy(move || -> FutureResult<_, _> { unimplemented!() })
    }
}
