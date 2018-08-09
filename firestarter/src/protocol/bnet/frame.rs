//! Contains a codec to encode/decode protocol objects into/from a byte stream.

use bytes::{BigEndian, BufMut, ByteOrder, Bytes, BytesMut};
use firestarter_generated::proto::bnet::protocol::Header;
use prost::Message;
use tokio_codec::{Decoder, Encoder};

pub use self::error::*;

#[derive(Debug)]
/// The object that is sent between client/server.
///
/// Both client and server must be BNet protocol compatible to handle
/// this object.
pub struct BNetPacket {
    header: Header,
    body: Bytes,
}

impl BNetPacket {
    /// Constructs a new BNetPacket.
    pub fn new(header: Header, body: Bytes) -> Self {
        Self { header, body }
    }

    /// Retrieve the header of this packet.
    ///
    /// The header contains meta-information like addressing, error codes
    /// and more.
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Retrieve the payload of this packet.
    ///
    /// The actual meaning is unclear and is up to the service handler
    /// to figure out.
    pub fn body(&self) -> &Bytes {
        &self.body
    }

    /// Deconstruct this packet into a seperate header and payload object.
    pub fn split(self) -> (Header, Bytes) {
        let BNetPacket { header, body } = self;
        (header, body)
    }
}

#[derive(Debug)]
/// Object performing the conversion from and into a byte stream for
/// the protocol object, [`BNetPacket`].
pub struct BNetCodec {
    header: Option<Header>,
    header_length: Option<u16>,
    body_size: Option<u32>,
}

impl BNetCodec {
    /// Create a new codec.
    pub fn new() -> Self {
        Self {
            header: None,
            header_length: None,
            body_size: None,
        }
    }
}

// 2 bytes long preamble, interpreted as U16BE
const HEADER_PREAMBLE_LENGTH: usize = 2;

impl Encoder for BNetCodec {
    type Item = BNetPacket;
    type Error = CodecError;

    fn encode(&mut self, item: BNetPacket, destination: &mut BytesMut) -> Result<(), CodecError> {
        let (header, body) = item.split();
        let frame_length = HEADER_PREAMBLE_LENGTH + header.encoded_len() + body.len();
        destination.reserve(frame_length);

        // Cast by dropping overflowing bits
        let header_preamble = header.encoded_len() as u16;
        destination.put_u16_be(header_preamble);
        header.encode(destination)?;
        destination.put(body);

        Ok(())
    }
}

impl Decoder for BNetCodec {
    type Item = BNetPacket;
    type Error = CodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<BNetPacket>, CodecError> {
        if self.header_length.is_none() {
            if src.len() < 2 {
                // Return indicating more data is necessary.
                return Ok(None);
            }

            let preamble_buf = src.split_to(HEADER_PREAMBLE_LENGTH).freeze();
            // TLS detection
            // https://stackoverflow.com/a/10355804
            match (preamble_buf[0], preamble_buf[1]) {
                (0x16, version) if version <= 0x03 => return Err(CodecError::TLSEnabled)?,
                _ => {}
            }

            let header_length = BigEndian::read_u16(&preamble_buf);
            self.header_length = Some(header_length);
        }

        if self.header.is_none() {
            let header_length = self.header_length.unwrap() as usize;
            if src.len() < header_length {
                return Ok(None);
            }

            let header_buf = src.split_to(header_length).freeze();
            let header = Header::decode(header_buf)?;

            let body_size = header.size.ok_or_else(|| CodecError::MissingData {
                field_name: "Size".into(),
            })?;
            self.body_size = Some(body_size);
            self.header = Some(header);
        }

        if let Some(body_size) = self.body_size {
            let body_size = body_size as usize;
            if src.len() >= body_size {
                let header = self.header.take().unwrap();
                let body = src.split_to(body_size).freeze();
                let _ = self.header_length.take();

                let packet = BNetPacket::new(header, body);
                return Ok(Some(packet));
            }
        }

        // More bytes required
        Ok(None)
    }
}

mod error {
    use prost;
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

        #[fail(display = "Error while decoding a Protobuffer wire-stream: {:}", _0)]
        /// Failure to construct frame due to malformed data.
        ProtoDecode(#[cause] prost::DecodeError),

        #[fail(display = "Error while encoding a Protobuffer message: {:}", _0)]
        /// Failure to encode a frame into a bytestream.
        ProtoEncode(#[cause] prost::EncodeError),
    }

    // Implementation necessary as per constraint from Encoder::Error + Decoder::Error
    impl From<io::Error> for CodecError {
        fn from(x: io::Error) -> Self {
            CodecError::Io(x)
        }
    }

    // Usability improvement
    impl From<prost::DecodeError> for CodecError {
        fn from(x: prost::DecodeError) -> Self {
            CodecError::ProtoDecode(x)
        }
    }

    // Usability improvement
    impl From<prost::EncodeError> for CodecError {
        fn from(x: prost::EncodeError) -> Self {
            CodecError::ProtoEncode(x)
        }
    }
}
