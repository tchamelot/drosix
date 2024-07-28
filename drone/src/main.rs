use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use drone::flight_controller::FlightController;
use drone::log::Logger;
use drone::remote::remote;

fn main() {
    let log_sink = Logger::init();
    let (answer_tx, _answer_rx) = channel();

    let (command_tx, command_rx) = channel();

    let mut controller = FlightController::new(command_rx, answer_tx);

    let drone = thread::spawn(move || controller.run());
    let remote = thread::spawn(move || remote(command_tx));

    loop {
        log_sink.handle_events();
        thread::sleep(Duration::from_millis(10));
    }

    let _ = drone.join();
    let _ = remote.join();
}
