use signal_hook::{consts::TERM_SIGNALS, flag};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use thread_priority::{
    RealtimeThreadSchedulePolicy, ScheduleParams, ThreadBuilder, ThreadPriority, ThreadSchedulePolicy
};

use drone::flight_controller::FlightController;
use drone::log::Logger;
use drone::remote::remote;
use drone::types::Command;

fn main() {
    let stop = Arc::new(AtomicBool::new(false));
    for signals in TERM_SIGNALS {
        flag::register_conditional_shutdown(*signals, 1, Arc::clone(&stop)).unwrap();
        flag::register(*signals, Arc::clone(&stop)).unwrap();
    }

    let mut log_sink = Logger::init();

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

    let remote_tx = command_tx.clone();
    let _remote = thread::Builder::new().name("remote".into()).spawn(move || remote(remote_tx)).unwrap();

    while !stop.load(Ordering::Relaxed) {
        log_sink.handle_logs();
        thread::sleep(Duration::from_millis(10));
    }

    // We want to crash anyway if we got here
    command_tx.send(Command::Stop).unwrap();

    let _ = drone.join();
    // let _ = remote.join();
    log_sink.handle_logs();
}
