//! Firestarter
//!
//! HS server simulation framework.

#![deny(missing_docs)]
// Awaiting a fix for the following issue: https://github.com/rust-lang/rust/issues/24584
// `cargo test` generates a binary from the lib and each test case. Binaries are NOT
// checked for missing documentation.
// #![cfg_attr(test,deny(missing_docs))]

#[macro_use]
extern crate typed_builder;
#[macro_use]
extern crate slog;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate error_chain;

extern crate bytes;
extern crate futures;
extern crate prost;
extern crate slog_stdlog;
extern crate tokio;
extern crate tokio_codec;
extern crate tokio_executor;
extern crate tokio_tcp;
extern crate tokio_timer;

extern crate firestarter_generated;

pub mod log;
pub mod protocol;
pub mod rpc;
pub mod server;
pub mod service;

pub use self::error::*;

mod error {
    error_chain! {
        errors {}

        links {}

        foreign_links {}
    }
}
