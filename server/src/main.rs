use server::Server;

fn main() {
    let server = Server::builder()
        .http_address(([0, 0, 0, 0], 8080))
        .http_root("../webapp/dist")
        .user_registry("htpasswd")
        .rtc_address(([192, 168, 1, 11], 3333))
        .build();

    server.run();
}
