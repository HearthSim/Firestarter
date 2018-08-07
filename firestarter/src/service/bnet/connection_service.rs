//! Service handling RPC requests/responses that manipulate the connection between
//! client and server.

use bytes::Bytes;
use futures::future::lazy;
use futures::prelude::*;

use protocol::bnet::frame::BNetPacket;
use protocol::bnet::session::{ClientSharedData, LightWeightSession};
use rpc::router::RPCRouter;
use rpc::system::{RPCError, RPCService, ServiceBinder, ServiceHash};
use rpc::transport::Request;

#[allow(missing_docs)]
#[repr(u32)]
#[derive(Debug, Copy, Clone)]
/// Addressable methods for this service.
pub enum Methods {
    Connect = 1,
    Bind = 2,
    Echo = 3,
    ForceDisconnect = 4,
    KeepAlive = 5,
    Encrypt = 6,
    RequestDisconnect = 7,
}

struct Inner {}

/// Service handling RPC requests/responses that manipulate the connection between
/// client and server.
///
/// See the module documentation for more information.
pub struct ConnectionService(Inner);

impl RPCService for ConnectionService {
    type Method = Methods;

    fn get_hash() -> ServiceHash {
        ServiceHash::from_name(Self::get_name())
    }

    fn get_name() -> &'static str {
        "bnet.protocol.connection.ConnectionService"
    }

    fn get_methods() -> &'static [(&'static Methods, &'static str)] {
        &[
            (&Methods::Connect, "Connect"),
            (&Methods::Bind, "Bind"),
            (&Methods::Echo, "Echo"),
            (&Methods::ForceDisconnect, "ForceDisconnect"),
            (&Methods::KeepAlive, "KeepAlive"),
            (&Methods::Encrypt, "Encrypt"),
            (&Methods::RequestDisconnect, "RequestDisconnect"),
        ]
    }
}

impl ServiceBinder for ConnectionService {
    type Service = Self;

    fn bind() -> Self {
        ConnectionService(Inner {})
    }
}

impl RPCRouter for ConnectionService {
    type Packet = Request<BNetPacket>;
    type Method = Methods;
    type SharedData = ClientSharedData;
    type Future = Box<Future<Item = Option<Bytes>, Error = RPCError>>;

    fn can_accept(packet: &Request<BNetPacket>) -> Result<&'static Methods, ()> {
        let packet_header = packet.as_ref().unwrap().header();
        let packet_service_hash = packet_header.service_id;
        let packet_method_id = packet_header.method_id.ok_or(())?;

        if packet_service_hash != ConnectionService::get_hash().as_uint() {
            return Err(());
        }

        ConnectionService::get_methods()
            .iter()
            .filter(|&(&method, _)| method as u32 == packet_method_id)
            .map(|&(method, _)| method)
            .next()
            .ok_or(())
    }

    fn handle(
        &mut self,
        method: &Methods,
        data: &mut ClientSharedData,
        packet: &Request<BNetPacket>,
    ) -> Self::Future {
        let packet = packet.as_ref().unwrap();
        let payload = packet.body().clone();
        let state = &mut self.0;

        match method {
            Methods::Connect => Box::new(op_connect(state, payload)),
            _ => unimplemented!(),
        }
    }
}

fn validate_connect_request<'a>(
    packet: &'a Request<BNetPacket>,
) -> impl Future<Item = (), Error = RPCError> {
    let packet_data = packet.as_ref().unwrap();
    let header = packet_data.header();
    let method_test = header.method_id.ok_or_else(|| RPCError::UnknownRequest {
        service_name: ConnectionService::get_name(),
    });

    let result = match method_test {
        Ok(method) if method == (Methods::Connect as u32) => Ok(()),
        Ok(method) => Err(RPCError::InvalidRequest {
            service_name: ConnectionService::get_name(),
            method_id: Methods::Connect as u32,
        }),
        Err(e) => Err(e),
    };

    lazy(move || result)
}

/// Attempts to perform the connect operation on a lightweight client session.
pub fn lightweight_session_connect<'a>(
    session: LightWeightSession,
    packet: &'a Request<BNetPacket>,
) -> impl Future<Item = (LightWeightSession, Option<Bytes>), Error = RPCError> {
    let payload = packet.as_ref().unwrap().body().clone();

    validate_connect_request(packet)
        .and_then(move |_| op_connect(&mut Inner {}, payload))
        .map(move |response| (session, response))
}

fn op_connect(
    state: &mut Inner,
    payload: Bytes,
) -> impl Future<Item = Option<Bytes>, Error = RPCError> {
    lazy(|| Err(RPCError::NotImplemented))
}
