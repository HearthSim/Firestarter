//! Module containing types which perform internal packet routing.
//!
//! Adressable services are collected and a message pump accepts and delivers
//! packets to these services.

use std::collections::VecDeque;
use std::default::Default;
use std::fmt;
use std::sync::{Arc, Mutex};

use futures::prelude::*;
use slog;
use tokio_codec::Framed;
use tokio_tcp::TcpStream;

use protocol::bnet::frame::{BNetCodec, BNetPacket};
use protocol::bnet::session::SessionError;
use rpc::router::{ProcessResult, RPCHandling, RouteDecision};
use rpc::system::{RPCError, RPCResult, ServiceBinderGenerator};
use rpc::transport::{Request, Response};
use server::lobby::ServerShared;

type BNetBlockinOperation =
    Box<Future<Item = RouteDecision<Response<BNetPacket>>, Error = RPCError> + Send>;

#[derive(Debug)]
/// Structure containing important data related to the active client session.
pub struct ClientSharedData {
    server_shared: Arc<Mutex<ServerShared>>,
    logger: slog::Logger,
}

impl ClientSharedData {
    /// Creates a new instance of data which is shared accross all services.
    pub fn new(server_shared: Arc<Mutex<ServerShared>>, logger: slog::Logger) -> Self {
        ClientSharedData {
            server_shared,
            logger,
        }
    }

    /// Creates a stub instance with only an active logger.
    pub fn stub(logger: slog::Logger) -> Self {
        ClientSharedData {
            server_shared: Arc::new(Mutex::new(ServerShared::default())),
            logger: logger,
        }
    }

    /// Retrieve the logger instance for this session.
    pub fn logger(&self) -> &slog::Logger {
        &self.logger
    }
}

/// Object which routes packets between provided services.
///
/// This router object can process incoming packets from an external client and
/// packets matching the internal routing protocol.
pub struct RoutingLogistic<BNReq, BNRes>
where
    BNReq: RPCHandling<ClientSharedData, Request<BNetPacket>, Response<BNetPacket>>,
    BNRes: RPCHandling<ClientSharedData, Response<BNetPacket>, Response<BNetPacket>>,
{
    bnet_request_handlers: BNReq,
    bnet_response_handlers: BNRes,

    bnet_blocking_ops_queue: [Option<BNetBlockinOperation>; 2],
    queued_responses: VecDeque<Response<BNetPacket>>,

    codec: Framed<TcpStream, BNetCodec>,
    shared_data: ClientSharedData,
}

impl<BNReq, BNRes> Future for RoutingLogistic<BNReq, BNRes>
where
    BNReq: RPCHandling<ClientSharedData, Request<BNetPacket>, Response<BNetPacket>>,
    BNRes: RPCHandling<ClientSharedData, Response<BNetPacket>, Response<BNetPacket>>,
{
    type Item = ();
    type Error = SessionError;

    fn poll(&mut self) -> Poll<(), SessionError> {
        // Poll blocked future.
        self.process_blocking_operations()?;

        // The idea is to process maximum one blocking operation at a time
        // while allowing non-blocking operations to get through, so we keep
        // reading from the socket..
        // It's possible a next packet will be blocking as well, in that case
        // we buffer that specific packet and pause this entire session if that
        // buffer is full!
        //
        // Note: This concept has the implicit requirement that building a future
        // is cheap or can be optimized away when we cannot accept another blocking
        // operation.
        if !Self::queue_room_available(&self.bnet_blocking_ops_queue) {
            return Ok(Async::NotReady);
        }

        // Only pull new packets if there is room to store potential blocking ones.
        'receive: loop {
            if Self::queue_room_available(&self.bnet_blocking_ops_queue) {
                if let Async::Ready(bnet_packet_opt) = self.codec.poll()? {
                    match bnet_packet_opt {
                        None => {
                            // TODO; Find out if we want to gracefully shut down
                            // instead of immediately dropping everything.
                            return Ok(Async::Ready(()));
                        }
                        Some(packet) => {
                            self.process_external_bnet(packet)?;
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

impl<BNReq, BNRes> RoutingLogistic<BNReq, BNRes>
where
    BNReq: RPCHandling<ClientSharedData, Request<BNetPacket>, Response<BNetPacket>>,
    BNRes: RPCHandling<ClientSharedData, Response<BNetPacket>, Response<BNetPacket>>,
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

            bnet_blocking_ops_queue: Default::default(),
            queued_responses: VecDeque::with_capacity(2),

            codec,
            shared_data,
        }
    }

    fn queue_room_available<'me, I, T>(storage: I) -> bool
    where
        I: IntoIterator<Item = &'me Option<T>>,
        T: 'me,
    {
        storage.into_iter().filter(|x| Option::is_none(x)).count() > 0
    }

    fn push_to_queue<'me, I, T>(storage: I, item: T) -> Result<(), T>
    where
        I: IntoIterator<Item = &'me mut Option<T>>,
        T: 'me,
    {
        match storage
            .into_iter()
            .skip_while(|x| Option::is_none(x))
            .next()
        {
            Some(open_spot) => {
                *open_spot = Some(item);
                return Ok(());
            }
            None => return Err(item),
        };
    }

    fn process_blocking_operations(&mut self) -> Result<(), SessionError> {
        // Poll all blocking operations.
        let poll_results: Result<Vec<(_, _)>, _> = self
            .bnet_blocking_ops_queue
            .iter_mut()
            .enumerate()
            .filter(|(idx, x)| Option::is_some(x))
            .map(|(idx, op)| match Future::poll(op.as_mut().unwrap()) {
                Ok(data) => Ok((idx, data)),
                Err(e) => Err(e),
            })
            .collect();

        // Remove all operations which are ready.
        // Also process the data results.
        let poll_data = poll_results?.into_iter().filter_map(|(idx, x)| match x {
            Async::Ready(x) => Some((idx, x)),
            Async::NotReady => None,
        });

        for (idx, decision) in poll_data {
            let _ = self.bnet_blocking_ops_queue.get_mut(idx).take();
            self.handle_bnet_response_decision(decision);
        }

        Ok(())
    }

    fn process_external_bnet(&mut self, packet: BNetPacket) -> Result<(), SessionError> {
        let mut packet_opt = Some(packet);

        if let Some(packet) = packet_opt.take() {
            match packet.try_as_request() {
                Ok(request) => {
                    self.route_bnet_request(request)?;
                }
                Err(packet) => packet_opt = Some(packet),
            };
        }

        if let Some(packet) = packet_opt.take() {
            match packet.try_as_response() {
                Ok(response) => {
                    self.route_bnet_response(response)?;
                }
                Err(packet) => packet_opt = Some(packet),
            };
        }

        // We have a potentially malformed packet if we reach this statement.
        let _ = packet_opt.take();
        Err(RPCError::NoRoute)?
    }

    fn route_bnet_request(&mut self, request: Request<BNetPacket>) -> RPCResult<()> {
        let route_result = self
            .bnet_request_handlers
            .route_packet(&mut self.shared_data, request);

        match route_result? {
            ProcessResult::Immediate(decision) => self.handle_bnet_response_decision(decision),
            ProcessResult::NotReady(blocking_operation) => {
                // If this method fails, the current operation will be dropped.
                // The wrapping logic MUST make sure nothing gets dropped!
                if let Err(_) =
                    Self::push_to_queue(&mut self.bnet_blocking_ops_queue, blocking_operation)
                {
                    unreachable!();
                }
            }
        };

        Ok(())
    }

    fn route_bnet_response(&mut self, response: Response<BNetPacket>) -> RPCResult<()> {
        let route_result = self
            .bnet_response_handlers
            .route_packet(&mut self.shared_data, response);

        match route_result? {
            ProcessResult::Immediate(decision) => self.handle_bnet_response_decision(decision),
            ProcessResult::NotReady(blocking_operation) => {
                // If this method fails, the current operation will be dropped.
                // The wrapping logic MUST make sure nothing gets dropped!
                if let Err(_) =
                    Self::push_to_queue(&mut self.bnet_blocking_ops_queue, blocking_operation)
                {
                    unreachable!();
                }
            }
        };

        Ok(())
    }

    fn handle_bnet_response_decision(&mut self, decision: RouteDecision<Response<BNetPacket>>) {
        match decision {
            RouteDecision::Stop => {}
            RouteDecision::Out(packet) => {
                self.queued_responses.push_back(packet);
            }
            RouteDecision::Forward(packet) => {
                unimplemented!();
            }
        };
    }
}

impl<BNReq, BNRes> fmt::Debug for RoutingLogistic<BNReq, BNRes>
where
    BNReq: RPCHandling<ClientSharedData, Request<BNetPacket>, Response<BNetPacket>> + fmt::Debug,
    BNRes: RPCHandling<ClientSharedData, Response<BNetPacket>, Response<BNetPacket>> + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RoutingLogistic")
            .field("bnet_request_handlers", &self.bnet_request_handlers)
            .field("bnet_response_handlers", &self.bnet_response_handlers)
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
