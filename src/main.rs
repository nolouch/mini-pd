mod allocator;
mod cluster;
mod config;
mod error;
mod kv;
mod net;

use slog::info;
use sloggers::terminal::{Destination, TerminalLoggerBuilder};
use sloggers::types::Severity;
use sloggers::Build;

use parking_lot::Mutex;
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time::Duration};

pub use cluster::stats::RegionStats;
pub use config::Config;
pub use error::{Error, Result};
pub use kv::{AddressMap, Command, Event, Msg, Res};
pub use net::Server;

fn main() {
    let mut addrs = HashMap::new();
    addrs.insert(1, "172.16.4.3:3279".to_owned());
    let mut builder = TerminalLoggerBuilder::new();
    builder.level(Severity::Debug);
    builder.destination(Destination::Stderr);
    let logger = builder.build().unwrap();
    let mut config = Config::default();
    config.my_id = 1;
    config.address = "172.16.4.3:3279".to_string();
    config.advertise_address = "172.16.4.3:3279".to_string();
    config.data_dir = Path::new("./pd-data").to_path_buf();
    config.initial_peers = vec![1];
    config
        .initial_address_book
        .insert(1, "172.16.4.3:3279".to_owned());
    config.raft_election_ticks = 5;
    config.raft_heartbeat_ticks = 1;

    let mut s = Server::new(Arc::new(Mutex::new(addrs)), config, logger.clone());
    s.start().unwrap_or_else(|e| panic!("meet error {}", e));
    info!(logger, "server is ready to serve");
    let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term)).unwrap();
    while !term.load(Ordering::Relaxed) {
        // Do some time-limited stuff here
        // (if this could block forever, then there's no guarantee the signal will have any
        // effect).
        thread::sleep(Duration::from_secs(1));
    }
}
