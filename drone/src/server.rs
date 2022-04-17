use futures::{FutureExt, Stream, TryStreamExt};
use std::{
    collections::{HashMap, HashSet}, net::SocketAddr, str::FromStr, sync::Arc
};
use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::RwLock;
use tokio::select;
use warp::Filter;
use webrtc_unreliable as webrtc;
use drosix_api::{Answer, Command};
use crate::rtc_server::{RtcStatus, RtcServer};
use crate::shadow::Shadow;
use pwhash::unix::verify;

use rkyv::{archived_root, ser::{serializers::AllocSerializer, Serializer}, Infallible, Deserialize};

#[cfg(not(feature = "mock"))]
const RTC_ADDR: &'static str = "192.168.6.1:3333";
#[cfg(feature = "mock")]
const RTC_ADDR: &'static str = "192.168.1.12:3333";

#[cfg(not(feature = "mock"))]
const HTTP_ADDR: ([u8; 4], u16) = ([0, 0, 0, 0], 443);
#[cfg(feature = "mock")]
const HTTP_ADDR: ([u8; 4], u16) = ([0, 0, 0, 0], 8080);

#[cfg(not(feature = "mock"))]
const FILES: &'static str = "/var/www";
#[cfg(feature = "mock")]
const FILES: &'static str = "../webapp/dist";

#[cfg(not(feature = "mock"))]
const FALLBACK: &'static str = "/var/www/index.html";
#[cfg(feature = "mock")]
const FALLBACK: &'static str = "../webapp/dist/index.html";

fn login(payload: bytes::Bytes) -> impl warp::Reply {
    let content = String::from_utf8_lossy(payload.as_ref());
    let auth_user = content.split_once('\n').and_then(|(user, password)| {
        Shadow::from_name(user)
            .map(|entry| verify(password, &entry.password))
            .and_then(|auth| auth.then(|| user))
    });
    if let Some(user) = auth_user {
        let token = user;
        warp::reply::with_header(warp::http::StatusCode::OK,
            "Set-Cookie", format!("token={}; Secure; HttpOnly", token))
    }
    else {
        warp::reply::with_header(warp::http::StatusCode::SEE_OTHER,
            "Location", "/login")
    }
}

#[tokio::main(flavor = "current_thread")]
pub async fn server(measures: Receiver<Answer>,
                    commands: Sender<Command>) {
    let rtc_address = SocketAddr::from_str(RTC_ADDR).unwrap();
    let rtc_server = RtcServer::new(rtc_address, measures, commands).await
        .unwrap();
    let rtc_endpoint = rtc_server.endpoint();
    let rtc_status = rtc_server.status();

    let rtc_endpoint = warp::any().map(move || rtc_endpoint.clone());
    let rtc_status = warp::any().map(move || rtc_status.clone());

    let login = warp::path("login").and(warp::body::content_length_limit(64))
        .and(warp::body::bytes())
        .map(login);

    let webrtc = warp::path("webrtc").and(rtc_endpoint)
                                     .and(warp::body::stream())
                                     .and(warp::post())
                                     .and_then(RtcServer::connect);

    let subscribe = warp::path!("measure" / u32).and(rtc_status.clone())
                                                .and(warp::put())
                                                .and_then(RtcServer::subscribe);
    let unsubscribe =
        warp::path!("measure" / u32).and(rtc_status.clone())
                                    .and(warp::delete())
                                    .and_then(RtcServer::unsubscribe);

    let take_control =
        warp::path!("control" / u32).and(rtc_status.clone())
                                    .and(warp::get())
                                    .and_then(RtcServer::take_control);

    let release_control =
        warp::path!("control" / u32).and(rtc_status)
                                    .and(warp::put())
                                    .and_then(RtcServer::release_control);

    let api = warp::path("api").and(webrtc.or(subscribe)
                                          .or(unsubscribe)
                                          .or(take_control)
                                          .or(release_control));

    let app = warp::fs::dir(FILES).or(warp::fs::file(FALLBACK));

    let http_server = warp::serve(api.or(app)).tls()
        .cert_path("/etc/ssl/drosix.pem")
        .key_path("/etc/ssl/drosix.rsa").run(HTTP_ADDR);
    let rtc_server = rtc_server.run();
    tokio::join!(http_server, rtc_server);
}
