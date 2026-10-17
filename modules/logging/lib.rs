//! Single process-wide ShroudForge logging pipeline.

use std::{
    fmt,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use tracing::{Event, Subscriber, field::Visit};
use tracing_subscriber::{
    Layer,
    filter::LevelFilter,
    fmt::{FmtContext, FormatEvent, FormatFields, format::Writer},
    layer::SubscriberExt,
    registry::LookupSpan,
    util::SubscriberInitExt,
};

mod log_writer;
use log_writer::*;
mod format_log_line;
use format_log_line::{LineFormat, STARTED};
pub use format_log_line::{format_line, timestamp};
mod initialize_logging;
pub use initialize_logging::initialize;
mod append_log;
pub use append_log::append;
mod lock_log_file;
use lock_log_file::with_log_lock;
mod allows_log_level;
pub use allows_log_level::allows;
use allows_log_level::configured_level;
mod write_activity;
pub use write_activity::write_activity;
