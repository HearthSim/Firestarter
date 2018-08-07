//! Module containing types which perform internal packet routing.
//!
//! Adressable services are collected and a message pump accepts and delivers
//! packets to these services.

use std::default::Default;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use frunk::HNil;
use futures::future::{lazy, FutureResult};
use futures::prelude::*;

use protocol::bnet::frame::BNetPacket;
use protocol::bnet::session::ClientSharedData;
use rpc::router::RPCHandling;
use rpc::system::{RPCError, ServiceBinderGenerator};
use rpc::transport::{Request, Response};

#[allow(missing_docs)]
pub trait RouterBehaviour: Future<Item = (), Error = RPCError> {
    fn handle_external_bnet(&mut self, packet: BNetPacket);
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct RoutingLogistic<BNReq, BNRes>
where
    BNReq: RPCHandling<ClientSharedData, Request<BNetPacket>>,
    BNRes: RPCHandling<ClientSharedData, Response<BNetPacket>>,
{
    bnet_request_handlers: BNReq,
    bnet_response_handlers: BNRes,
    shared_data: ClientSharedData,
}

mod default {
    use super::*;

    use service::bnet::connection_service::ConnectionService;

    type DBNReq = Hlist![ConnectionService,];
    type DBNRes = Hlist![];

    impl RoutingLogistic<DBNReq, DBNRes> {
        /// Creates a new router with implemented service handlers.
        pub fn default_handlers(shared_data: ClientSharedData) -> Self {
            let bnet_request_handlers = <DBNReq as ServiceBinderGenerator>::default();
            let bnet_response_handlers = <DBNRes as ServiceBinderGenerator>::default();
            RoutingLogistic::new(shared_data, bnet_request_handlers, bnet_response_handlers)
        }
    }
}

impl<BNReq, BNRes> RoutingLogistic<BNReq, BNRes>
where
    BNReq: RPCHandling<ClientSharedData, Request<BNetPacket>>,
    BNRes: RPCHandling<ClientSharedData, Response<BNetPacket>>,
{
    /// Creates a new object for routing packets between provided services.
    fn new(
        shared_data: ClientSharedData,
        bnet_request_handlers: BNReq,
        bnet_response_handlers: BNRes,
    ) -> Self {
        RoutingLogistic {
            bnet_request_handlers,
            bnet_response_handlers,
            shared_data,
        }
    }

    fn route_bnet_request(
        &mut self,
        packet: Request<BNetPacket>,
    ) -> impl Future<Item = Option<Bytes>, Error = RPCError> {
        let handling_result = self
            .bnet_request_handlers
            .route_packet(&mut self.shared_data, &packet);
        lazy(|| -> FutureResult<Option<Bytes>, RPCError> { unimplemented!() })
    }

    fn route_bnet_response(
        &mut self,
        packet: Response<BNetPacket>,
    ) -> impl Future<Item = Option<Bytes>, Error = RPCError> {
        let handling_result = self
            .bnet_response_handlers
            .route_packet(&mut self.shared_data, &packet);
        lazy(|| -> FutureResult<Option<Bytes>, RPCError> { unimplemented!() })
    }
}

impl<BNReq, BNRes> RouterBehaviour for RoutingLogistic<BNReq, BNRes>
where
    BNReq: RPCHandling<ClientSharedData, Request<BNetPacket>>,
    BNRes: RPCHandling<ClientSharedData, Response<BNetPacket>>,
{
    fn handle_external_bnet(&mut self, packet: BNetPacket) {
        let mut packet_opt = Some(packet);

        if let Some(packet) = packet_opt.take() {
            match packet.try_as_request() {
                Ok(request) => {
                    self.route_bnet_request(request);
                }
                Err(packet) => packet_opt = Some(packet),
            };
        }

        if let Some(packet) = packet_opt.take() {
            match packet.try_as_response() {
                Ok(response) => {
                    self.route_bnet_response(response);
                }
                Err(packet) => packet_opt = Some(packet),
            };
        }

        if let Some(packet) = packet_opt {
            // TODO Error handling because packet doesn't match request/response layout.
        }
    }
}

impl<BNReq, BNRes> Future for RoutingLogistic<BNReq, BNRes>
where
    BNReq: RPCHandling<ClientSharedData, Request<BNetPacket>>,
    BNRes: RPCHandling<ClientSharedData, Response<BNetPacket>>,
{
    type Item = ();
    type Error = RPCError;

    fn poll(&mut self) -> Poll<(), RPCError> {
        // Pump message queue.
        unimplemented!()
    }
}
