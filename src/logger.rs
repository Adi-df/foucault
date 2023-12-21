use std::env;
use std::fmt::Write;
use std::fs::{File, OpenOptions};
use std::io::Write as IoWrite;
use std::sync::Mutex;

use log::Log;

pub struct SimpleLogger {
    file: Option<Mutex<File>>,
}

impl SimpleLogger {
    pub fn new(clear: bool) -> Self {
        Self {
            file: env::var("LOG_FILE").ok().map(|file_name| {
                Mutex::new(
                    OpenOptions::new()
                        .create(true)
                        .write(true)
                        .append(!clear)
                        .open(file_name)
                        .unwrap(),
                )
            }),
        }
    }
}

impl Log for SimpleLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            if let Some(file) = &self.file {
                let mut file = file.lock().unwrap();
                let mut new_record = String::new();
                writeln!(&mut new_record, "{} - {}", record.level(), record.args()).unwrap();
                file.write_all(new_record.as_bytes()).unwrap();
            }
        }
    }

    fn flush(&self) {}
}
