use crate::kprintln;
use crate::utils::time::now;
use alloc::format;
use core::sync::atomic::{AtomicBool, Ordering};
use log::{set_logger_racy, set_max_level, Level, LevelFilter, Log, Metadata, Record};


static mut LOGGER: Option<Logger> = None;
static SHOULD_LOG: AtomicBool = AtomicBool::new(true);

pub struct Logger {
    filter_level: LevelFilter,
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        SHOULD_LOG.load(Ordering::Relaxed) && metadata.level() <= self.filter_level
    }

    fn log(&self, record: &Record) {
        let level = record.level();

        let level = match level {
            Level::Error => format!("\x1b[31m{level: <5}\x1b[0m"),
            Level::Warn => format!("\x1b[33m{level: <5}\x1b[0m"),
            Level::Info => format!("\x1b[32m{level: <5}\x1b[0m"),
            Level::Debug => format!("\x1b[35m{level: <5}\x1b[0m"),
            Level::Trace => format!("\x1b[34m{level: <5}\x1b[0m")
        };

        let timestamp = now();

        kprintln!("[{} | {} | {}] {}", timestamp, level, record.target(), record.args());
    }

    fn flush(&self) {}
}

pub fn without_logging<T>(f: impl FnOnce() -> T) -> T {
    SHOULD_LOG.store(false, Ordering::Relaxed);
    let result = f();
    SHOULD_LOG.store(true, Ordering::Relaxed);

    result
}

pub fn init_logger() {
    let filter_level = LevelFilter::Debug;

    let logger = Logger {
        filter_level
    };

    unsafe {
        LOGGER = Some(logger);
        #[allow(static_mut_refs)]
        set_logger_racy(LOGGER.as_ref().unwrap()).expect("Logger already set");
        set_max_level(filter_level);
    }
}