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

/// Accepts the chosen (packet) request format and produces the provided response.
///
/// Use the 'can_accept' method to validate which address to use when calling 'handle'.
pub trait RPCRouter {
    /// The dataformat that's accepted for handling.
    type Request: RPCPacket;
    /// The dataformat that's returned by the implemented method.
    type Response: RPCPacket;
    /// Type to address a specific RPC method.
    type Method: 'static + Send;
    /// The dataformat that's shared between all handlers of the same client session.
    type SharedData;

    /// Tests if this router can (and will) accept the provided request.
    ///
    /// This method also returns the specific method address that will accept
    /// the data as provided to this method.
    fn can_accept(packet: &Self::Request) -> RPCResult<&'static Self::Method>;

    /// Proces the provided request and build a response.
    ///
    /// The response is either calculated immediately or returned as a future.
    /// The caller is responsible for polling the future in order to retrieve the final result.
    fn handle(
        &mut self,
        method: &Self::Method,
        shared_data: &mut Self::SharedData,
        packet: Self::Request,
    ) -> RPCResult<ProcessResult<Self::Response>>;
}

/// Trait for containers of [`RPCRouter`] instances which share the same
/// Request and Response (packet) format.
///
/// Implementers eagerly try to get an appropriate response from one of its routers.
/// The first valid response encountered will be returned instantly, ignoring the result
/// of all other routers.
///
/// # Note
/// The order in which all routers are polled is dependant on the implementation.
pub trait RPCHandling<Data, Request, Response>
where
    Request: RPCPacket,
    Response: RPCPacket,
{
    /// Attempt to route the provided packet to a compatible handler.
    fn route_packet(
        &mut self,
        shared_data: &mut Data,
        packet: Request,
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
        fn route_packet(&mut self, _: &mut Data, _: Request) -> RPCResult<ProcessResult<Response>> {
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
            packet: Request,
        ) -> RPCResult<ProcessResult<Response>> {
            let will_handle = X::can_accept(&packet);
            if let Ok(method) = will_handle {
                return X::handle(&mut self.head, &method, shared_data, packet);
            }

            Tail::route_packet(&mut self.tail, shared_data, packet)
        }
    }
}
