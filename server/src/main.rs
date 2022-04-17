use drosix_api::{Answer, Command};
use server::Server;
use tokio::sync::mpsc::channel;

fn main() {
    let (command_sender, _) = channel(10);
    let (_, answer_receiver) = channel(10);
    let server = Server::builder()
        .http_address(([0, 0, 0, 0], 8080))
        .http_root("../webapp/dist")
        .rtc_address(([192, 168, 1, 14], 3334))
        .command(command_sender)
        .answer(answer_receiver)
        .build();

    server.run();
}
