#![deny(missing_docs)]
// Awaiting a fix for the following issue: https://github.com/rust-lang/rust/issues/24584
// `cargo test` generates a binary from the lib and each test case. Binaries are NOT
// checked for missing documentation.
// #![cfg_attr(test,deny(missing_docs))]

//! Firestarter project
