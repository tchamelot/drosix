use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use thread_priority::{
    RealtimeThreadSchedulePolicy, ScheduleParams, ThreadBuilder, ThreadPriority, ThreadSchedulePolicy
};

use drone::flight_controller::FlightController;
use drone::log::Logger;
use drone::remote::remote;

fn main() {
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
    let remote = thread::Builder::new().name("remote".into()).spawn(move || remote(command_tx)).unwrap();

    loop {
        log_sink.handle_logs();
        thread::sleep(Duration::from_millis(10));
    }

    let _ = drone.join();
    let _ = remote.join();
}
