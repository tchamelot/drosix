use futures::future::ready;
use futures_signals::signal::{Mutable, SignalExt};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use wasm_bindgen_futures::spawn_local;
use yew::agent::*;
use yew::format::Nothing;
use yew::services::dialog::DialogService;
use yew::services::fetch::*;
use yew::services::resize::*;
use yew::Callback;

use crate::services::webrtc_binding::*;
use message::DrosixMessage;

#[derive(Debug, Deserialize, Serialize)]
pub struct State {
    pub measures: Mutable<[f32; 3]>,
    pub control: Mutable<[f64; 4]>,
    pub size: Mutable<(i32, i32)>,
}

impl Default for State {
    fn default() -> State {
        let size = WindowDimensions::get_dimensions(&yew::utils::window());
        State {
            measures: Mutable::new([0.0; 3]),
            control: Mutable::new([0.0; 4]),
            size: Mutable::new((size.width, size.height)),
        }
    }
}

pub type ArcState = Arc<State>;

#[derive(Deserialize, Serialize)]
pub enum StoreInput {
    Subscribe,
    Unsubscribe,
    TakeControl,
    ReleaseControl,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum StoreOutput {
    StateInstance(Arc<State>),
}

pub struct Store {
    link: AgentLink<Store>,
    state: ArcState,
    webrtc: WebrtcTask,
    webrtc_id: Option<u32>,
    api: HashMap<TaskId, FetchTask>,
    #[allow(dead_code)]
    size_evt: ResizeTask,
}

type TaskId = Rc<String>;
type TaskBundle = (Rc<String>, bool);

pub enum Msg {
    Control([f64; 4]),
    WebrtcStatus(WebrtcStatus),
    WebrtcReceived(DrosixMessage),
    ApiStatus(TaskBundle),
    Size((i32, i32)),
}

impl Agent for Store {
    type Reach = Context;
    type Message = Msg;
    type Input = StoreInput;
    type Output = StoreOutput;

    fn create(link: AgentLink<Self>) -> Self {
        let state = Arc::new(State::default());

        let cb = link.callback(|control| Msg::Control(control));
        let handler = state.as_ref().control.signal_cloned().for_each(move |u| {
            cb.emit(u);
            ready(())
        });
        spawn_local(handler);

        let webrtc = WebrtcService::new().connect(
            "api/webrtc",
            link.callback(|data| Msg::WebrtcReceived(data)),
            link.callback(|status| Msg::WebrtcStatus(status)),
        );
        let webrtc_id = None;

        let api = HashMap::new();

        let cb = link.callback(|size: WindowDimensions| Msg::Size((size.width, size.height)));
        let size_evt = ResizeService::new().register(cb);

        Self {
            link,
            state,
            webrtc,
            webrtc_id,
            api,
            size_evt,
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Msg::WebrtcReceived(data) => match data {
                DrosixMessage::Measure(data) => {
                    let data = [data[0] as f32, data[1] as f32, data[2] as f32];
                    self.state.as_ref().measures.set(data);
                }
                DrosixMessage::ServerHello(id) => {
                    self.webrtc_id = Some(id);
                    log::info!("Got id {}", id);
                }
                DrosixMessage::Error => log::error!("Webrtc error"),
                _ => (),
            },
            Msg::WebrtcStatus(status) => match status {
                WebrtcStatus::Opened => {
                    self.webrtc.send(DrosixMessage::ClientHello);
                    log::info!("Channel opened");
                }
                WebrtcStatus::Error => {
                    log::error!("Error in webrtc channel");
                }
            },
            Msg::ApiStatus((task_id, status)) => {
                if !status {
                    DialogService::new().alert("Error while accessing api");
                    log::info!("Error while accessig api")
                }
                self.api.remove_entry(task_id.as_ref());
            }
            Msg::Control(val) => {
                self.webrtc.send(DrosixMessage::Control(val));
            }
            Msg::Size(size) => {
                self.state.as_ref().size.set(size);
            }
        }
    }

    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        match msg {
            StoreInput::Subscribe => self.subscribe(),
            StoreInput::Unsubscribe => self.unsubscribe(),
            StoreInput::TakeControl => self.take_control(),
            StoreInput::ReleaseControl => self.release_control(),
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.link
            .respond(id, StoreOutput::StateInstance(self.state.clone()));
    }
}

impl Store {
    fn subscribe(&mut self) {
        if self.webrtc_id.is_some() {
            let request = Request::put(format!(
                //"http://chartreuse:8080/api/measure/{}",
                "/api/measure/{}",
                self.webrtc_id.unwrap()
            ))
            .body(Nothing)
            .unwrap();
            let task_name = Rc::new(String::from("subscriber"));
            if let Ok(task) =
                FetchService::new().fetch(request, self.api_handler(task_name.clone()))
            {
                let task_name = Rc::new(String::from("subscriber"));
                self.api.insert(task_name, task);
            }
        }
    }

    fn unsubscribe(&mut self) {
        if self.webrtc_id.is_some() {
            let request = Request::delete(format!("/api/measure/{}", self.webrtc_id.unwrap()))
                .body(Nothing)
                .unwrap();
            let task_name = Rc::new(String::from("unsubscriber"));
            if let Ok(task) =
                FetchService::new().fetch(request, self.api_handler(task_name.clone()))
            {
                self.api.insert(task_name, task);
            }
        }
    }

    fn take_control(&mut self) {
        if self.webrtc_id.is_some() {
            let request = Request::get(format!("/api/control/{}", self.webrtc_id.unwrap()))
                .body(Nothing)
                .unwrap();
            let task_name = Rc::new(String::from("taker"));
            if let Ok(task) =
                FetchService::new().fetch(request, self.api_handler(task_name.clone()))
            {
                self.api.insert(task_name, task);
            }
        }
    }

    fn release_control(&mut self) {
        if self.webrtc_id.is_some() {
            let request = Request::put(format!("/api/control/{}", self.webrtc_id.unwrap()))
                .body(Nothing)
                .unwrap();
            let task_name = Rc::new(String::from("realeaser"));
            if let Ok(task) =
                FetchService::new().fetch(request, self.api_handler(task_name.clone()))
            {
                self.api.insert(task_name, task);
            }
        }
    }

    fn api_handler(&self, name: Rc<String>) -> Callback<Response<Nothing>> {
        let cb = self.link.callback(|status| Msg::ApiStatus(status));
        let handler = move |response: Response<Nothing>| {
            cb.emit((name.clone(), response.status().is_success()));
        };
        handler.into()
    }
}
