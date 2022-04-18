use server::Server;

fn main() {
    let server = Server::builder()
        .http_address(([0, 0, 0, 0], 8080))
        .http_root("../webapp/dist")
        .user_registry("htpasswd")
        .build();

    server.run();
}
