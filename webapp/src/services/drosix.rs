use web_sys::{
    MessageEvent, RtcPeerConnection, RtcSdpType,
    RtcSessionDescriptionInit, RtcIceCandidateInit,
    RtcDataChannelInit, RtcDataChannel,
};
use js_sys::Reflect;
use wasm_bindgen_futures::{JsFuture, spawn_local};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use yew_agent::{AgentLink, Agent, Context, HandlerId};
use std::collections::HashSet;
use rkyv::{archived_root, ser::{serializers::AllocSerializer, Serializer}, Infallible, Deserialize};

use drosix_api::{Answer, Command};

use futures::executor::block_on;
use futures::future::ready;

use reqwest::{Client, Body};

pub struct DrosixService {
    link: AgentLink<DrosixService>,
    peer: RtcPeerConnection,
    channel: RtcDataChannel,
    connected: Option<u32>,
    subscriber: HashSet<HandlerId>,
}

pub enum Message {
    Received(Vec<u8>),
    Opened,
    Error,
    Connect(RtcSessionDescriptionInit, RtcIceCandidateInit),
}

pub enum Input {
    Subscribe,
    Unsubscribe,
    TakeControl,
    ReleaseControl,
}

impl Agent for DrosixService {
    type Reach = Context<Self>;
    type Message = Message;
    type Input = Input;
    type Output = Answer;

    fn create(link: AgentLink<Self>) -> Self {
        let peer = RtcPeerConnection::new().unwrap();

        let channel = peer.create_data_channel_with_data_channel_dict("channel", RtcDataChannelInit::new().ordered(false).max_retransmits(0)); 

        let onmessage_cb = link.callback(|data| Message::Received(data));
        let onmessage = Closure::wrap(Box::new(move |event: MessageEvent| {
            let data = js_sys::Uint8Array::new(&event.data()).to_vec();
            onmessage_cb.emit(data);
        }) as Box<dyn FnMut(MessageEvent)>);
        channel.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));

        let onopen_cb = link.callback(|_| Message::Opened);
        let onopen = Closure::wrap(Box::new(move |_: MessageEvent| {
            onopen_cb.emit(());
        }) as Box<dyn FnMut(MessageEvent)>);
        channel.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        
        let onerror_cb = link.callback(|_| Message::Opened);
        let onerror = Closure::wrap(Box::new(move |_: MessageEvent| {
            onerror_cb.emit(());
        }) as Box<dyn FnMut(MessageEvent)>);
        channel.set_onerror(Some(onerror.as_ref().unchecked_ref()));


        let offer = block_on(JsFuture::from(peer.create_offer())).unwrap(); // and_then(|offer|
        let offer_sdp = Reflect::get(&offer, &JsValue::from_str("sdp")).unwrap()
            .as_string()
            .unwrap();
        let mut offer_obj = RtcSessionDescriptionInit::new(RtcSdpType::Offer);
        offer_obj.sdp(&offer_sdp);
        block_on(JsFuture::from(peer.set_local_description(&offer_obj))).unwrap(); // and_then(||)
        let connect_cb = link.callback(|(sdp, candidate)| Message::Connect(sdp, candidate));
        spawn_local(async move {
            let res = Client::new().post("api/webrtc").body(Body::from(offer_sdp)).send().await.unwrap();
            let res = res.error_for_status().unwrap(); // cannot and_then because of await
            let json: serde_json::Value = res.json().await.unwrap();
                
            let sdp = json.get("anwser")
                .and_then(|answer| answer.get("sdp"))
                .and_then(|sdp| sdp.as_str())
                .map(|sdp| String::from(sdp))
                .unwrap_or_default();
            let mut session_desc = RtcSessionDescriptionInit::new(RtcSdpType::Answer);
            session_desc.sdp(&sdp);
                
            let candidate = json.get("candidate");
            let sdp_m_line_index = candidate
                .and_then(|candidate| candidate.get("sdpMLineIndex"))
                .and_then(|x| x.as_u64())
                .map(|x| x as u16);

            let sdp_mid = candidate
                .and_then(|candidate| candidate.get("sdpMid"))
                .and_then(|x| x.as_str());

            let candidate = candidate
                .and_then(|candidate| candidate.get("candidate"))
                .and_then(|x| x.as_str())
                .map(|x| String::from(x))
                .unwrap_or_default();

            let mut candidate = RtcIceCandidateInit::new(&candidate);
            candidate.sdp_m_line_index(sdp_m_line_index).sdp_mid(sdp_mid);

            connect_cb.emit((session_desc, candidate));
        });
        Self{
            link,
            peer,
            channel,
            connected: None,
            subscriber: HashSet::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Message::Received(data) => {
                let answer = unsafe{archived_root::<Answer>(&data)};
                match answer.deserialize(&mut Infallible).unwrap() {
                    Answer::ServerHello(id) => {
                        self.connected = Some(id);
                        log::info!("Got id {}", id);
                    },
                    Answer::Error => {
                        log::info!("Answer error");
                    },
                    x => for sub in self.subscriber.iter() {
                        self.link.respond(*sub, x.clone());
                    }
                }
            },
            Message::Opened => {
                let mut serializer = AllocSerializer::<0>::default();
                serializer.serialize_value(&Command::ClientHello).unwrap();
                let bytes = serializer.into_serializer().into_inner();
                self.channel.send_with_u8_array(&bytes);
            },
            Message::Error => {
                log::info!("Webrtc error");
            },
            Message::Connect(description, candidate) => {
                block_on(JsFuture::from(self.peer.set_remote_description(&description))).unwrap();
                block_on(JsFuture::from(self.peer.add_ice_candidate_with_opt_rtc_ice_candidate_init(Some(&candidate))));
                log::info!("WebRTC channel opened");
            }
        }
    }

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscriber.insert(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscriber.remove(&id);
    }
}
