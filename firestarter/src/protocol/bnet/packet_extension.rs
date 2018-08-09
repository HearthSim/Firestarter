//! Additionall methods for operating on [`BNetPacket`]s.

use bytes::{Bytes, BytesMut};
use prost::Message;

use protocol::bnet::frame::BNetPacket;
use rpc::transport::{RPCPacket, Request, Response};

use firestarter_generated::proto::bnet::protocol::{Header, NoData};

const RESPONSE_SERVICE_ID: u32 = 254;
const RESPONSE_METHOD_ID: u32 = 0;

impl RPCPacket for BNetPacket {}

impl BNetPacket {
    /// Try to parse a [`Request`] from this packet.
    pub fn try_as_request(self) -> Result<Request<Self>, Self> {
        match self.try_as_response() {
            // EXPLAIN: Asserted Response<X> contains exactly one packet.
            Ok(response) => Err(response.unwrap()),
            Err(packet) => Ok(Request::new(packet)),
        }
    }

    /// Try to parse a [`Response`] from this packet.
    pub fn try_as_response(self) -> Result<Response<Self>, Self> {
        let service = self.header().service_id;
        let method = self.header().method_id;
        match (service, method) {
            (s, m) if s == RESPONSE_SERVICE_ID && m.unwrap_or(0) == RESPONSE_METHOD_ID => {
                Ok(Response::new(self))
            }
            _ => Err(self),
        }
    }
}

impl Response<BNetPacket> {
    /// Build a packet which is a direct [`Response`] to the mentioned [`Request`].
    pub fn from_request(request: Request<BNetPacket>, body: Bytes) -> Self {
        // EXPLAIN: Asserted Request<X> contains exactly one packet.
        let request = request.unwrap();
        let Header { token, .. } = request.header();

        let response_header = Header {
            service_id: RESPONSE_SERVICE_ID,
            method_id: Some(RESPONSE_METHOD_ID),
            token: *token,
            size: Some(body.len() as u32),
            ..Default::default()
        };

        let packet = BNetPacket::new(response_header, body);
        Response::new(packet)
    }

    /// Builds an empty response packet for the given request.
    pub fn empty(request: Request<BNetPacket>) -> Self {
        // EXPLAIN: Asserted Request<X> contains exactly one packet.
        let request = request.unwrap();
        let Header { token, .. } = request.header();

        let payload = NoData {};
        let mut body = BytesMut::with_capacity(payload.encoded_len());
        // EXPLAIN: Asserted there is enough room in the buffer.
        payload.encode(&mut body).unwrap();
        let body = body.freeze();

        let response_header = Header {
            service_id: RESPONSE_SERVICE_ID,
            method_id: Some(RESPONSE_METHOD_ID),
            token: *token,
            size: Some(body.len() as u32),
            ..Default::default()
        };

        let packet = BNetPacket::new(response_header, body);
        Response::new(packet)
    }
}
