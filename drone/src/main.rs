use std::thread;
use tokio::sync::broadcast::channel;

#[cfg(not(feature = "mock"))]
mod drone;
#[cfg(feature = "mock")]
mod mock;
mod server;

#[cfg(not(feature = "mock"))]
use drone::drone;
#[cfg(feature = "mock")]
use mock::drone;
use server::server;

fn main() {
    let (sender, _) = channel(10);
    let sender_drone = sender.clone();
    let sender_server = sender.clone();

    let drone = thread::spawn(move || drone(sender_drone));
    let server = thread::spawn(move || server(sender_server));

    drone.join().unwrap();
    server.join().unwrap();
}
