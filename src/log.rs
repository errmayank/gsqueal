use console::style;
use std::fmt::Arguments;

#[derive(Debug)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

pub fn message(level: LogLevel, message: Arguments) {
    let log_level = match level {
        LogLevel::Info => style("[info]").green().bold(),
        LogLevel::Warn => style("[warn]").yellow().bold(),
        LogLevel::Error => style("[error]").red().bold(),
    };
    println!("{} {}", log_level, message);
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::log::message($crate::log::LogLevel::Info, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::log::message($crate::log::LogLevel::Warn, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::log::message($crate::log::LogLevel::Error, format_args!($($arg)*))
    };
}
