use futures::prelude::*;
use std::net::SocketAddr;

use protocol::bnet::session::SessionError;
use protocol::bnet::util::hash_item;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default)]
/// Object representing an unique identifier for a client.
pub struct ClientHash(u64);

impl ClientHash {
    /// Create a new identifier from the provided socket address.
    pub fn from_socket_address(address: SocketAddr) -> Self {
        ClientHash(hash_item(address))
    }

    /// Converts the current hash value into an unsigned integer.
    pub fn as_uint(self) -> u64 {
        self.0
    }
}

#[derive(Debug)]
/// A complete user session.
///
/// This structure contains the necessary data to properly communicate with a specific client.
pub struct ClientSession<Router>
where
    Router: Future<Error = SessionError>,
{
    address: SocketAddr,
    router: Router,
}

impl<Router> ClientSession<Router>
where
    Router: Future<Error = SessionError>,
{
    /// Creates a new session object for the connected client.
    pub fn new(address: SocketAddr, router: Router) -> Self {
        ClientSession { address, router }
    }
}

impl<Router> Future for ClientSession<Router>
where
    Router: Future<Error = SessionError>,
{
    type Item = ();
    type Error = SessionError;

    fn poll(&mut self) -> Poll<(), SessionError> {
        // Activate message pump of router.
        let _ = try_ready!(self.router.poll());

        // TODO; Do other stuff..
        // Here is where you would execute post session logic.

        Ok(Async::Ready(()))
    }
}
