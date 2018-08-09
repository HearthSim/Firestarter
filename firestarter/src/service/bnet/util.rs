//! Module containing utilities for working with BNET RPC
//! services.

use num_traits::AsPrimitive;

use firestarter_generated::proto::bnet::protocol::Header;
use rpc::system::{RPCError, RPCResult, RPCService};

/// Tests if the BNET header is valid for the provided BNET service.
pub fn default_accept_check<Service: RPCService>(
    header: &Header,
) -> RPCResult<&'static Service::Method> {
    let packet_service_id = header.service_id;
    let packet_method_id = header.method_id.ok_or(RPCError::UnknownRequest {
        service_name: Service::get_name(),
    })?;

    if packet_service_id != Service::get_id() {
        return Err(RPCError::UnknownRequest {
            service_name: Service::get_name(),
        });
    }

    Service::get_methods()
        .iter()
        .filter(|&(&method, _)| method.as_() == packet_method_id)
        .map(|&(method, _)| method)
        .next()
        .ok_or(RPCError::UnknownRequest {
            service_name: Service::get_name(),
        })
}
