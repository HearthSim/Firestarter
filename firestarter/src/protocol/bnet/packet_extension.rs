//! Additionall methods for operating on [`BNetPacket`]s.

use bytes::Bytes;

use protocol::bnet::frame::BNetPacket;
use rpc::transport::{Request, Response};

impl BNetPacket {
    /// Try to parse a [`Request`] from this packet.
    pub fn try_as_request(self) -> Result<Request<Self>, Self> {
        unimplemented!()
    }

    /// Try to parse a [`Response`] from this packet.
    pub fn try_as_response(self) -> Result<Response<Self>, Self> {
        unimplemented!()
    }
}

impl Response<BNetPacket> {
    /// Build a packet which is a direct [`Response`] to the mentioned [`Request`].
    pub fn from_request(request: Request<BNetPacket>, body: Bytes) -> Self {
        unimplemented!()
    }
}
