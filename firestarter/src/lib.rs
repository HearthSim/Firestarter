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

extern crate slog_stdlog;
extern crate tokio_tcp;

pub mod log;
pub mod server;
