//! Adapter methods to connect the logging infrastructure of this library with the one from
//! the consumer.
//!
//! # Caution
//! It's highly recommended to manually provide a compatible logger when executing code
//! from this crate. These methods should only be used as fallback or to provide concise
//! example code.
//! Please see the [`server`] module for specific information.
//!
//! # Note
//! Logging in this library is done through the [slog crate][].
//!
//! [slog crate]: https://github.com/slog-rs/slog

use slog;
use slog::Drain;
use slog_stdlog;

/// A root logger instance that can be used when library consumers didn't explicitly
/// passed one into this library.
pub fn default_logger() -> slog::Logger {
    from_std_log()
}

/// A root logger instance using the [log crate][] as drain.
///
/// # Note
/// The [log crate][] is only a facade so only provides macros and structs which accept
/// log messages. These items require a [log backend][], code that handles outputting these messages
/// to the screen/disk/network.
///
/// [log crate]: https://github.com/rust-lang-nursery/log
/// [log backend]: https://docs.rs/log/0.4.3/log/#available-logging-implementations
pub fn from_std_log() -> slog::Logger {
    slog::Logger::root(slog_stdlog::StdLog.fuse(), o!())
}
