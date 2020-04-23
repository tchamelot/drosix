use crate::services::webrtc_binding::*;
use message::DrosixMessage;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use yew::format::Nothing;
use yew::services::fetch::*;
use yew::worker::*;
use yew::Callback;

#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    Subscribe,
    Unsubscribe,
}

pub enum Msg {
    Received(DrosixMessage),
    Status(WebrtcStatus),
    Api(bool),
}

pub struct DrosixAgent {
    link: AgentLink<Self>,
    subscribers: HashSet<HandlerId>,
    handle: WebrtcTask,
    api: Option<FetchTask>,
    id: Option<u32>,
}

impl Agent for DrosixAgent {
    type Reach = Context;
    type Message = Msg;
    type Input = Action;
    type Output = [f32; 3];
    fn create(link: AgentLink<Self>) -> Self {
        let handle = WebrtcService::new().connect(
            "api/webrtc",
            link.callback(|data| Msg::Received(data)),
            link.callback(|status| Msg::Status(status)),
        );
        Self {
            link: link,
            subscribers: HashSet::new(),
            handle: handle,
            id: None,
            api: None,
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Msg::Received(data) => match data {
                DrosixMessage::Measure(data) => {
                    let data = [data[0] as f32, data[1] as f32, data[2] as f32];
                    for sub in self.subscribers.iter() {
                        self.link.respond(*sub, data);
                    }
                }
                DrosixMessage::ServerHello(id) => {
                    self.id = Some(id);
                    log::info!("Got id {}", id);
                }
                DrosixMessage::Error => log::error!("Webrtc error"),
                _ => (),
            },
            Msg::Status(status) => match status {
                WebrtcStatus::Opened => {
                    self.handle.send(DrosixMessage::ClientHello);
                    log::info!("Channel opened");
                }
                WebrtcStatus::Error => {
                    log::error!("Error in webrtc channel");
                }
            },
            Msg::Api(status) => {
                self.api = None;
                if !status {
                    log::info!("Error while accessig api")
                }
            }
        }
    }

    fn handle_input(&mut self, msg: Self::Input, _: HandlerId) {
        match msg {
            Action::Subscribe => {
                self.subscribe();
            }
            Action::Unsubscribe => {
                self.unsubscribe();
            }
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}

impl DrosixAgent {
    fn subscribe(&mut self) {
        if self.api.is_none() && self.id.is_some() {
            let request = Request::put(format!(
                "http://chartreuse:8080/api/measure/{}",
                self.id.unwrap()
            ))
            .body(Nothing)
            .unwrap();
            self.api = Some(
                FetchService::new()
                    .fetch(request, self.api_handler())
                    .unwrap(),
            );
        }
    }

    fn unsubscribe(&mut self) {
        if self.api.is_none() && self.id.is_some() {
            let request = Request::delete(format!(
                "http://chartreuse:8080/api/measure/{}",
                self.id.unwrap()
            ))
            .body(Nothing)
            .unwrap();
            self.api = Some(
                FetchService::new()
                    .fetch(request, self.api_handler())
                    .unwrap(),
            );
        }
    }

    fn api_handler(&self) -> Callback<Response<Nothing>> {
        let cb = self.link.callback(|ok| Msg::Api(ok));
        let handler = move |response: Response<Nothing>| {
            cb.emit(response.status().is_success());
        };
        handler.into()
    }
}
