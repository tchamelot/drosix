use futures::{FutureExt, Stream, TryStreamExt};
use std::{
    collections::{HashMap, HashSet}, net::SocketAddr, str::FromStr, sync::Arc
};
use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::RwLock;
use tokio::select;
use webrtc_unreliable as webrtc;
use drosix_api::{Answer, Command};

use rkyv::{archived_root, ser::{serializers::AllocSerializer, Serializer}, Infallible, Deserialize};

pub struct RtcStatus {
    clients: HashMap<u32, SocketAddr>,
    pub measure_clients: HashSet<SocketAddr>,
    pub control_client: Option<SocketAddr>,
    next_id: u32,
}

impl RtcStatus {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            control_client: None,
            measure_clients: HashSet::new(),
            next_id: 0
        }
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

pub struct RtcServer {
    server: webrtc::Server,
    status: Arc<RwLock<RtcStatus>>,
    measures: Receiver<Answer>,
    commands: Sender<Command>,
}

impl RtcServer {
    pub async fn new(addr: SocketAddr,
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

    pub fn endpoint(&self) -> webrtc::SessionEndpoint {
        self.server.session_endpoint()
    }

    pub fn status(&self) -> Arc<RwLock<RtcStatus>> {
        self.status.clone()
    }

    pub async fn run(mut self) {
        #[derive(Debug)]
        enum Incoming {
            Webrtc(Command, SocketAddr),
            Drone(Answer),
        }

        loop {
            // Select receiving source (WebRTC or channel)
            // Then process sending (WebRTC or channel
            let received = select! {
                msg = self.server.recv() => msg.map(|msg| {
                        let command = unsafe{ archived_root::<Command>(msg.message.as_ref()) };
                        let command = command.deserialize(&mut Infallible).unwrap();
                        Incoming::Webrtc(command, msg.remote_addr)}),
                // answer = self.measures.recv() => answer
                //     .map(|answer| Incoming::Drone(answer))
                //     .ok_or(IoError::new(IoErrorKind::Other, "")),
            };
            match received {
                Ok(Incoming::Webrtc(Command::ClientHello, addr)) => {//self.receive_webrtc(msg).await,
                    let mut status = self.status.as_ref().write().await;
                    if let Some(id) = status.add_client(addr) {
                        let mut serializer = AllocSerializer::<64>::default();
                        serializer.serialize_value(&Answer::ServerHello(id)).unwrap();
                        let bytes = serializer.into_serializer().into_inner();
                        self.server
                            .send(&bytes,
                                webrtc::MessageType::Binary,
                                &addr)
                            .await
                            .unwrap();
                        println!("New client id {}", id);
                    }
                },
                // TODO better handling
                Ok(Incoming::Webrtc(Command::Flight(v), addr)) => {
                    let status = self.status.as_ref().read().await;
                    if Some(addr) == status.control_client {
                        self.commands.send(Command::Flight(v)).await;
                    }
                },
                Ok(Incoming::Drone(answer)) => {
                    let status = self.status.as_ref().read().await;
                    let mut serializer = AllocSerializer::<64>::default();
                    serializer.serialize_value(&answer).unwrap();
                    let bytes = serializer.into_serializer().into_inner();
                    for client in status.measure_clients.iter() {
                        let _ = self.server
                            .send(&bytes, webrtc::MessageType::Binary, client)
                            .await;
                    }
                },
                _ => (),
            }
        }
    }

    pub async fn connect(mut endpoint: webrtc::SessionEndpoint,
                     body: impl Stream<Item = Result<impl bytes::Buf, warp::Error>>
                         + Send
                         + Sync)
                     -> Result<impl warp::Reply, warp::Rejection> {
        match endpoint.http_session_request(body.map_ok(|mut buf| {
            buf.copy_to_bytes(buf.remaining())
        })).await
        {
            Ok(resp) => Ok(resp),
            Err(_) => Err(warp::reject()),
        }
    }

    pub async fn subscribe(id: u32,
                       status: Arc<RwLock<RtcStatus>>)
                       -> Result<impl warp::Reply, warp::Rejection> {
        let mut status = status.as_ref().write().await;
        Ok(status.subscribe(id))
    }

    pub async fn unsubscribe(id: u32,
                         status: Arc<RwLock<RtcStatus>>)
                         -> Result<impl warp::Reply, warp::Rejection> {
        let mut status = status.as_ref().write().await;
        Ok(status.unsubscribe(id))
    }

    pub async fn take_control(id: u32,
                          status: Arc<RwLock<RtcStatus>>)
                          -> Result<impl warp::Reply, warp::Rejection> {
        let mut status = status.as_ref().write().await;
        Ok(status.take_control(id))
    }

    pub async fn release_control(id: u32,
                             status: Arc<RwLock<RtcStatus>>)
                             -> Result<impl warp::Reply, warp::Rejection> {
        let mut status = status.as_ref().write().await;
        Ok(status.release_control(id))
    }

    pub async fn receive_webrtc(&mut self, message: webrtc::MessageResult<'_>) {
        let command = unsafe{ archived_root::<Command>(message.message.as_ref()) };
        match command.deserialize(&mut Infallible).unwrap() {
            Command::ClientHello => {
                let mut status = self.status.as_ref().write().await;
                if let Some(id) = status.add_client(message.remote_addr) {
                    let mut serializer = AllocSerializer::<64>::default();
                    serializer.serialize_value(&Answer::ServerHello(id)).unwrap();
                    let bytes = serializer.into_serializer().into_inner();
                    self.server
                        .send(&bytes,
                              message.message_type,
                              &message.remote_addr)
                        .await
                        .unwrap();
                    println!("New client id {}", id);
                }
            },
            Command::Flight(v) => {
                let status = self.status.as_ref().read().await;
                if Some(message.remote_addr) == status.control_client {
                    self.commands.send(Command::Flight(v)).await;
                }
            },
            _ => (),
        };
    }

    pub async fn send_webrtc(&mut self, answer: Answer) {
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
