//! Important types for defining a router type which can process
//! RPC data.

use bytes::Bytes;
use futures::prelude::*;

use rpc::system::RPCError;
use rpc::transport::RPCPacket;

type BoxedRouterFuture = Box<Future<Item = Option<Bytes>, Error = RPCError>>;

#[allow(missing_docs)]
pub trait RPCRouter {
    type Packet: RPCPacket;
    type Method: 'static + Send;
    type SharedData;
    type Future: Future<Item = Option<Bytes>, Error = RPCError>;

    fn can_accept(packet: &Self::Packet) -> Result<&'static Self::Method, ()>;

    fn handle(
        &mut self,
        method: &Self::Method,
        shared_data: &mut Self::SharedData,
        packet: &Self::Packet,
    ) -> Self::Future;
}

#[allow(missing_docs)]
pub trait RPCHandling<Data, Packet>
where
    Packet: RPCPacket,
{
    fn route_packet(
        &mut self,
        shared_data: &mut Data,
        packet: &Packet,
    ) -> Result<BoxedRouterFuture, ()>;
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

    impl<Data, Packet> RPCHandling<Data, Packet> for HNil
    where
        Packet: RPCPacket,
    {
        fn route_packet(&mut self, _: &mut Data, _: &Packet) -> Result<BoxedRouterFuture, ()> {
            Err(())
        }
    }

    impl<Data, Packet, X, Tail, Method> RPCHandling<Data, Packet> for HCons<X, Tail>
    where
        Packet: RPCPacket,
        X: RPCRouter<
            Packet = Packet,
            Method = Method,
            SharedData = Data,
            Future = BoxedRouterFuture,
        >,
        Tail: RPCHandling<Data, Packet>,
        Method: 'static + Send,
    {
        fn route_packet(
            &mut self,
            shared_data: &mut Data,
            packet: &Packet,
        ) -> Result<BoxedRouterFuture, ()> {
            let will_handle = X::can_accept(packet);
            if let Ok(method) = will_handle {
                let response = X::handle(&mut self.head, &method, shared_data, packet);
                return Ok(response);
            }

            Tail::route_packet(&mut self.tail, shared_data, packet)
        }
    }
}
