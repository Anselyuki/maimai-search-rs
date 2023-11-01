use colored::Colorize;
use log::{Level, Metadata, Record};
use log::{LevelFilter, SetLoggerError};

static LOGGER: SimpleLogger = SimpleLogger;

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let args = record.args();
            match record.level() {
                Level::Error => { eprintln!("{}{} {}", "error".red().bold(), ":".bold(), args); }
                Level::Warn => { println!("{}{} {}", "warning".yellow().bold(), ":".bold(), args); }
                Level::Info => { println!("{}{} {}", "info".green().bold(), ":".bold(), args); }
                _ => {}
            }
        }
    }
    fn flush(&self) {}
}

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info))
}