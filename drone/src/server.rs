use futures::{FutureExt, Stream, TryStreamExt};
// use std::sync::mpsc;
use std::{
    collections::{HashMap, HashSet}, io::Error as IoError, net::SocketAddr, str::FromStr, sync::Arc
};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::RwLock;
use tokio::select;
use warp::Filter;
use webrtc_unreliable as webrtc;
use drosix_api::{Answer, Command};

use rkyv::{archived_root, ser::{serializers::AllocSerializer, Serializer}, Infallible, Deserialize};

#[cfg(not(feature = "mock"))]
const RTC_ADDR: &'static str = "192.168.6.1:3333";
#[cfg(feature = "mock")]
const RTC_ADDR: &'static str = "192.168.1.12:3333";

#[cfg(not(feature = "mock"))]
const HTTP_ADDR: ([u8; 4], u16) = ([0, 0, 0, 0], 80);
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

struct RtcStatus {
    clients: HashMap<u32, SocketAddr>,
    pub measure_clients: HashSet<SocketAddr>,
    pub control_client: Option<SocketAddr>,
    next_id: u32,
}

impl RtcStatus {
    pub fn new() -> Self {
        Self { clients: HashMap::new(),
               control_client: None,
               measure_clients: HashSet::new(),
               next_id: 0 }
    }

    pub fn add_client(&mut self, addr: SocketAddr) -> Option<u32> {
        if !self.clients.values().any(|&client_addr| client_addr == addr) {
            self.clients.insert(self.next_id, addr);
            self.next_id += 1;
            Some(self.next_id - 1)
        } else {
            None
        }
    }

    pub fn subscribe(&mut self, id: u32) -> warp::http::StatusCode {
        if let Some(client) = self.clients.get(&id) {
            println!("Client {} subscribed", id);
            self.measure_clients.insert(*client);
            warp::http::StatusCode::CREATED
        } else {
            warp::http::StatusCode::BAD_REQUEST
        }
    }

    pub fn unsubscribe(&mut self, id: u32) -> warp::http::StatusCode {
        if let Some(client) = self.clients.get(&id) {
            println!("Client {} unsubscribed", id);
            if self.measure_clients.remove(client) {
                warp::http::StatusCode::NO_CONTENT
            } else {
                warp::http::StatusCode::BAD_REQUEST
            }
        } else {
            warp::http::StatusCode::BAD_REQUEST
        }
    }

    pub fn take_control(&mut self, id: u32) -> warp::http::StatusCode {
        println!("cliend {} try to take control", id);
        if let (None, Some(client)) =
            (self.control_client, self.clients.get(&id))
        {
            println!("cliend {} take control", id);
            self.control_client = Some(*client);
            warp::http::StatusCode::OK
        } else {
            warp::http::StatusCode::BAD_REQUEST
        }
    }

    pub fn release_control(&mut self, id: u32) -> warp::http::StatusCode {
        println!("cliend {} try to release control", id);
        if self.control_client.as_ref() == self.clients.get(&id) {
            println!("cliend {} release control", id);
            self.control_client = None;
            warp::http::StatusCode::OK
        } else {
            warp::http::StatusCode::BAD_REQUEST
        }
    }
}

struct RtcServer {
    server: webrtc::Server,
    status: Arc<RwLock<RtcStatus>>,
    measures: Receiver<Answer>,
    commands: Sender<Command>,
}

impl RtcServer {
    async fn new(addr: SocketAddr,
                 measures: Receiver<Answer>,
                 commands: Sender<Command>)
                 -> Result<Self, IoError> {
        let server = webrtc::Server::new(addr, addr).await?;
        let status = Arc::new(RwLock::new(RtcStatus::new()));
        Ok(RtcServer { server,
                       status,
                       measures,
                       commands })
    }

    fn endpoint(&self) -> webrtc::SessionEndpoint {
        self.server.session_endpoint()
    }

    fn status(&self) -> Arc<RwLock<RtcStatus>> {
        self.status.clone()
    }

    async fn run(mut self) {
        let mut buffer = vec![0; 64];
        loop {
            select! {
                status = self.server.recv(&mut buffer).fuse() => match status {
                    Ok(status) => self.receive(status, &buffer).await,
                    _ => (),
                },
                data = self.measures.recv() => match data {
                    Some(answer) => self.send(answer).await,
                    _ => (),
                }
            }
        }
    }

    async fn connect(mut endpoint: webrtc::SessionEndpoint,
                     body: impl Stream<Item = Result<impl bytes::Buf, warp::Error>>
                         + Send
                         + Sync)
                     -> Result<impl warp::Reply, warp::Rejection> {
        match endpoint.http_session_request(body.map_ok(|mut buf| {
                                                    buf.to_bytes()
                                                }))
                      .await
        {
            Ok(resp) => Ok(resp),
            Err(_) => Err(warp::reject()),
        }
    }

    async fn subscribe(id: u32,
                       status: Arc<RwLock<RtcStatus>>)
                       -> Result<impl warp::Reply, warp::Rejection> {
        let mut status = status.as_ref().write().await;
        Ok(status.subscribe(id))
    }

    async fn unsubscribe(id: u32,
                         status: Arc<RwLock<RtcStatus>>)
                         -> Result<impl warp::Reply, warp::Rejection> {
        let mut status = status.as_ref().write().await;
        Ok(status.unsubscribe(id))
    }

    async fn take_control(id: u32,
                          status: Arc<RwLock<RtcStatus>>)
                          -> Result<impl warp::Reply, warp::Rejection> {
        let mut status = status.as_ref().write().await;
        Ok(status.take_control(id))
    }

    async fn release_control(id: u32,
                             status: Arc<RwLock<RtcStatus>>)
                             -> Result<impl warp::Reply, warp::Rejection> {
        let mut status = status.as_ref().write().await;
        Ok(status.release_control(id))
    }

    async fn receive(&mut self,
                     msg_status: webrtc::MessageResult,
                     data: &[u8]) {

        let msg = unsafe{ archived_root::<Command>(data) };
        match msg.deserialize(&mut Infallible).unwrap() {
            Command::ClientHello => {
                let mut status = self.status.as_ref().write().await;
                if let Some(id) = status.add_client(msg_status.remote_addr) {
                    let mut serializer = AllocSerializer::<64>::default();
                    serializer.serialize_value(&Answer::ServerHello(id)).unwrap();
                    let bytes = serializer.into_serializer().into_inner();
                    self.server
                        .send(&bytes,
                              msg_status.message_type,
                              &msg_status.remote_addr)
                        .await
                        .unwrap();
                    println!("New client id {}", id);
                }
            },
            Command::Flight(v) => {
                let status = self.status.as_ref().read().await;
                if Some(msg_status.remote_addr) == status.control_client {
                    self.commands.send(Command::Flight(v)).await;
                }
            },
            _ => (),
        };
    }

    async fn send(&mut self, answer: Answer) {
        let status = self.status.as_ref().read().await;
        let mut serializer = AllocSerializer::<64>::default();
        serializer.serialize_value(&answer).unwrap();
        let bytes = serializer.into_serializer().into_inner();
        for client in status.measure_clients.iter() {
            let _ = self.server
                        .send(&bytes, webrtc::MessageType::Binary, client)
                        .await;
        }
    }
}

#[tokio::main(flavor = "current_thread")]
pub async fn server(measures: Receiver<Answer>,
                    commands: Sender<Command>) {
    let rtc_address = SocketAddr::from_str(RTC_ADDR).unwrap();
    let rtc_server =
        RtcServer::new(rtc_address, measures, commands).await
                                                                   .unwrap();
    let rtc_endpoint = rtc_server.endpoint();
    let rtc_status = rtc_server.status();

    let rtc_endpoint = warp::any().map(move || rtc_endpoint.clone());
    let rtc_status = warp::any().map(move || rtc_status.clone());

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

    let http_server = warp::serve(api.or(app)).run(HTTP_ADDR);
    let rtc_server = rtc_server.run();
    tokio::join!(http_server, rtc_server);
}
