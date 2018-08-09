#![allow(missing_docs)]

use bytes::BytesMut;
use num_traits::AsPrimitive;
use prost::Message;

use protocol::bnet::frame::BNetPacket;
use protocol::bnet::router::ClientSharedData;
use rpc::router::{ProcessResult, RPCRouter, RouteDecision};
use rpc::system::{RPCError, RPCResult, RPCService, ServiceBinder, ServiceHash};
use rpc::transport::{Request, Response};
use service::bnet::service_info::ExportedServiceID;
use service::bnet::util::default_accept_check;

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
/// Addressable methods for this service.
pub enum Methods {
    Logon = 1,
    ModuleNotify = 2,
    ModuleMessage = 3,
    SelectGameAccountDeprecated = 4,
    GenerateSSOToken = 5,
    SelectGameAccount = 6,
    VerifyWebCredentials = 7,
}

impl AsPrimitive<u32> for Methods {
    fn as_(self) -> u32 {
        self as u32
    }
}

struct Inner {}

pub struct AuthenticationServer(Inner);

impl RPCService for AuthenticationServer {
    type Method = Methods;

    fn get_hash() -> ServiceHash {
        ServiceHash::from_name(Self::get_name())
    }

    fn get_id() -> u32 {
        ExportedServiceID::AuthenticationServer as u32
    }

    fn get_name() -> &'static str {
        "bnet.protocol.authentication.AuthenticationServer"
    }

    fn get_methods() -> &'static [(&'static Methods, &'static str)] {
        &[
            (&Methods::Logon, "Logon"),
            (&Methods::ModuleNotify, "ModuleNotify"),
            (&Methods::ModuleMessage, "ModuleMessage"),
            (
                &Methods::SelectGameAccountDeprecated,
                "SelectGameAccountDeprecated",
            ),
            (&Methods::GenerateSSOToken, "GenerateSSOToken"),
            (&Methods::SelectGameAccount, "SelectGameAccount"),
            (&Methods::VerifyWebCredentials, "VerifyWebCredentials"),
        ]
    }
}

impl ServiceBinder for AuthenticationServer {
    type Service = Self;

    fn bind() -> Self {
        AuthenticationServer(Inner {})
    }
}

impl RPCRouter for AuthenticationServer {
    type Request = Request<BNetPacket>;
    type Response = Response<BNetPacket>;
    type Method = Methods;
    type SharedData = ClientSharedData;

    fn can_accept(packet: &Request<BNetPacket>) -> RPCResult<&'static Methods> {
        // EXPLAIN: Asserted Request<&X> contains exactly one packet.
        let packet_header = packet.as_ref().unwrap().header();
        default_accept_check::<Self>(packet_header)
    }

    fn handle(
        &mut self,
        method: &Methods,
        data: &mut ClientSharedData,
        packet: Request<BNetPacket>,
    ) -> RPCResult<ProcessResult<Response<BNetPacket>>> {
        let state = &mut self.0;

        match method {
            Methods::Logon => op_logon(state, data, packet)
                .map(RouteDecision::Out)
                .map(ProcessResult::Immediate),
            _ => unimplemented!(),
        }
    }
}

fn op_logon(
    state: &mut Inner,
    shared: &mut ClientSharedData,
    request: Request<BNetPacket>,
) -> RPCResult<Response<BNetPacket>> {
    use firestarter_generated::proto::bnet::protocol::authentication::{LogonRequest, LogonResult};

    let payload = request.as_ref().unwrap().body().clone();
    let message = LogonRequest::decode(payload)?;
    trace!(shared.logger(), "Logon request"; "message" => ?message);

    Ok(Response::empty(request))
}
