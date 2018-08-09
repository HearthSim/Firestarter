//! Module containing data types for controlling the
//! session that's setup between server and client(s).

mod error;
mod session_full;
mod session_light;

// All public types are re-exported under this
// module.
// Less repeating 'session' is less tedious!
pub use self::error::*;
pub use self::session_full::*;
pub use self::session_light::*;
