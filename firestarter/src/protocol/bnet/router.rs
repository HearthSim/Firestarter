//! Module containing types which perform internal packet routing.
//!
//! Adressable services are collected and a message pump accepts and delivers
//! packets to these services.

use std::default::Default;
use std::marker::PhantomData;

use frunk::HNil;
use futures::prelude::*;
use futures::future::{lazy, FutureResult};
use bytes::Bytes;


use protocol::bnet::frame::BNetPacket;
use rpc::router::RPCHandling;
use rpc::system::{RPCError, ServiceBinderGenerator};
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
            let bnet_request_handlers = <DBNReq as ServiceBinderGenerator>::default();
            let bnet_response_handlers = <DBNRes as ServiceBinderGenerator>::default();
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

    fn route_bnet_request(&mut self, packet: Request<BNetPacket>) -> impl Future<Item = Option<Bytes>, Error = RPCError> {
    	let handling_result = self.bnet_request_handlers.route_packet(&packet);
        lazy(|| -> FutureResult<Option<Bytes>, RPCError> { unimplemented!() })
    }

    fn route_bnet_response(&mut self, packet: Response<BNetPacket>) -> impl Future<Item = Option<Bytes>, Error = RPCError> {
    	let handling_result = self.bnet_response_handlers.route_packet(&packet);
        lazy(|| -> FutureResult<Option<Bytes>, RPCError> { unimplemented!() })
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
        let mut packet_opt = Some(packet);

        if let Some(packet) = packet_opt {
            match packet.try_as_request() {
                Ok(request) => {
                	return self.route_bnet_request(request);
                }
                Err(packet) => packet_opt = Some(packet),
            };
        }

        if let Some(packet) = packet_opt {
            match packet.try_as_response() {
                Ok(response) => {
                    return self.route_bnet_response(response);
                }
                Err(packet) => packet_opt = Some(packet),
            };
        }

        if let Some(packet) = packet_opt {
            // TODO Error handling because packet doesn't match request/response layout.
        }
    }
}
