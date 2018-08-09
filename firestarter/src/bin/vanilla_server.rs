extern crate dotenv;
extern crate failure;
extern crate firestarter;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_json;
extern crate slog_term;

use dotenv::dotenv;
use slog::Drain;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::OpenOptions;
use std::net::SocketAddr;
use std::path::Path;

use firestarter::server::lobby;

const KEY_SERVER_MOUNT: &str = "SERVER_ADDRESS";
const KEY_LOG_PATH: &str = "LOG_FILEPATH";

const DEFAULT_SERVER_MOUNT: &str = "127.0.0.1:1119";
const DEFAULT_LOG_PATH: &str = "./server.log";

fn main() -> Result<(), failure::Error> {
    // Read environment variables from directory structure.
    dotenv().ok();

    // Load required environment variables (or defaults).
    let server_mount: SocketAddr = env::var(KEY_SERVER_MOUNT)
        .unwrap_or_else(|_| String::from(DEFAULT_SERVER_MOUNT))
        .parse()?;
    let log_path: OsString =
        env::var_os(KEY_LOG_PATH).unwrap_or_else(|| OsString::from(OsStr::new(DEFAULT_LOG_PATH)));
    let log_path = Path::new(&log_path);

    // Setup file logger
    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)?;
    let file_logger = slog_json::Json::default(log_file);

    // Setup console logging
    let console_decorator = slog_term::TermDecorator::new().build();
    let console_logger = slog_term::CompactFormat::new(console_decorator).build();

    // Create root logger
    // Note: Each logger can have a filter applied.
    // The idea is to print everything to the terminal, but write out the more important
    // events to a file.
    // This approach causes storage media to fill slowly while the terminal simply cycles
    // through its buffer, favoring newer messages.
    let multiplex_logger = slog::Duplicate::new(
        slog::LevelFilter::new(console_logger, slog::Level::Trace),
        slog::LevelFilter::new(file_logger, slog::Level::Info),
    );
    let async_logger = slog_async::Async::new(multiplex_logger.ignore_res()).build();

    // Note: The default feature configuration of SLog disables trace logging.
    // Make sure to set the correct feature according to your general filter preferences
    // within Cargo.toml of this package!
    let root_logger = slog::Logger::root(async_logger.fuse(), o!());

    /* Prepare for launching the server */

    // Allow the server to retry binding to the mount point.
    // If binding fails, it's allowed to try the next port.
    // This results in the server being bound to one of the following ports
    // [1119; 1124[ , depending on which ports are available and which are not.
    let retry_config = lobby::BindRetryConfig::builder()
        .max_retries(5)
        .try_next_port(true)
        .build();

    // Configuration details for the server itself.
    let config = lobby::ServerConfig::builder()
        .bind_address(server_mount)
        .bind_fallback(retry_config)
        .logger(root_logger)
        .build();

    // Build server and 'just run' it.
    // This uses the default Tokio runtime, which uses a threadpool executor and
    // a reactor on a seperate thread.
    //
    // Note: This method will block the current thread until the reactor is completely
    // shut down.
    // See [`LobbyServer::split`] for a [`ServerHandle`] which can control the server
    // while it's running on the reactor.
    lobby::LobbyServer::with(config)?.run();

    Ok(())
}
