use alloc::format;
use core::fmt::Arguments;
use goolog::log::{set_max_level, Level, LevelFilter};
use crate::kprintln;
use crate::utils::time::now;

pub fn print_log(target: &str, level: Level, args: &Arguments) {
    let level = match level {
        Level::Error => format!("\x1b[31m{level: <5}\x1b[0m"),
        Level::Warn => format!("\x1b[33m{level: <5}\x1b[0m"),
        Level::Info => format!("\x1b[32m{level: <5}\x1b[0m"),
        Level::Debug => format!("\x1b[35m{level: <5}\x1b[0m"),
        Level::Trace => format!("\x1b[34m{level: <5}\x1b[0m")
    };

    let timestamp = now();

    kprintln!("[{} | {} | {}] {}", timestamp, level, target, args);
}

pub fn init_logger() {
    goolog::init_logger(
        Some(Level::Trace),
        None,
        &|_timestamp, target, level, args| print_log(target, level, args)
    )
        .expect("Could not initialize logger");

    set_max_level(LevelFilter::Info);
}