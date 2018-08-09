//! Important types for defining a router type which can process
//! RPC data.

use futures::prelude::*;

use rpc::system::{RPCError, RPCResult};
use rpc::transport::internal::InternalPacket;
use rpc::transport::RPCPacket;

type BoxedFuture<Item> = Box<Future<Item = Item, Error = RPCError> + Send + 'static>;

/// A result returned by RPC services.
pub enum ProcessResult<Packet> {
    /// Immediate result.
    Immediate(RouteDecision<Packet>),
    /// The result is still being calculated.
    /// Poll the inner future for completion.
    NotReady(BoxedFuture<RouteDecision<Packet>>),
}

/// Object that can be used by logic to communicate intent to
/// RPC compatible routers.
pub enum RouteDecision<Packet> {
    /// Stop routing.
    ///
    /// This effectively stops processing data on the current route.
    Stop,

    /// Returns the specified packet to the connected endpoint.
    Out(Packet),

    /// Internally route the provided packet to another service.
    Forward(InternalPacket),
}

#[allow(missing_docs)]
pub trait RPCRouter {
    type Request: RPCPacket;
    type Response: RPCPacket;
    type Method: 'static + Send;
    type SharedData;

    fn can_accept(packet: &Self::Request) -> RPCResult<&'static Self::Method>;

    fn handle(
        &mut self,
        method: &Self::Method,
        shared_data: &mut Self::SharedData,
        packet: &Self::Request,
    ) -> RPCResult<ProcessResult<Self::Response>>;
}

#[allow(missing_docs)]
pub trait RPCHandling<Data, Request, Response>
where
    Request: RPCPacket,
    Response: RPCPacket,
{
    fn route_packet(
        &mut self,
        shared_data: &mut Data,
        packet: &Request,
    ) -> RPCResult<ProcessResult<Response>>;
}

mod hlist_extensions {
    use super::*;

    use frunk::{HCons, HNil};

    impl<Data, Request, Response> RPCHandling<Data, Request, Response> for HNil
    where
        Request: RPCPacket,
        Response: RPCPacket,
    {
        fn route_packet(
            &mut self,
            _: &mut Data,
            _: &Request,
        ) -> RPCResult<ProcessResult<Response>> {
            Err(RPCError::NoRoute)
        }
    }

    impl<Data, Request, Response, X, Tail, Method> RPCHandling<Data, Request, Response>
        for HCons<X, Tail>
    where
        Request: RPCPacket,
        Response: RPCPacket,
        X: RPCRouter<Request = Request, Response = Response, Method = Method, SharedData = Data>,
        Tail: RPCHandling<Data, Request, Response>,
        Method: 'static + Send,
    {
        fn route_packet(
            &mut self,
            shared_data: &mut Data,
            packet: &Request,
        ) -> RPCResult<ProcessResult<Response>> {
            let will_handle = X::can_accept(packet);
            if let Ok(method) = will_handle {
                return X::handle(&mut self.head, &method, shared_data, packet);
            }

            Tail::route_packet(&mut self.tail, shared_data, packet)
        }
    }
}
