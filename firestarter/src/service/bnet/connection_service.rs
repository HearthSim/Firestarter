//! Service handling RPC requests/responses that manipulate the connection between
//! client and server.

use bytes::BytesMut;
use prost::Message;

use protocol::bnet::frame::BNetPacket;
use protocol::bnet::router::ClientSharedData;
use protocol::bnet::session::LightWeightSession;
use rpc::router::{ProcessResult, RPCRouter, RouteDecision};
use rpc::system::{RPCError, RPCResult, RPCService, ServiceBinder, ServiceHash};
use rpc::transport::{Request, Response};

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
    type Request = Request<BNetPacket>;
    type Response = Response<BNetPacket>;
    type Method = Methods;
    type SharedData = ClientSharedData;

    fn can_accept(packet: &Request<BNetPacket>) -> RPCResult<&'static Methods> {
        let packet_header = packet.as_ref().unwrap().header();
        let packet_service_hash = packet_header.service_id;
        let packet_method_id = packet_header.method_id.ok_or(RPCError::UnknownRequest {
            service_name: Self::get_name(),
        })?;

        if packet_service_hash != ConnectionService::get_hash().as_uint() {
            return Err(RPCError::UnknownRequest {
                service_name: Self::get_name(),
            });
        }

        ConnectionService::get_methods()
            .iter()
            .filter(|&(&method, _)| method as u32 == packet_method_id)
            .map(|&(method, _)| method)
            .next()
            .ok_or(RPCError::UnknownRequest {
                service_name: Self::get_name(),
            })
    }

    fn handle(
        &mut self,
        method: &Methods,
        data: &mut ClientSharedData,
        packet: Request<BNetPacket>,
    ) -> RPCResult<ProcessResult<Response<BNetPacket>>> {
        let state = &mut self.0;

        match method {
            Methods::Connect => op_connect(state, data, packet)
                .map(RouteDecision::Out)
                .map(ProcessResult::Immediate),
            _ => unimplemented!(),
        }
    }
}

fn validate_connect_request<'a>(packet: &'a Request<BNetPacket>) -> RPCResult<()> {
    let packet_data = packet.as_ref().unwrap();
    let header = packet_data.header();
    let method_test = header.method_id.ok_or_else(|| RPCError::UnknownRequest {
        service_name: ConnectionService::get_name(),
    });

    let result = match method_test {
        Ok(method) if method == (Methods::Connect as u32) => Ok(()),
        Ok(_) => Err(RPCError::InvalidRequest {
            service_name: ConnectionService::get_name(),
            method_id: Methods::Connect as u32,
        }),
        Err(e) => Err(e),
    };

    result
}

/// Attempts to perform the connect operation on a lightweight client session.
pub fn lightweight_session_connect(
    session: &LightWeightSession,
    packet: Request<BNetPacket>,
) -> RPCResult<Response<BNetPacket>> {
    let _ = validate_connect_request(&packet)?;
    let logger = session.logger().clone();
    op_connect(&mut Inner {}, &mut ClientSharedData::stub(logger), packet)
}

fn op_connect(
    state: &mut Inner,
    shared: &mut ClientSharedData,
    request: Request<BNetPacket>,
) -> RPCResult<Response<BNetPacket>> {
    use chrono::Local;
    use firestarter_generated::proto::bnet::protocol::connection::{
        BindRequest, BindResponse, ConnectRequest, ConnectResponse,
    };
    use firestarter_generated::proto::bnet::protocol::ProcessId;
    use service::bnet::service_info::{SERVICES_EXPORTED_BINDING, SERVICES_IMPORTED_BINDING};

    let payload = request.as_ref().unwrap().body().clone();
    let message = ConnectRequest::decode(payload)?;
    trace!(shared.logger(), "Handshake request"; "message" => ?message);

    let bind_request = message.bind_request;
    if bind_request.is_none() {
        Err(RPCError::InvalidRequest {
            service_name: ConnectionService::get_name(),
            method_id: Methods::Connect as u32,
        })?;
    }

    // This destructuring is probably difficult to understand.
    // We're receiving the BindRequest from the client perspective;
    // this means that any service that is "imported" is EXPORTED by us.
    // Analogue for "exported", which is IMPORTED by us.
    //
    // The comments below will explain building a response in the
    // perspective of the client.
    let BindRequest {
        imported_service_hash: exported_services,
        exported_service: imported_services,
    } = bind_request.unwrap();

    // Match all imported service IDs with our info.
    let match_imported_services = imported_services.into_iter().all(|s| {
        // Find the service for the provided hash.
        let known_import_opt = SERVICES_IMPORTED_BINDING.get(&s.hash).map(|m| (*m) as u32);
        if let Some(id) = known_import_opt {
            if id == s.id {
                return true;
            }
        }
        false
    });
    if !match_imported_services {
        Err(RPCError::InvalidRequest {
            service_name: ConnectionService::get_name(),
            method_id: Methods::Connect as u32,
        })?;
    }

    // Build a mapping for our exported services according to the service info.
    let service_bindings: Vec<u32> = exported_services
        .into_iter()
        .map(|hash| {
            SERVICES_EXPORTED_BINDING
                .get(&hash)
                .map(|m| (*m) as u32)
                .unwrap_or(0)
        })
        .collect();
    let bind_response = BindResponse {
        imported_service_id: service_bindings,
    };

    // Start collecting all data into a response.
    let time = Local::now().timestamp();
    let precise_time = Local::now().timestamp_nanos();
    let response_message = ConnectResponse {
        server_id: ProcessId {
            label: 3868510373,
            epoch: time as u32,
        },
        client_id: Some(ProcessId {
            label: 1255760,
            epoch: time as u32,
        }),
        bind_result: Some(0),
        bind_response: Some(bind_response),
        server_time: Some(precise_time as u64),
        ..Default::default()
    };

    trace!(shared.logger(), "Handshake response ready"; "message" => ?response_message);
    let mut body = BytesMut::with_capacity(response_message.encoded_len());
    response_message.encode(&mut body).unwrap();
    let body = body.freeze();

    Ok(Response::from_request(request, body))
}
