use colored::*;
use std::fmt::{self, Write};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::Layer;

pub mod prelude {
    pub use super::MaidFormatLayer;
    pub use tracing_subscriber::prelude::*;
}

pub struct MaidFormatLayer;

impl MaidFormatLayer {
    pub fn new() -> Self { Self }

    fn format_level(&self, level: &Level) -> ColoredString {
        match *level {
            Level::TRACE => "TRACE".magenta(),
            Level::DEBUG => "DEBUG".cyan(),
            Level::INFO => "INFO".green(),
            Level::WARN => "WARN".yellow(),
            Level::ERROR => "FATAL".red(),
        }
    }

    fn get_bright_color(&self, level: &Level) -> Box<dyn Fn(&str) -> ColoredString> {
        match *level {
            Level::TRACE => Box::new(|s| s.bright_magenta()),
            Level::DEBUG => Box::new(|s| s.bright_cyan()),
            Level::INFO => Box::new(|s| s.bright_green()),
            Level::WARN => Box::new(|s| s.bright_yellow()),
            Level::ERROR => Box::new(|s| s.bright_red()),
        }
    }
}

impl<S> Layer<S> for MaidFormatLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let metadata = event.metadata();
        let mut output = String::new();

        output.push_str(&format!("{} ", self.format_level(metadata.level())));

        if let Some(path) = metadata.module_path() {
            let without_first = path.split("::").skip(1).collect::<Vec<_>>().join("::");
            output.push_str(&format!("{} ", without_first.white()));
        }

        event.record(&mut CommandVisitor {
            output: &mut output,
            colorizer: self.get_bright_color(metadata.level()),
        });

        println!("{}", output);
    }
}

struct CommandVisitor<'a> {
    output: &'a mut String,
    colorizer: Box<dyn Fn(&str) -> ColoredString>,
}

impl<'a> tracing::field::Visit for CommandVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            let msg = format!("{:?}", value);
            let clean_msg = msg.trim_matches('"');
            write!(self.output, "{}", (self.colorizer)(clean_msg)).unwrap();
        } else {
            write!(self.output, " {}={:?}", field.name(), value).unwrap();
        }
    }
}
