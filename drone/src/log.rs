use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::thread;
use std::time::{Duration, Instant};

use log::{Level, LevelFilter, Log, Metadata, Record};

struct SyncRecord {
    timestamp: Instant,
    level: Level,
    content: String,
}

pub struct LogSink {
    receiver: Receiver<SyncRecord>,
    start: Instant,
}

pub struct Logger {
    sender: SyncSender<SyncRecord>,
}

impl Logger {
    pub fn init() -> LogSink {
        let (sender, receiver) = sync_channel(20);
        let start = Instant::now();
        let logger = Box::new(Self {
            sender,
        });
        let _ = log::set_logger(Box::leak(logger)).map(|()| log::set_max_level(LevelFilter::Trace));
        LogSink {
            receiver,
            start,
        }
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let _ = self.sender.try_send(SyncRecord {
                timestamp: Instant::now(),
                content: std::fmt::format(*record.args()),
                level: record.level(),
            });
        }
    }

    fn flush(&self) {}
}

impl LogSink {
    pub fn handle_events(&self) {
        for record in self.receiver.try_iter() {
            println!(
                "[{}] {}: {}",
                record.timestamp.duration_since(self.start).as_secs_f32(),
                record.level,
                record.content
            );
        }
    }
}
