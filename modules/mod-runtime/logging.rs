macro_rules! info {
    ($($arg:tt)*) => {
        tracing::info!(target: "shroudforge::api", $($arg)*)
    };
}

macro_rules! warning {
    ($($arg:tt)*) => {
        tracing::warn!(target: "shroudforge::api", $($arg)*)
    };
}

macro_rules! error {
    ($($arg:tt)*) => {
        tracing::error!(target: "shroudforge::api", $($arg)*)
    };
}

macro_rules! debug {
    ($($arg:tt)*) => {
        tracing::debug!(target: "shroudforge::api", $($arg)*)
    };
}

macro_rules! warn_span {
    ($($arg:tt)*) => {
        tracing::warn_span!(target: "shroudforge::api", $($arg)*)
    };
}

pub(crate) use debug;
pub(crate) use error;
pub(crate) use info;
pub(crate) use warning as warn; // use alias to resolve ambiguity with builtin attribute

pub(crate) use warn_span;
