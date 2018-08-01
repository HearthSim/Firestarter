//! Important types for defining an RPC service.
use bytes::Bytes;
use futures::prelude::*;

use rpc::transport::{RPCPacket, Request, Response, RouteHeader};
use rpc::util::fnv_hash_bytes;

pub use self::error::*;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
/// Unique representation value of a specific service.
pub struct ServiceHash(u32);

impl ServiceHash {
    /// Creates a new ServiceHash value from the provided name.
    pub fn from_name<S: AsRef<str>>(name: S) -> Self {
        let data = name.as_ref().as_bytes();
        let hash = fnv_hash_bytes(data);
        ServiceHash(hash)
    }
}

#[allow(missing_docs)]
pub trait RPCService {
    type Method: 'static + Sized;
    type Error;

    fn get_hash() -> ServiceHash;
    fn get_name() -> &'static str;
    fn get_methods() -> &'static [(&'static str, &'static Self::Method)];
}

#[allow(missing_docs)]
pub trait RPCObject: RPCService {
    type Packet: RPCPacket;
    type Future: Future<Item = Option<Bytes>, Error = Self::Error>;

    fn recognize(packet: &Request<Self::Packet>) -> Result<&'static Self::Method, Self::Error>;
    fn call(
        &mut self,
        method: &'static Self::Method,
        packet: &Request<Self::Packet>,
    ) -> Self::Future;
}

#[allow(missing_docs)]
pub trait RPCProxy: RPCService {
    type Packet: RPCPacket;
    type Future: Future<Item = (), Error = Self::Error>;

    fn recognize(
        request_metadata: &RouteHeader,
        full_packet: &Response<Self::Packet>,
    ) -> Result<Self::Method, Self::Error>;
    fn handle_response(
        &mut self,
        method: Self::Method,
        packet: Response<Self::Packet>,
    ) -> Self::Future;
}

/*
#[allow(missing_docs)]
pub trait Recognize {
    type Method: 'static + Sized;
    type Incoming;
    type Service: RPCService<Method = Self::Method>;
    type RouteError;

    fn recognize(
        &self,
        packet: &RouteHeader,
    ) -> Result<(&mut Self::Service, &Self::Method), Self::RouteError>;
}
*/

mod error {
    use prost;

    #[derive(Debug, Fail)]
    /// Error type related to executing logic trigger by an RPC request or response.
    pub enum RPCError {
        #[fail(display = "A malformed request for service {:} has been received", service_name)]
        /// Failure to process the request because of an unknown address.
        UnknownRequest {
            /// The name of the service that was addressed.
            service_name: &'static str,
        },

        #[fail(
            display = "A malformed request for service {:}, method {:}, was received",
            service_name,
            method_id
        )]
        /// Failure to process the request because it's not in the expected format.
        InvalidRequest {
            /// The name of the service that was addressed.
            service_name: &'static str,
            /// The method id that was addressed.
            method_id: u32,
        },

        #[fail(display = "The client sent an unrequested response, token {:}", token)]
        /// Failure to process the response because there was no known request linked to the token.
        InvalidResponse {
            /// The token as found in the response packet.
            token: u32,
        },

        #[fail(display = "Error while decoding a Protobuffer payload: {:}", _0)]
        /// Failure to construct an object from a proto message.
        ProtoDecode(#[cause] prost::DecodeError),

        #[fail(display = "Error while encoding a Protobuffer message: {:}", _0)]
        /// Failure to encode a proto message into a packet payload.
        ProtoEncode(#[cause] prost::EncodeError),
    }

    // Usability improvement
    impl From<prost::DecodeError> for RPCError {
        fn from(x: prost::DecodeError) -> Self {
            RPCError::ProtoDecode(x)
        }
    }

    // Usability improvement
    impl From<prost::EncodeError> for RPCError {
        fn from(x: prost::EncodeError) -> Self {
            RPCError::ProtoEncode(x)
        }
    }
}
