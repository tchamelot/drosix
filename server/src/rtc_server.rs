use futures::{Stream, TryStreamExt};
use std::net::SocketAddr;
use std::io::Error as IoError;
use tokio::select;
use webrtc_unreliable as webrtc;

use warp::{Filter, Reply, Rejection};


pub struct RtcServer {
    server: webrtc::Server,
}

impl RtcServer {
    pub async fn new(addr: SocketAddr) -> Result<Self, IoError> {
        let server = webrtc::Server::new(addr, addr).await?;
        Ok(Self { server})
    }

    pub fn api(&self) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let endpoint = self.server.session_endpoint();
        warp::path("webrtc")
            .and(warp::post())
            .map(move|| endpoint.clone())
            .and(warp::body::stream())
            .and_then(Self::connect)
    }

    async fn connect(mut endpoint: webrtc::SessionEndpoint,
                     body: impl Stream<Item = Result<impl bytes::Buf, warp::Error>>
                         + Send
                         + Sync)
                     -> Result<impl Reply, Rejection> {
        eprintln!("RTC session request");
        match endpoint.http_session_request(body.map_ok(|mut buf| {
            buf.copy_to_bytes(buf.remaining())
        })).await
        {
            Ok(resp) => Ok(resp),
            Err(_) => Err(warp::reject()),
        }
    }

    pub async fn run(mut self) {
        loop {
            select! {
                msg = self.server.recv() => msg.map(|msg| {
                    eprintln!("[RTC] {:#?}", msg.message.as_slice());
                }).unwrap(),
            };
        }
    }

    

}
