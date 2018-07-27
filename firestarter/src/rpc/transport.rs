//! Types wich semantically constrain meaning of data.

#[derive(Debug)]
/// Represents an RPC request.
pub struct Request<Packet>(Packet);

impl<Packet> Request<Packet> {
    /// Wraps a packet into a request.
    pub fn new(data: Packet) -> Self {
        Request(data)
    }

    /// Extract the packet from this object.
    pub fn into_inner(self) -> Packet {
        self.0
    }

    /// Create a request with a reference to the containing
    /// packet.
    pub fn as_ref(&self) -> Request<&Packet> {
        Request::new(&self.0)
    }
}

#[derive(Debug)]
/// Represents an RPC response.
pub struct Response<Packet>(Packet);

impl<Packet> Response<Packet> {
    /// Wraps a packet into a response.
    pub fn new(data: Packet) -> Self {
        Response(data)
    }

    /// Extracts the packet from this object.
    pub fn into_inner(self) -> Packet {
        self.0
    }

    /// Create a response with a reference to the containing
    /// packet.
    pub fn as_ref(&self) -> Response<&Packet> {
        Response::new(&self.0)
    }
}
