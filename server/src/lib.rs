use std::net::SocketAddr;
use std::path::PathBuf;
use std::fs::File;
use std::io::{BufReader, BufRead};
use warp::{Filter, Reply, Rejection, reject};
use warp::http::{StatusCode, Response};
use bytes::Bytes;
use typed_builder::TypedBuilder;

#[cfg(feature = "shadow")]
mod shadow;

mod rtc_server;

use rtc_server::RtcServer;

#[derive(Debug)]
enum ServerError {
    Login,
    Other,
}

impl reject::Reject for ServerError {}

#[derive(TypedBuilder)]
pub struct Server{
    #[builder(setter(into))]
    http_address: SocketAddr,
    #[builder(setter(into))]
    http_root: PathBuf,
    #[builder(setter(into), default="/etc/.htpasswd".into())]
    user_registry: PathBuf,
    #[builder(setter(strip_option, into), default)]
    cert: Option<PathBuf>,
    #[builder(setter(strip_option, into), default)]
    key: Option<PathBuf>,
    #[builder(setter(into))]
    rtc_address: SocketAddr,
}

impl Server {
    async fn login(payload: Bytes, registry: PathBuf) -> Result<impl Reply, Rejection> {
        let content = String::from_utf8_lossy(payload.as_ref());
        let auth_user = content.split_once('&').and_then(|(user, password)| {
            let user = user.strip_prefix("username=").unwrap_or_default();
            let password = password.strip_prefix("password=").unwrap_or_default();
            #[cfg(feature = "shadow")]
            {
                shadow::Shadow::from_name(user)
                    .map(|entry| pwhash::unix::verify(password, &entry.password))
                    .and_then(|auth| auth.then(|| user))
            }
            #[cfg(not(feature = "shadow"))]
            {
                File::open(registry)
                    .map(|file| {
                        let reader = BufReader::new(file);
                        reader.lines().find(|entry| {
                            entry.as_ref().map(|x| x.starts_with(&(String::from(user) + ":")))
                                .unwrap_or_default()
                        })
                        .transpose()
                        .ok()
                        .flatten()
                        .and_then(|entry| {
                            entry.split_once(':')
                                .map(|(_, hash)| pwhash::unix::verify(password, hash))
                        })
                        .and_then(|auth| auth.then(|| user))
                    }).unwrap_or_default()
            }
        });
        if let Some(user) = auth_user {
            let token = user;
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Set-Cookie", format!("token={}; Secure; HttpOnly; Path=/api", token))
                .body(""))
        }
        else {
            eprintln!("reject login");
            Ok(Response::builder().status(warp::http::StatusCode::UNAUTHORIZED)
                .body(""))
        }
    }

    async fn handle_rejection(reject: warp::Rejection) -> Result<impl warp::Reply, warp::Rejection> {
        if reject.is_not_found() {
            eprintln!("Handle not found");
            Response::builder().status(warp::http::StatusCode::NOT_FOUND)
        } else if let Some(error) = reject.find::<ServerError>() {
            match error {
                ServerError::Login => {
                    eprintln!("Handle login error");
                    Response::builder().status(warp::http::StatusCode::UNAUTHORIZED)
                },
                ServerError::Other => {
                    eprintln!("Handle other error");
                    Response::builder().status(StatusCode::NOT_FOUND)
                }
            }
        } else {
            eprintln!("Unknown internal error");
            Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR)
        }.body("").map_err(|_|warp::reject::custom(ServerError::Other))

    }


    #[tokio::main(flavor = "current_thread")]
    pub async fn run(self) {
        let fallback = self.http_root.join("index.html");
        let rtc_server = RtcServer::new(self.rtc_address).await.unwrap();


        let registry = warp::any().map(move || self.user_registry.clone());
        let login = warp::path("login").and(warp::body::content_length_limit(64))
            .and(warp::body::bytes())
            .and(registry)
            .and_then(Self::login);

        let api = warp::path("api")
            .and(login
                .or(rtc_server.api())
                .recover(Self::handle_rejection));

        let app = warp::fs::dir(self.http_root).or(warp::fs::file(fallback));

        let routes = api.or(app);

        if let Some((cert, key)) = self.cert.zip(self.key) {
            let tls_server = warp::serve(routes).tls()
                .cert_path(cert)
                .key_path(key).run(self.http_address);
            tokio::join!(tls_server, rtc_server.run());
        } else {
            let http_server = warp::serve(routes).run(self.http_address);
            tokio::join!(http_server, rtc_server.run());
        }
    }
}
