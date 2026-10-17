macro_rules! warning {
    ($($arg:tt)*) => {
        tracing::warn!(target: "shroudforge::package", $($arg)*)
    };
}

macro_rules! debug {
    ($($arg:tt)*) => {
        tracing::debug!(target: "shroudforge::package", $($arg)*)
    };
}

pub(crate) use debug;
pub(crate) use warning as warn; // use alias to resolve ambiguity with builtin attribute
