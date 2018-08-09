//! Module containing types which perform internal packet routing.
//!
//! Adressable services are collected and a message pump accepts and delivers
//! packets to these services.

use std::collections::VecDeque;
use std::default::Default;
use std::fmt;
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
use rpc::transport::internal::InternalPacket;
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
pub struct RoutingLogistic<BNReq, BNRes>
where
    BNReq: RPCHandling<ClientSharedData, Request<BNetPacket>, Response<BNetPacket>>,
    BNRes: RPCHandling<ClientSharedData, Response<BNetPacket>, Never>,
{
    bnet_request_handlers: BNReq,
    bnet_response_handlers: BNRes,

    bnet_blocking_buffer: Option<BNetPacket>,
    bnet_blocked_response:
        Option<Box<Future<Item = RouteDecision<Response<BNetPacket>>, Error = RPCError> + Send>>,
    queued_responses: VecDeque<Response<BNetPacket>>,

    codec: Framed<TcpStream, BNetCodec>,
    shared_data: ClientSharedData,
}

impl<BNReq, BNRes> RoutingLogistic<BNReq, BNRes>
where
    BNReq: RPCHandling<ClientSharedData, Request<BNetPacket>, Response<BNetPacket>>,
    BNRes: RPCHandling<ClientSharedData, Response<BNetPacket>, Never>,
{
    fn poll_blocking_operation(&mut self) -> Poll<(), SessionError> {
        let mut ready_decision = None;
        if let Some(blocked) = self.bnet_blocked_response.as_mut() {
            ready_decision = Some(try_ready!(blocked.poll()));
        }

        // Seperate block is necessary because we cannot take out the future
        // while holding a borrow to it.
        // Until NLL arrives.
        if let Some(decision) = ready_decision {
            let _ = self.bnet_blocked_response.take();

            match decision {
                RouteDecision::Stop => {}
                RouteDecision::Out(packet) => {
                    self.queued_responses.push_back(packet);
                }
                RouteDecision::Forward(packet) => unimplemented!(),
            };
        }

        Ok(Async::Ready(()))
    }

    fn attempt_drain_blocking_buffer(&mut self) -> Result<(), SessionError> {
        if self.bnet_blocked_response.is_none() {
            if let Some(packet) = self.bnet_blocking_buffer.take() {
                self.handle_external_bnet(packet)?;
            }
        }

        Ok(())
    }

    fn handle_external_bnet(&mut self, packet: BNetPacket) -> Result<(), SessionError> {
        let mut packet_opt = Some(packet);

        if let Some(packet) = packet_opt.take() {
            match packet.try_as_request() {
                Ok(request) => {
                    let response = self.route_bnet_request(request);
                    unimplemented!()
                }
                Err(packet) => packet_opt = Some(packet),
            };
        }

        if let Some(packet) = packet_opt.take() {
            match packet.try_as_response() {
                Ok(response) => {
                    let response = self.route_bnet_response(response);
                    unimplemented!()
                }
                Err(packet) => packet_opt = Some(packet),
            };
        }

        if let Some(packet) = packet_opt {
            Err(RPCError::NoRoute)?;
        }

        Ok(())
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
        // Poll blocked future.
        let blocked_operation_result = match self.poll_blocking_operation()? {
            Async::Ready(()) => {}
            Async::NotReady => {
                // The idea is to process maximum one blocking operation at a time
                // while allowing non-blocking operations to get through, so we keep
                // reading from the socket..
                // It's possible a next packet will be blocking as well, in that case
                // we buffer that specific packet and pause this entire session if that
                // buffer is full!
                if self.bnet_blocking_buffer.is_some() {
                    return Ok(Async::NotReady);
                }
            }
        };

        // Process next blocking packet.
        self.attempt_drain_blocking_buffer()?;

        // Only pull new packets if there is room to store potential blocking ones.
        'receive: loop {
            if self.bnet_blocking_buffer.is_none() {
                if let Async::Ready(bnet_packet_opt) = self.codec.poll()? {
                    match bnet_packet_opt {
                        None => {
                            // TODO; Find out if we want to gracefully shut down
                            // instead of immediately dropping everything.
                            return Ok(Async::Ready(()));
                        }
                        Some(packet) => {
                            let result = self.handle_external_bnet(packet);
                            unimplemented!();

                            // Explicitly continue the loop because the default case
                            // is breaking.
                            continue 'receive;
                        }
                    };
                }
            }

            // In the general case, break the loop.
            break;
        }

        // Push awaiting responses to the client.
        let mut has_written = false;
        let mut response_overflow = None;
        while let Some(response_packet) = self.queued_responses.pop_front() {
            match self.codec.start_send(response_packet.unwrap())? {
                AsyncSink::Ready => has_written = true,
                AsyncSink::NotReady(overflow) => {
                    response_overflow = Some(overflow);
                    break;
                }
            };
        }

        // Store overflowed packet at the first position in the queue
        // for the next attempt.
        if let Some(overflow) = response_overflow {
            self.queued_responses.push_front(Response::new(overflow));
        }

        if has_written {
            match self.codec.poll_complete()? {
                Async::Ready(_) => {}
                Async::NotReady => {
                    // TODO; Find out if another poll_complete at the start
                    // of the next loop is necessary.
                    // Normally the response queue would catch all overflow
                    // so errors won't happen.
                }
            };
        }

        // Either the codec doesn't have any new packets to process or there are
        // (multiple) blocking service handlers awaiting completion.
        Ok(Async::NotReady)
    }
}

impl<BNReq, BNRes> fmt::Debug for RoutingLogistic<BNReq, BNRes>
where
    BNReq: RPCHandling<ClientSharedData, Request<BNetPacket>, Response<BNetPacket>> + fmt::Debug,
    BNRes: RPCHandling<ClientSharedData, Response<BNetPacket>, Never> + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RoutingLogistic")
            .field("bnet_request_handlers", &self.bnet_request_handlers)
            .field("bnet_response_handlers", &self.bnet_response_handlers)
            .field("bnet_blocking_buffer", &self.bnet_blocking_buffer)
            .field("queued_responses", &self.queued_responses)
            .field("codec", &self.codec)
            .field("shared_data", &self.shared_data)
            .finish()
    }
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

            bnet_blocking_buffer: None,
            bnet_blocked_response: None,
            queued_responses: VecDeque::with_capacity(2),

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
