use std::sync::mpsc::channel;
use std::thread;

use drone::flight_controller::FlightController;
use drone::remote::remote;

fn main() {
    let (answer_tx, answer_rx) = channel();

    let (command_tx, command_rx) = channel();

    let mut controller = FlightController::new(command_rx, answer_tx).expect("Failed to start flight controller");

    let drone = thread::spawn(move || controller.run());
    let remote = thread::spawn(move || remote(command_tx));

    drone.join().unwrap();
    remote.join().unwrap();
}
