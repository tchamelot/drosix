use std::net::SocketAddr;
use std::path::PathBuf;
use warp::Filter;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct Server{
    #[builder(setter(into))]
    http_address: SocketAddr,
    #[builder(setter(into))]
    http_root: PathBuf,
    #[builder(setter(strip_option, into), default)]
    cert: Option<PathBuf>,
    #[builder(setter(strip_option, into), default)]
    key: Option<PathBuf>,
}

impl Server {
    #[tokio::main(flavor = "current_thread")]
    pub async fn run(self) {
        let fallback = self.http_root.join("index.html");
        let app = warp::fs::dir(self.http_root).or(warp::fs::file(fallback));
        
        let routes = app;

        if let Some((cert, key)) = self.cert.zip(self.key) {
            let tls_server = warp::serve(routes).tls()
                .cert_path(cert)
                .key_path(key).run(self.http_address);
            tls_server.await;
        } else {
            let http_server = warp::serve(routes).run(self.http_address);
            http_server.await;
        }
    }
}
