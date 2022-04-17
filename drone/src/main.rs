use std::thread;
use tokio::sync::mpsc::channel;

use drone::flight_controller::FlightController;
use server::Server;

fn main() {
    let (answer_tx, answer_rx) = channel(10);

    let (command_tx, command_rx) = channel(10);

    let server = Server::builder()
        .http_address(([0, 0, 0, 0], 443))
        .http_root("/var/www")
        .cert("/etc/ssl/drosix.pem")
        .key("/etc/ssl/drosix.rsa")
        .build();

    let mut controller = FlightController::new(command_rx, answer_tx)
        .expect("Failed to start flight controller");

    let drone = thread::spawn(move || controller.run());
    let server = thread::spawn(move || server.run());

    drone.join().unwrap();
    server.join().unwrap();
}
