//! Important types for defining a router type which can process
//! RPC data.

use futures::prelude::*;
use bytes::Bytes;

use rpc::system::RPCError;
use rpc::transport::RPCPacket;

type BoxedRouterFuture = Box<Future<Item = Option<Bytes>, Error = RPCError>>;

#[allow(missing_docs)]
pub trait RPCRouter {
    type Packet: RPCPacket;
    type Method: 'static + Send;
    type Future: Future<Item = Option<Bytes>, Error = RPCError>;

    fn can_accept(packet: &Self::Packet) -> Result<&'static Self::Method, ()>;

    fn handle(&mut self, method: &Self::Method, packet: &Self::Packet) -> Self::Future;
}

#[allow(missing_docs)]
pub trait RPCHandling<'me, Packet>
where
    Packet: RPCPacket,
{
    fn route_packet(&'me mut self, packet: &'me Packet) -> Result<BoxedRouterFuture, ()>;
}

/*
#[allow(missing_docs)]
pub trait NewRouter<Service, Packet>
where
    Service: RPCService,
    <Service as RPCService>::Method: 'static + Send,
    Packet: RPCPacket,
{
    type Router: RPCRouter<Method = Service::Method, Packet = Packet>;

    fn from_service(&self) -> Self::Router;
}
*/

mod hlist_extensions {
    use super::*;

    use frunk::prelude::HList;
    use frunk::{HCons, HNil};

    impl<'me, Packet> RPCHandling<'me, Packet> for HNil
    where
        Packet: RPCPacket,
    {
        fn route_packet(&'me mut self, _: &'me Packet) -> Result<BoxedRouterFuture, ()> {
            Err(())
        }
    }

    impl<'me, Packet, X, Tail, Method> RPCHandling<'me, Packet> for HCons<X, Tail>
    where
        Packet: RPCPacket,
        X: RPCRouter<Packet = Packet, Method = Method, Future = BoxedRouterFuture>,
        Tail: RPCHandling<'me, Packet>,
        Method: 'static + Send,
    {
        fn route_packet(&'me mut self, packet: &'me Packet) -> Result<BoxedRouterFuture, ()> {
            let will_handle = X::can_accept(packet);
            if let Ok(method) = will_handle {
                let response = X::handle(&mut self.head, &method, packet);
                return Ok(response);
            }

            Tail::route_packet(&mut self.tail, packet)
        }
    }
}
