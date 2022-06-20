use std::thread;
use tokio::sync::mpsc::channel;

use drone::flight_controller::FlightController;

#[cfg(feature = "mock")]
mod mock;
mod server;

#[cfg(feature = "mock")]
use mock::drone;
use server::server;

fn main() {
    let (answer_tx, answer_rx) = channel(10);

    let (command_tx, command_rx) = channel(10);

    let mut controller = FlightController::new(command_rx, answer_tx)
        .expect("Failed to start flight controller");

    let drone = thread::spawn(move || controller.run());
    // let server = thread::spawn(move || server(answer_rx, command_tx));

    drone.join().unwrap();
    // server.join().unwrap();
}
