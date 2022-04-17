use futures::{Stream, TryFuture};
use std::sync::Arc;
use std::str::FromStr;
use std::net::{SocketAddr, ToSocketAddrs};
use std::collections::{HashMap, HashSet};
use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use std::path::PathBuf;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::RwLock;
use tokio::select;
use warp::Filter;
use warp::http::{Response, StatusCode};
use webrtc_unreliable as webrtc;
use drosix_api::{Answer, Command};
use crate::rtc_server::{RtcStatus, RtcServer};
use crate::shadow::Shadow;
use pwhash::unix::verify;
use typed_builder::TypedBuilder;

use rkyv::{archived_root, ser::{serializers::AllocSerializer, Serializer}, Infallible, Deserialize};

mod rtc_server;
mod shadow;


#[derive(Debug)]
enum ServerError {
    Login,
    Other,
}

impl warp::reject::Reject for ServerError {}


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
    #[builder(setter(into))]
    rtc_address: SocketAddr,
    command: Sender<Command>,
    answer: Receiver<Answer>,
}

impl Server {
    async fn login(payload: bytes::Bytes) -> Result<impl warp::Reply, warp::Rejection> {
        let content = String::from_utf8_lossy(payload.as_ref());
        let auth_user = content.split_once('&').and_then(|(user, password)| {
            let user = user.strip_prefix("username=").unwrap_or_default();
            let password = password.strip_prefix("password=").unwrap_or_default();
            // Shadow::from_name(user)
            //     .map(|entry| verify(password, &entry.password))
            //     .and_then(|auth| auth.then(|| user))

            if user == "root" && password == "toor" {
                Some(user)
            } else {
                None
            }
        });
        if let Some(user) = auth_user {
            let token = user;
            eprintln!("auth user {}", user);
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Set-Cookie", format!("token={}; Secure; HttpOnly; Path=/api", token))
                .body(""))
        }
        else {
            eprintln!("reject login");
            Err(warp::reject::custom(ServerError::Login))
        }
    }

    fn auth() -> impl warp::Filter<Extract = (), Error = warp::Rejection> + Clone {
        warp::any()
            .and(warp::cookie::optional::<String>("token"))
            .and_then(|token: Option<String>| async move {
                if token.map(|x| x == "tchamelot").unwrap_or_default() {
                    eprintln!("user auth");
                    Ok(())
                } else {
                    eprintln!("user not auth");
                    Err(warp::reject::custom(ServerError::Login))
                }
            })
            .untuple_one()
    }

    async fn handle_rejection(reject: warp::Rejection) -> Result<impl warp::Reply, warp::Rejection> {
        let mut response = warp::http::Response::builder();
        println!("Handle error");
        if let Some(error) = reject.find::<ServerError>() {
            match error {
                ServerError::Login => {
                    println!("Handle login error");
                    response = response.header("Location", "/login");
                    response = response.status(warp::http::StatusCode::UNAUTHORIZED);
                },
            _ => (),
            }
            response.body("").or_else(|_| Err(warp::reject::custom(ServerError::Other)))
        } else {
            Err(reject)
        }
    }

    #[tokio::main(flavor = "current_thread")]
    pub async fn run(self) {
        let rtc_server = RtcServer::new(self.rtc_address, self.answer, self.command).await.unwrap();
        let rtc_endpoint = rtc_server.endpoint();
        let rtc_status = rtc_server.status();

        let rtc_endpoint = warp::any().map(move || rtc_endpoint.clone());
        let rtc_status = warp::any().map(move || rtc_status.clone());

        let login = warp::path("login").and(warp::body::content_length_limit(64))
            .and(warp::body::bytes())
            .and_then(Server::login);

        let webrtc = warp::path("webrtc")
            .and(Self::auth())
            .and(rtc_endpoint)
            .and(warp::body::stream())
            .and(warp::post())
            .and_then(RtcServer::connect);

        let subscribe = warp::path!("measure" / u32)
            .and(Self::auth())
            .and(rtc_status.clone())
            .and(warp::put())
            .and_then(RtcServer::subscribe);

        let unsubscribe =
            warp::path!("measure" / u32)
            .and(Self::auth())
            .and(rtc_status.clone())
            .and(warp::delete())
            .and_then(RtcServer::unsubscribe);

        let take_control =
            warp::path!("control" / u32)
            .and(Self::auth())
            .and(rtc_status.clone())
            .and(warp::get())
            .and_then(RtcServer::take_control);

        let release_control =
            warp::path!("control" / u32)
            .and(Self::auth())
            .and(rtc_status)
            .and(warp::put())
            .and_then(RtcServer::release_control);

        let api = warp::path("api")
            .and(webrtc
                .or(subscribe)
                .or(unsubscribe)
                .or(take_control)
                .or(release_control)
                .or(login));

        let fallback = self.http_root.join("index.html");
        let app = warp::fs::dir(self.http_root).or(warp::fs::file(fallback));
        
        let routes = api.or(app).recover(Self::handle_rejection);

        let rtc_server = rtc_server.run();

        if let Some((cert, key)) = self.cert.zip(self.key) {
            // let tls_server = warp::serve(routes).tls()
            //     .cert_path(cert)
            //     .key_path(key).run(self.http_address);
            // tokio::join!(tls_server, rtc_server);
        } else {
            let http_server = warp::serve(routes).run(self.http_address);
            tokio::join!(http_server, rtc_server);
        }
    }
}
