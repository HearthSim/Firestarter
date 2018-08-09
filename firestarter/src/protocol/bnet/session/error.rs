use protocol::bnet::frame::CodecError;
use rpc::system::RPCError;

#[derive(Debug, Fail)]
/// Error type related to a client session.
pub enum SessionError {
    #[fail(display = "Client disconnected")]
    /// Session failure because of client disconnect.
    ClientDisconnect,

    #[fail(display = "Client didn't send expected request")]
    /// Session failure because of faulty client communications.
    MissingRequest,

    #[fail(display = "A passed deadline triggered a timeout on the connection")]
    /// The client did not responsd (properly) within the set deadline.
    Timeout,

    #[fail(display = "{}", _0)]
    /// Session failure due to malformed data.
    Codec(#[cause] CodecError),

    #[fail(display = "{}", _0)]
    /// Session failure due to a service error.
    RPC(#[cause] RPCError),
}

// Usability improvement
impl From<CodecError> for SessionError {
    fn from(x: CodecError) -> Self {
        SessionError::Codec(x)
    }
}

// Usability improvement
impl From<RPCError> for SessionError {
    fn from(x: RPCError) -> Self {
        SessionError::RPC(x)
    }
}
