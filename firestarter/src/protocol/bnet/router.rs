//! Module containing types which perform internal packet routing.
//!
//! Adressable services are collected and a message pump accepts and delivers
//! packets to these services.

use std::default::Default;
use std::marker::PhantomData;

use frunk::HNil;

use protocol::bnet::frame::BNetPacket;
use rpc::router::RPCHandling;
use rpc::system::ServiceBinderGenerator;
use rpc::transport::{Request, Response};

#[allow(missing_docs)]
pub trait RouterBehaviour {
    fn pump_packets(&mut self);
    fn handle_external_bnet(&mut self, packet: BNetPacket);
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct RoutingLogistic<'logistics, BNReq, BNRes>
where
    BNReq: RPCHandling<'logistics, Request<BNetPacket>>,
    BNRes: RPCHandling<'logistics, Response<BNetPacket>>,
{
    bnet_request_handlers: BNReq,
    bnet_response_handlers: BNRes,
    _phantom: PhantomData<&'logistics ()>,
}

mod default {
    use super::*;
    use std::default::Default;

    use service::bnet::connection_service::ConnectionService;

    type DBNReq = Hlist![ConnectionService,];
    type DBNRes = Hlist![];

    impl<'a> Default for RoutingLogistic<'a, DBNReq, DBNRes> {
        fn default() -> Self {
            let bnet_request_handlers = ServiceBinderGenerator::default();
            let bnet_response_handlers = ServiceBinderGenerator::default();
            RoutingLogistic::new(bnet_request_handlers, bnet_response_handlers)
        }
    }
}

impl<'logistics, BNReq, BNRes> RoutingLogistic<'logistics, BNReq, BNRes>
where
    BNReq: RPCHandling<'logistics, Request<BNetPacket>>,
    BNRes: RPCHandling<'logistics, Response<BNetPacket>>,
{
	/// Creates a new object for routing packets between provided services.
    pub fn new(bnet_request_handlers: BNReq, bnet_response_handlers: BNRes) -> Self {
        RoutingLogistic {
            bnet_request_handlers,
            bnet_response_handlers,
            _phantom: PhantomData,
        }
    }
}

impl<'logistics, BNReq, BNRes> RouterBehaviour for RoutingLogistic<'logistics, BNReq, BNRes>
where
    BNReq: RPCHandling<'logistics, Request<BNetPacket>>,
    BNRes: RPCHandling<'logistics, Response<BNetPacket>>,
{
    fn pump_packets(&mut self) {
        unimplemented!()
    }

    fn handle_external_bnet(&mut self, packet: BNetPacket) {
        unimplemented!()
    }
}
