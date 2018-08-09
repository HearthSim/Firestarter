//! Important types for defining an RPC service.

use num_traits::AsPrimitive;

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

/// An object that exposes itself through the RPC interface.
///
/// An RPC service is addressable through its hash, see [`ServiceHash`].
/// Its methods are addressable through the provided Method object, which
/// is an enum most of the time. (An enum constraint cannot be expressed
/// currently).
pub trait RPCService {
    /// Type used for addressing service methods.
    ///
    /// Most of the time this is an enumeration, but that requirement cannot
    /// be expressed currently.
    type Method: AsPrimitive<u32> + 'static + Sized;

    /// Retrieve the unique hash of this service.
    ///
    /// The hash can be used to send requests to this service.
    fn get_hash() -> ServiceHash;

    /// Retrieves the ordinal to which this service is bound.
    fn get_id() -> u32;

    /// Retrieve the name of this services.
    fn get_name() -> &'static str;

    /// Retrieve all addressable RPC methods attached to this service.
    fn get_methods() -> &'static [(&'static Self::Method, &'static str)];
}

/// The ability to create a new RPC service object.
pub trait ServiceBinder {
    /// The type of service instance that will be generated.
    type Service: RPCService;

    /// Creates a new service instance.
    fn bind() -> Self::Service;
}

/// Behaviour intended for containers of RPC service objects.
///
/// Implementations should constrain their children with the [`ServiceBinder`]
/// trait to properly build the objects itself.
pub trait ServiceBinderGenerator {
    /// Generate a bunch of services with the default parameters.
    fn default() -> Self;
}

mod hlist_extensions {
    use super::*;

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
