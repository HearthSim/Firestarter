//! Important types for defining an RPC service.
use bytes::Bytes;
use failure::Error;
use futures::prelude::*;

use rpc::transport::{RPCPacket, Request, Response, RouteHeader};
use rpc::util::fnv_hash_bytes;

pub use self::error::*;

#[allow(missing_docs)]
pub type BoxedRPCResult<Error> = Box<Future<Item = Option<Bytes>, Error = Error>>;

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

    fn get_hash() -> ServiceHash;
    fn get_name() -> &'static str;
    fn get_methods() -> &'static [(&'static str, &'static Self::Method)];
}

#[allow(missing_docs)]
pub trait RPCObject {
    type Error: Into<Error>;

    type Packet: RPCPacket;
    type Future: Future<Item = Option<Bytes>, Error = Self::Error>;

    fn call(&mut self, packet: &Self::Packet) -> Self::Future;
}

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
