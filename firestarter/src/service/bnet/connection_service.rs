//! Service handling RPC requests/responses that manipulate the connection between
//! client and server.

use bytes::{Bytes, BytesMut};

use prost::Message;
use futures::future::{lazy, FutureResult};
use futures::prelude::*;

use protocol::bnet::frame::BNetPacket;
use protocol::bnet::session::LightWeightSession;
use rpc::system::{RPCError, RPCObject, RPCService, ServiceHash};
use rpc::transport::Request;

#[derive(Debug, Default)]
/// Service handling RPC requests/responses that manipulate the connection between
/// client and server.
///
/// See the module documentation for more information.
pub struct ConnectionService {}

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

impl ConnectionService {
    // TODO: Replace with general Service-like approach.
    const SERVICE_NAME: &'static str = stringify!(ConnectionService);

    /// See [`Methods::Connect`]
    pub const METHOD_CONNECT: Methods = Methods::Connect;
    /// See [`Methods::Bind`]
    pub const METHOD_BIND: Methods = Methods::Bind;
    /// See [`Methods::Echo`]
    pub const METHOD_ECHO: Methods = Methods::Echo;
    /// See [`Methods::ForceDisconnect`]
    pub const METHOD_FORCE_DISCONNECT: Methods = Methods::ForceDisconnect;
    /// See [`Methods::KeepAlive`]
    pub const METHOD_KEEP_ALIVE: Methods = Methods::KeepAlive;
    /// See [`Methods::Encrypt`]
    pub const METHOD_ENCRYPT: Methods = Methods::Encrypt;
    /// See [`Methods::RequestDisconnect`]
    pub const METHOD_REQUEST_DISCONNECT: Methods = Methods::RequestDisconnect;
}

impl RPCService for ConnectionService {
    type Method = Methods;

    fn get_hash() -> ServiceHash {
        ServiceHash::from_name(Self::get_name())
    }

    fn get_name() -> &'static str {
        Self::SERVICE_NAME
    }

    fn get_methods() -> &'static [(&'static str, &'static Methods)] {
        &[
            ("Connect", &Self::METHOD_CONNECT),
            ("Bind", &Self::METHOD_BIND),
            ("Echo", &Self::METHOD_ECHO),
            ("ForceDisconnect", &Self::METHOD_FORCE_DISCONNECT),
            ("KeepAlive", &Self::METHOD_KEEP_ALIVE),
            ("Encrypt", &Self::METHOD_ENCRYPT),
            ("RequestDisconnect", &Self::METHOD_REQUEST_DISCONNECT),
        ]
    }
}

/// TODO
pub fn connect_operation(
    packet: &Request<BNetPacket>,
) -> impl Future<Item = Option<Bytes>, Error = RPCError> {
    lazy(|| -> FutureResult<Option<Bytes>, RPCError> { unimplemented!() })
}

impl ConnectionService {
    // Validates if the provided request is ACTUALLY a ConnectRequest
    fn is_connect_request<'a>(packet: &'a Request<BNetPacket>) -> Result<(), RPCError> {
        let ref packet_data = packet.as_ref().unwrap();
        let header = packet_data.header();
        let method = header.method_id.ok_or(RPCError::UnknownRequest {
            service_name: Self::SERVICE_NAME,
        })?;
        if method == (Methods::Connect as u32) {
            return Ok(());
        }

        Err(RPCError::InvalidRequest {
            service_name: Self::SERVICE_NAME,
            method_id: Methods::Connect as u32,
        })
    }

    /// Handles a direct connect request without going through routing and service handling.
    ///
    /// This method can be used to directly handshake with a client, without side-effects.
    pub fn connect_direct<'a>(
        session: LightWeightSession,
        request: &'a Request<BNetPacket>,
    ) -> impl Future<Item = (LightWeightSession, Option<Bytes>), Error = RPCError> {
        use chrono::Local;
        use firestarter_generated::proto::bnet::protocol::connection::{
            BindRequest, BindResponse, ConnectRequest, ConnectResponse,
        };
        use firestarter_generated::proto::bnet::protocol::ProcessId;
        use service::bnet::service_info::{SERVICES_EXPORTED_BINDING, SERVICES_IMPORTED_BINDING};

        // 'static values are created from borrows. These satisfy the closure requirement when moved
        // into them.
        let is_valid = Self::is_connect_request(request);
        // Note: It's safe to unwrap because Request<> only has one variant at the moment!
        let packet_body = request.as_ref().unwrap().body().clone();

        lazy(move || {
            is_valid?;
            let message = ConnectRequest::decode(packet_body)?;
            trace!(session.logger(), "Handshake request"; "message" => ?message);

            let bind_request = message.bind_request;
            if bind_request.is_none() {
                Err(RPCError::InvalidRequest {
                    service_name: Self::SERVICE_NAME,
                    method_id: Self::METHOD_CONNECT as u32,
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
                    service_name: Self::SERVICE_NAME,
                    method_id: Self::METHOD_CONNECT as u32,
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

            trace!(session.logger(), "Handshake response ready"; "message" => ?response_message);
            let mut body = BytesMut::new();
            body.reserve(response_message.encoded_len());
            response_message.encode(&mut body)?;

            let response = Some(body.freeze());
            Ok((session, response))
        })
    }
}
