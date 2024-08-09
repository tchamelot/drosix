use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::time::Instant;

use log::{Level, LevelFilter, Log, Metadata, Record};

#[cfg(feature = "profiling")]
use metrics_util::debugging::{DebugValue, DebuggingRecorder, Snapshotter};
#[cfg(feature = "profiling")]
use rstats::Stats;

struct SyncRecord {
    timestamp: Instant,
    level: Level,
    content: String,
}

pub struct LogSink {
    receiver: Receiver<SyncRecord>,
    #[cfg(feature = "profiling")]
    snapchotter: Snapshotter,
    start: Instant,
    #[cfg(feature = "profiling")]
    previous: Instant,
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

        #[cfg(feature = "profiling")]
        let snapchotter = {
            let recorder = DebuggingRecorder::new();
            let snapchotter = recorder.snapshotter();
            recorder.install().expect("Cannot install global recorder");
            snapchotter
        };
        LogSink {
            receiver,
            #[cfg(feature = "profiling")]
            snapchotter,
            start,
            #[cfg(feature = "profiling")]
            previous: start,
        }
    }
}

impl Log for Logger {
    fn enabled(&self, _: &Metadata) -> bool {
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
    pub fn handle_logs(&mut self) {
        for record in self.receiver.try_iter() {
            println!(
                "[{:<9.5}] {:<5}: {}",
                record.timestamp.duration_since(self.start).as_secs_f32(),
                record.level,
                record.content
            );
        }
        #[cfg(feature = "profiling")]
        {
            let delta = self.previous.elapsed().as_secs_f32();
            if delta > 0.5 {
                let snapchot = self.snapchotter.snapshot();
                for (key, _, _, metric) in snapchot.into_vec().iter() {
                    if let DebugValue::Histogram(histogram) = metric {
                        let max = histogram.iter().max().map(|x| x.into_inner()).unwrap_or(0.0);
                        let stats = histogram.ameanstd().unwrap();
                        let freq = histogram.len() as f32 / delta;
                        println!(
                            "[{:<9.5}] {:<5}: {} frequency: {:>6.2}Hz, max: {:>6.2e}s, mean {:>6.2e}s ± {:>4.2e}s",
                            self.start.elapsed().as_secs_f32(),
                            Level::Trace,
                            key.key().labels().next().unwrap().value(),
                            freq,
                            max,
                            stats.centre,
                            stats.spread
                        );
                    }
                }
            }
            self.previous = Instant::now();
        }
    }
}
