//! Module containing types which perform internal packet routing.
//!
//! Adressable services are collected and a message pump accepts and delivers
//! packets to these services.

use std::collections::VecDeque;
use std::default::Default;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use frunk::HNil;
use futures::future::{lazy, FutureResult};
use futures::prelude::*;
use slog;
use tokio_codec::Framed;
use tokio_tcp::TcpStream;

use protocol::bnet::frame::{BNetCodec, BNetPacket};
use protocol::bnet::session::SessionError;
use rpc::router::{RPCHandling, RouteDecision};
use rpc::system::{RPCError, RPCResult, ServiceBinderGenerator};
use rpc::transport::{Never, Request, Response};
use server::lobby::ServerShared;

#[derive(Debug)]
/// Structure containing important data related to the active client session.
pub struct ClientSharedData {
    server_shared: Arc<Mutex<ServerShared>>,
    logger: slog::Logger,
}

impl ClientSharedData {
    /// Retrieve the logger instance for this session.
    pub fn logger(&self) -> &slog::Logger {
        &self.logger
    }
}

#[allow(missing_docs)]
pub trait RouterBehaviour {
    fn handle_external_bnet(&mut self, packet: BNetPacket);
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct RoutingLogistic<BNReq, BNRes>
where
    BNReq: RPCHandling<ClientSharedData, Request<BNetPacket>, Response<BNetPacket>>,
    BNRes: RPCHandling<ClientSharedData, Response<BNetPacket>, Never>,
{
    bnet_request_handlers: BNReq,
    bnet_response_handlers: BNRes,

    codec: Framed<TcpStream, BNetCodec>,
    shared_data: ClientSharedData,
}

mod default {
    use super::*;

    use service::bnet::connection_service::ConnectionService;

    type DBNReq = Hlist![ConnectionService,];
    type DBNRes = Hlist![];

    impl RoutingLogistic<DBNReq, DBNRes> {
        /// Creates a new router with implemented service handlers.
        pub fn default_handlers(
            server_shared: Arc<Mutex<ServerShared>>,
            codec: Framed<TcpStream, BNetCodec>,
            logger: slog::Logger,
        ) -> Self {
            let shared_data = ClientSharedData {
                server_shared,
                logger,
            };
            let bnet_request_handlers = <DBNReq as ServiceBinderGenerator>::default();
            let bnet_response_handlers = <DBNRes as ServiceBinderGenerator>::default();
            RoutingLogistic::new(
                shared_data,
                codec,
                bnet_request_handlers,
                bnet_response_handlers,
            )
        }
    }
}

impl<BNReq, BNRes> RoutingLogistic<BNReq, BNRes>
where
    BNReq: RPCHandling<ClientSharedData, Request<BNetPacket>, Response<BNetPacket>>,
    BNRes: RPCHandling<ClientSharedData, Response<BNetPacket>, Never>,
{
    /// Creates a new object for routing packets between provided services.
    fn new(
        shared_data: ClientSharedData,
        codec: Framed<TcpStream, BNetCodec>,
        bnet_request_handlers: BNReq,
        bnet_response_handlers: BNRes,
    ) -> Self {
        RoutingLogistic {
            bnet_request_handlers,
            bnet_response_handlers,

            codec,
            shared_data,
        }
    }

    fn route_bnet_request(&mut self, request: Request<BNetPacket>) -> RPCResult<()> {
        let route_result = self
            .bnet_request_handlers
            .route_packet(&mut self.shared_data, &request);

        /*
            .and_then(move |bytes_opt| match bytes_opt {
                Some(body) => {
                    let response = Response::from_request(request, body);
                    Ok(RouteDecision::Out(response.unwrap()))
                }
                None => Ok(RouteDecision::Stop),
            })
        */
        unimplemented!()
    }

    fn route_bnet_response(&mut self, packet: Response<BNetPacket>) -> RPCResult<()> {
        let route_result = self
            .bnet_response_handlers
            .route_packet(&mut self.shared_data, &packet);

        /*
            .and_then(move |bytes_opt| {
                // TODO; Find out what to do with a response to a response!
                Ok(RouteDecision::Stop)
            })
        */
        unimplemented!()
    }

    fn handle_route_decision() {
        /*
        let current_future = self.bnet_response_queue.pop_front();
        match current_future {
            Some(future) => {
                let polled_result = future.poll()?;
                if let Async::Ready(route_decision) = polled_result {
                    match route_decision {
                        RouteDecision::Stop => {}
                        RouteDecision::Out(packet) => return Ok(Async::Ready(Some(packet))),
                        RouteDecision::Forward(_) => return Err(RPCError::NotImplemented),
                    }
                } else {
                    self.bnet_response_queue.push_front(current_future.unwrap());
                }
            }
            None => {}
        };
         */
        unimplemented!()
    }
}

impl<BNReq, BNRes> RouterBehaviour for RoutingLogistic<BNReq, BNRes>
where
    BNReq: RPCHandling<ClientSharedData, Request<BNetPacket>, Response<BNetPacket>>,
    BNRes: RPCHandling<ClientSharedData, Response<BNetPacket>, Never>,
{
    fn handle_external_bnet(&mut self, packet: BNetPacket) {
        let mut packet_opt = Some(packet);

        if let Some(packet) = packet_opt.take() {
            match packet.try_as_request() {
                Ok(request) => {
                    let response = self.route_bnet_request(request);
                    /*
                    self.bnet_response_queue.push_back(Box::new(response));
                    */
                }
                Err(packet) => packet_opt = Some(packet),
            };
        }

        if let Some(packet) = packet_opt.take() {
            match packet.try_as_response() {
                Ok(response) => {
                    let response = self.route_bnet_response(response);
                    /*
                    self.bnet_response_queue.push_back(Box::new(response));
                    */
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
    BNReq: RPCHandling<ClientSharedData, Request<BNetPacket>, Response<BNetPacket>>,
    BNRes: RPCHandling<ClientSharedData, Response<BNetPacket>, Never>,
{
    type Item = ();
    type Error = SessionError;

    fn poll(&mut self) -> Poll<(), SessionError> {
        // Pull external data.
        while let Async::Ready(bnet_packet_opt) = self.codec.poll()? {
            if let Some(bnet_packet) = bnet_packet_opt {
                self.handle_external_bnet(bnet_packet);
            } else {
                return Ok(Async::Ready(()));
            }
        }

        Ok(Async::NotReady)
    }
}
