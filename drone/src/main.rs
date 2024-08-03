use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use thread_priority::{
    RealtimeThreadSchedulePolicy, ScheduleParams, ThreadBuilder, ThreadPriority, ThreadSchedulePolicy
};

use drone::flight_controller::FlightController;
use drone::log::Logger;
use drone::remote::remote;
use metrics_util::debugging::{DebugValue, DebuggingRecorder};
use rstats::Stats;

fn main() {
    let log_sink = Logger::init();
    let recorder = DebuggingRecorder::new();
    let snapchotter = recorder.snapshotter();
    recorder.install().expect("Cannot install global recorder");

    let (answer_tx, _answer_rx) = channel();

    let (command_tx, command_rx) = channel();

    let mut controller = FlightController::new(command_rx, answer_tx);

    let drone = ThreadBuilder::default()
        .name("controller")
        .policy(ThreadSchedulePolicy::Realtime(RealtimeThreadSchedulePolicy::Fifo))
        .priority(ThreadPriority::from_posix(ScheduleParams {
            sched_priority: 40,
        }))
        .spawn_careless(move || controller.run())
        .unwrap();
    let remote = thread::Builder::new().name("remote".into()).spawn(move || remote(command_tx)).unwrap();

    let mut should_snapchot = 50;
    loop {
        log_sink.handle_events();
        should_snapchot -= 1;
        if should_snapchot == 0 {
            let snapchot = snapchotter.snapshot();
            for (key, _, _, metric) in snapchot.into_vec().iter() {
                if let DebugValue::Histogram(histogram) = metric {
                    println!(
                        "\t{} count: {}, max: {:.2e}, {}",
                        key.key(),
                        histogram.len(),
                        histogram.iter().max().map(|x| x.into_inner()).unwrap_or(-1.0),
                        histogram.ameanstd().unwrap(),
                    );
                }
            }
            should_snapchot = 50;
        }
        thread::sleep(Duration::from_millis(10));
    }

    let _ = drone.join();
    let _ = remote.join();
}
