use std::sync::mpsc;
use std::thread;
use tokio::sync::broadcast::channel;

use drone::flight_controller::FlightController;

#[cfg(feature = "mock")]
mod mock;
mod server;

#[cfg(feature = "mock")]
use mock::drone;
use server::server;

fn main() {
    let (sender, _) = channel(10);
    let sender_drone = sender.clone();
    let sender_server = sender.clone();

    let (control_tx, control_rx) = mpsc::channel();

    let mut controller = FlightController::new(control_rx, sender_drone)
        .expect("Failed to start flight controller");

    let drone = thread::spawn(move || controller.run());
    let server = thread::spawn(move || server(sender_server, control_tx));

    drone.join().unwrap();
    server.join().unwrap();
}
