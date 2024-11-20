#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)*) => {{
        lazy_static::lazy_static! {
            static ref LEVEL_COLORS: std::collections::HashMap<Level, (&'static str, colored::Color)> = {
                let mut map = std::collections::HashMap::new();
                map.insert(Level::Fatal, ("FATAL", colored::Color::BrightRed));
                map.insert(Level::Docker, ("DOCKER", colored::Color::BrightYellow));
                map.insert(Level::Info, ("INFO", colored::Color::Cyan));
                map.insert(Level::Build, ("BUILD", colored::Color::BrightGreen));
                map.insert(Level::Success, ("SUCCESS", colored::Color::Green));
                map.insert(Level::Debug, ("DEBUG", colored::Color::Magenta));
                map.insert(Level::Notice, ("NOTICE", colored::Color::BrightBlue));
                map.insert(Level::Warning, ("WARN", colored::Color::Yellow));
                map.insert(Level::Error, ("ERROR", colored::Color::Red));
                return map;
            };
        }

        if $level == Level::None {
            print!("{}", format_args!($($arg)*).to_string());
        } else {
            match LEVEL_COLORS.get(&$level) {
                Some((level_text, color)) => {
                    let level_text = level_text.color(*color);
                    println!("{} {}", level_text, format_args!($($arg)*).to_string())
                }
                None => println!("Unknown log level: {:?}", $level),
            };
        }
    }};
}
