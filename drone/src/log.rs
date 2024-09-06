use std::io::{BufWriter, Write};
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::time::Instant;

use anyhow::Result;
use log::{Level, LevelFilter, Log, Metadata, Record};
use serde::Deserialize;

use crate::config::DROSIX_CONFIG;

#[cfg(feature = "profiling")]
use metrics_util::debugging::{DebugValue, DebuggingRecorder, Snapshotter};
#[cfg(feature = "profiling")]
use rstats::Stats;

const BROADCAST: &'static str = "255.255.255.255:9000";

#[derive(Deserialize)]
#[serde(tag = "sink", rename_all = "lowercase")]
enum LogConfig {
    Stdout,
    File {
        path: String,
    },
    Udp {
        port: u16,
    },
}
impl LogConfig {
    fn to_writer(self) -> Result<BufWriter<Box<dyn Write>>> {
        let writer: Box<dyn Write> = match self {
            LogConfig::Stdout => Box::new(std::io::stdout()),
            LogConfig::File {
                path,
            } => Box::new(std::fs::File::create(path)?),
            LogConfig::Udp {
                port,
            } => Box::new(UdpBroadcastStream::from(UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], port)))?)),
        };
        Ok(BufWriter::with_capacity(1000, writer))
    }
}

struct SyncRecord {
    timestamp: Instant,
    level: Level,
    content: String,
}

pub struct LogSink {
    receiver: Receiver<SyncRecord>,
    start: Instant,
    output: BufWriter<Box<dyn Write>>,
    #[cfg(feature = "profiling")]
    snapchotter: Snapshotter,
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
        log::set_logger(Box::leak(logger))
            .map(|()| log::set_max_level(LevelFilter::Trace))
            .expect("Cannot install global logger");
        #[cfg(feature = "profiling")]
        let snapchotter = {
            let recorder = DebuggingRecorder::new();
            let snapchotter = recorder.snapshotter();
            recorder.install().expect("Cannot install global recorder");
            snapchotter
        };
        let config: LogConfig = DROSIX_CONFIG.get("log").unwrap_or(LogConfig::Stdout);
        let output = config.to_writer().unwrap();
        LogSink {
            receiver,
            start,
            output,
            #[cfg(feature = "profiling")]
            snapchotter,
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
// TODO Handle different log output:
// - stdout
// - file
// - udp
impl LogSink {
    pub fn handle_logs(&mut self) {
        for record in self.receiver.try_iter() {
            writeln!(
                self.output,
                "[{:<9.5}] {:<5}: {}",
                record.timestamp.duration_since(self.start).as_secs_f32(),
                record.level,
                record.content
            )
            .inspect_err(|err| eprintln!("{}", err))
            .ok();
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
                        writeln!(
                            self.output,
                            "[{:<9.5}] {:<5}: {} frequency: {:>6.2}Hz, max: {:>6.2e}s, mean {:>6.2e}s ± {:>4.2e}s",
                            self.start.elapsed().as_secs_f32(),
                            Level::Trace,
                            key.key().labels().next().unwrap().value(),
                            freq,
                            max,
                            stats.centre,
                            stats.spread
                        )
                        .inspect_err(|err| eprintln!("{}", err))
                        .ok();
                    }
                }
                self.previous = Instant::now();
            }
        }

        self.output.flush().inspect_err(|err| eprintln!("{}", err)).ok();
    }
}

struct UdpBroadcastStream(UdpSocket);

impl From<UdpSocket> for UdpBroadcastStream {
    fn from(value: UdpSocket) -> Self {
        // value.set_nonblocking(true);
        value.set_broadcast(true).unwrap();
        Self(value)
    }
}

impl Write for UdpBroadcastStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.send_to(buf, BROADCAST)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
