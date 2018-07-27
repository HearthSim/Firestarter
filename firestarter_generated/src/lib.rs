//! Crate containing build-time resources.

extern crate prost;

#[macro_use]
extern crate prost_derive;

#[allow(dead_code)]
pub mod proto {
    // Provisioning is done by the build script.
    include!(concat!(env!("OUT_DIR"), "/concat_compiled_proto.rs"));
}
