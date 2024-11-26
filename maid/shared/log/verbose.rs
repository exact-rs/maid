use std::fmt;
use tracing::{level_filters::LevelFilter, Level};

#[derive(clap::Args, Debug, Clone, Default)]
pub struct Verbosity<L: LogLevel = ErrorLevel> {
    #[arg(
        long,
        short = 'v',
        action = clap::ArgAction::Count,
        global = true,
        help = L::verbose_help(),
        long_help = L::verbose_long_help(),
    )]
    verbose: u8,

    #[arg(
        long,
        short = 'q',
        global = true,
        help = L::quiet_help(),
        long_help = L::quiet_long_help(),
        conflicts_with = "verbose",
    )]
    quiet: bool,

    #[arg(skip)]
    phantom: std::marker::PhantomData<L>,
}

#[allow(dead_code)]
impl<L: LogLevel> Verbosity<L> {
    pub fn new(verbose: u8, quiet: bool) -> Self {
        Verbosity {
            verbose,
            quiet,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn is_silent(&self) -> bool { self.log_level().is_none() }

    pub fn log_level(&self) -> Option<Level> { level_enum(self.verbosity()) }

    pub fn log_level_filter(&self) -> LevelFilter { return level_enum(self.verbosity()).map(LevelFilter::from_level).unwrap_or(LevelFilter::OFF); }

    fn verbosity(&self) -> i8 {
        if !self.quiet {
            level_value(L::default()) + (self.verbose as i8)
        } else {
            -1
        }
    }
}

fn level_value(level: Option<Level>) -> i8 {
    match level {
        None => -1,
        Some(Level::ERROR) => 0,
        Some(Level::WARN) => 1,
        Some(Level::INFO) => 2,
        Some(Level::DEBUG) => 3,
        Some(Level::TRACE) => 4,
    }
}

fn level_enum(verbosity: i8) -> Option<Level> {
    match verbosity {
        i8::MIN..=-1 => None,
        0 => Some(Level::ERROR),
        1 => Some(Level::WARN),
        2 => Some(Level::INFO),
        3 => Some(Level::DEBUG),
        4..=i8::MAX => Some(Level::TRACE),
    }
}

impl<L: LogLevel> fmt::Display for Verbosity<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.verbosity()) }
}

pub trait LogLevel {
    fn default() -> Option<Level>;

    fn verbose_help() -> Option<&'static str> { Some("Increase logging verbosity") }

    fn verbose_long_help() -> Option<&'static str> { None }

    fn quiet_help() -> Option<&'static str> { Some("Decrease logging verbosity") }

    fn quiet_long_help() -> Option<&'static str> { None }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct ErrorLevel;

impl LogLevel for ErrorLevel {
    fn default() -> Option<Level> { return Some(Level::ERROR); }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct WarnLevel;

impl LogLevel for WarnLevel {
    fn default() -> Option<Level> { return Some(Level::WARN); }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct InfoLevel;

impl LogLevel for InfoLevel {
    fn default() -> Option<Level> { return Some(Level::INFO); }
}
