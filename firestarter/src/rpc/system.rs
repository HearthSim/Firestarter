//! Important types for defining an RPC service.

pub use self::error::*;

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
}
