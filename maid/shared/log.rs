pub mod layer;
pub mod verbose;

pub mod prelude {
    pub use crate::{debug, error, info, trace, warn};
    pub use colored::{ColoredString, Colorize};
}

#[macro_export]
macro_rules! info {
    ($($field:tt)*) => {
        tracing::info!($($field)*)
    };
}

#[macro_export]
macro_rules! warn {
    ($($field:tt)*) => {
        tracing::warn!($($field)*)
    };
}

#[macro_export]
macro_rules! error {
    ($($field:tt)*) => {{
        tracing::error!($($field)*);
        std::process::exit(1);
    }};
}

#[macro_export]
macro_rules! debug {
    ($($field:tt)*) => {
        tracing::debug!($($field)*)
    };
}

#[macro_export]
macro_rules! trace {
    ($($field:tt)*) => {
        tracing::trace!($($field)*)
    };
}
