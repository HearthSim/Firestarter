//! Types wich semantically constrain meaning of data.

/// Marker trait signifying that a packet is compatible with the RPC system.
pub trait RPCPacket {}
impl<'a, X: RPCPacket> RPCPacket for &'a X {}

/// Stable type which replaces the NEVER type primitive.
// TODO; Remove this when never (!) is stabilized.
pub enum Never {}
impl RPCPacket for Never {}

#[derive(Debug)]
/// Represents an RPC request.
pub struct Request<Packet>(Packet);

impl<Packet> RPCPacket for Request<Packet> {}

impl<Packet> Request<Packet> {
    /// Wraps a packet into a request.
    pub fn new(data: Packet) -> Self {
        Request(data)
    }

    /// Take the original packet out of the Request wrapper.
    pub fn unwrap(self) -> Packet {
        self.0
    }

    /// Create a request with a reference to the containing
    /// packet.
    pub fn as_ref(&self) -> Request<&Packet> {
        Request(&self.0)
    }
}

#[derive(Debug)]
// TODO; Turn back into enum!
/// Represents an RPC response.
pub struct Response<Packet>(Packet);

impl<Packet> RPCPacket for Response<Packet> {}

impl<Packet> Response<Packet> {
    /// Wraps a packet into a response.
    pub fn new(data: Packet) -> Self {
        Response(data)
    }

    /// Take the original packet out of the Response wrapper.
    ///
    /// # Panics
    /// This method panics if the variant is Response::None!
    pub fn unwrap(self) -> Packet {
        self.0
    }

    /// Create a response with a reference to the containing
    /// packet.
    pub fn as_ref(&self) -> Response<&Packet> {
        Response(&self.0)
    }
}

/// Module containing all types for internally routing requests/responses.
pub mod internal {
    use bytes::Bytes;
    use rpc::system::ServiceHash;

    #[derive(Debug)]
    /// Data linking a request and a response together.
    ///
    /// Note: This structure currently doesn't expose fields because
    /// it's not clear what the purpose of a token could be.
    pub struct RouteToken {
        id: u32,
    }

    impl RouteToken {
        /// Creates a new token with the provided ID.
        pub fn new(id: u32) -> Self {
            RouteToken { id }
        }
    }

    /// Structure used to make routing decisions for accompanying payloads.
    pub struct RouteHeader {
        /// The hash of the service that's being addressed.
        pub service_hash: ServiceHash,
        /// ID of the method that's being addressed.
        pub method_id: u32,
        /// TODO
        pub token: Option<RouteToken>,
    }

    /// Structure for internally routing requests between services.
    pub struct InternalPacket {
        /// The header of this packet. This value contains meta-information
        /// about the packet.
        pub header: RouteHeader,
        /// The actual data contained by this packet.
        pub payload: Bytes,
    }
}
