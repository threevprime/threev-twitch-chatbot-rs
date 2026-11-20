use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::sync::Mutex;

use chrono::Utc;
use log::{LevelFilter, Metadata, Record};

struct Logger {
    file: Mutex<File>,
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let message = format!(
                "{} [{}] {}\n",
                formatted_timestamp(),
                record.level(),
                record.args()
            );

            // Log to console
            let _ = io::stderr().lock().write_all(message.as_bytes());

            // Log to file
            if let Ok(mut file) = self.file.lock() {
                let _ = file.write_all(message.as_bytes());
                let _ = file.flush();
            }
        }
    }

    fn flush(&self) {
        if let Ok(mut file) = self.file.lock() {
            let _ = file.flush();
        }
    }
}

fn formatted_timestamp() -> String {
    Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

static LOGGER: std::sync::OnceLock<Logger> = std::sync::OnceLock::new();

pub fn init_logger() {
    let level = match std::env::var("RUST_LOG")
        .ok()
        .as_deref()
        .unwrap_or("info")
        .to_lowercase()
        .as_str()
    {
        "off" => LevelFilter::Off,
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info,
    };

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("./data/application.log")
        .expect("Could not open log file");

    let logger = LOGGER.get_or_init(|| Logger {
        file: Mutex::new(file),
    });

    log::set_logger(logger).expect("Could not initialize logger");
    log::set_max_level(level);
}
