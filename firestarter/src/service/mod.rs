//! Module containing network available service implementations.
//!
//! A client can invoke these services through RPC requests.
//! Internal service routing will also be supported, so services themselves
//! can bounce requests to other services.

pub mod bnet;
