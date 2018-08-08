//! Important types for defining an RPC service.
use bytes::Bytes;
use failure::Error;
use futures::prelude::*;

use rpc::transport::{RPCPacket, Request, Response};
use rpc::util::fnv_hash_bytes;

pub use self::error::*;

/// Result type for RPC related methods.
pub type RPCResult<Item> = Result<Item, RPCError>;

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

    /// Converts the current hash into an unsigned integer value.
    pub fn as_uint(self) -> u32 {
        self.0
    }
}

#[allow(missing_docs)]
pub trait RPCService {
    type Method: 'static + Sized;

    fn get_hash() -> ServiceHash;
    fn get_name() -> &'static str;
    fn get_methods() -> &'static [(&'static Self::Method, &'static str)];
}

#[allow(missing_docs)]
pub trait ServiceBinder {
    type Service: RPCService;

    fn bind() -> Self::Service;
}

#[allow(missing_docs)]
pub trait ServiceBinderGenerator {
    fn default() -> Self;
}

mod hlist_extensions {
    use super::*;

    use frunk::prelude::HList;
    use frunk::{HCons, HNil};

    impl ServiceBinderGenerator for HNil {
        fn default() -> HNil {
            HNil
        }
    }

    impl<X, Tail> ServiceBinderGenerator for HCons<X, Tail>
    where
        X: ServiceBinder<Service = X> + RPCService,
        Tail: ServiceBinderGenerator,
    {
        fn default() -> Self {
            HCons {
                head: X::bind(),
                tail: Tail::default(),
            }
        }
    }
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

        #[fail(display = "There was no route found for this provided packet")]
        /// Failure to process data because no route was found for the provided packet.
        NoRoute,

        #[fail(display = "Unimplemented feature")]
        /// Failure to process data because an unimplemented feature was reached.
        NotImplemented,

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
