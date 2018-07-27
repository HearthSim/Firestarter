#![doc(hidden)]

use protocol::bnet::frame::BNetPacket;
use rpc::transport::{Request, Response};

impl BNetPacket {
    pub fn try_as_request(self) -> Result<Request<Self>, Self> {
        unimplemented!()
    }

    pub fn try_as_response(self) -> Result<Response<Self>, Self> {
        unimplemented!()
    }
}
