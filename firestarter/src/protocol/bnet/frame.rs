//! Contains a codec to encode/decode protocol objects into/from a byte stream.

use bytes::BytesMut;
use tokio_codec::{Decoder, Encoder};

pub use self::error::*;

#[derive(Debug)]
/// The object that is sent between client/server.
///
/// Both client and server must be BNet protocol compatible to handle
/// this object.
pub struct BNetPacket {}

#[derive(Debug)]
/// Object performing the conversion from and into a byte stream for
/// the protocol object, [`BNetPacket`].
pub struct BNetCodec {}

impl BNetCodec {
    /// Create a new codec.
    pub fn new() -> Self {
        Self {}
    }
}

impl Encoder for BNetCodec {
    type Item = BNetPacket;
    type Error = CodecError;

    fn encode(&mut self, _item: BNetPacket, _destination: &mut BytesMut) -> Result<(), CodecError> {
        unimplemented!()
    }
}

impl Decoder for BNetCodec {
    type Item = BNetPacket;
    type Error = CodecError;

    fn decode(&mut self, _src: &mut BytesMut) -> Result<Option<BNetPacket>, CodecError> {
        unimplemented!()
    }
}

mod error {
    use std::io;

    #[derive(Debug, Fail)]
    /// Error type related to encoding/decoding bytes to frame objects.
    pub enum CodecError {
        #[fail(display = "The client is trying to do a TLS handshake")]
        /// Failure to construct frame because of encryption.
        TLSEnabled,

        #[fail(display = "Required data is missing from a frame, field {:}", field_name)]
        /// Failure to construct frame because of missing data.
        MissingData {
            /// The name of the field with empty value.
            field_name: String,
        },

        #[fail(display = "{}", _0)]
        /// Failure to construct frame due to some input/output related error.
        Io(#[cause] io::Error),
    }

    // Implementation necessary as per constraint from Encoder::Error + Decoder::Error
    impl From<io::Error> for CodecError {
        fn from(x: io::Error) -> Self {
            CodecError::Io(x)
        }
    }
}
