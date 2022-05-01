use reqwasm::http::Request;
use std::future::Future;
use web_sys::{
    MessageEvent, RtcPeerConnection, RtcSdpType,
    RtcSessionDescriptionInit, RtcIceCandidateInit,
    RtcDataChannelInit, RtcDataChannel,
};
use js_sys::Reflect;
use wasm_bindgen_futures::{JsFuture, spawn_local};
use wasm_bindgen::prelude::{JsValue, Closure};
use wasm_bindgen::JsCast;
use serde_json as json;

use yewdux::prelude::*;
use crate::store::{Action, Store};

pub fn authenticate(username: String, password: String) -> impl Future<Output = bool> {
    async move {
        Request::post("/api/login")
                .body(format!("username={}&password={}", username, password))
                .send()
                .await
                .map(|response| response.ok())
                .unwrap_or_default()
        }
}

pub fn webrtc_connect(offer_sdp: String) -> impl Future<Output = json::Value> {
    async move {
        let response = Request::post("api/webrtc")
            .body(offer_sdp)
            .send()
            .await;
        match response {
            Ok(response) => response.json().await.unwrap_or_default(),
            Err(_) => json::Value::default(),
        }
    }
}

#[derive(Clone)]
pub struct Webrtc {
    dispatch: Dispatch<Store>,
    peer: RtcPeerConnection,
    channel: RtcDataChannel,
}

impl Webrtc {
    pub fn new() -> Self {
        let dispatch = Dispatch::new();
        let peer = RtcPeerConnection::new().unwrap();

        let channel = peer.create_data_channel_with_data_channel_dict("channel", RtcDataChannelInit::new().ordered(false).max_retransmits(0)); 

        let onopen_cb = dispatch.callback(|_| Action::WebrtcStatus);
        let onopen = Closure::wrap(Box::new(move |_: MessageEvent| {
            onopen_cb.emit(());
        }) as Box<dyn FnMut(MessageEvent)>);
        channel.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        onopen.forget();

        let onmessage_cb = dispatch.callback(|data| Self::receive(data));
        let onmessage = Closure::wrap(Box::new(move |event: MessageEvent| {
            let data = js_sys::Uint8Array::new(&event.data()).to_vec();
            onmessage_cb.emit(data);
        }) as Box<dyn FnMut(MessageEvent)>);
        channel.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();

        let peer_clone = peer.clone();
        spawn_local(async move {
            let offer = JsFuture::from(peer_clone.create_offer()).await.unwrap();
            let offer_sdp = Reflect::get(&offer, &JsValue::from_str("sdp")).unwrap()
                .as_string()
                .unwrap();
            let mut offer_obj = RtcSessionDescriptionInit::new(RtcSdpType::Offer);
            offer_obj.sdp(&offer_sdp);

            JsFuture::from(peer_clone.set_local_description(&offer_obj)).await.unwrap();

            let json = webrtc_connect(offer_sdp).await;
            let sdp = json.get("answer")
                .and_then(|answer| {
                    answer.get("sdp")
                })
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

            JsFuture::from(peer_clone.set_remote_description(&session_desc)).await.unwrap();
            JsFuture::from(peer_clone.add_ice_candidate_with_opt_rtc_ice_candidate_init(Some(&candidate))).await;
        });

        Self{
            dispatch,
            peer,
            channel,
        }

    }
    fn receive(_data: Vec<u8>) -> Action {
        Action::WebrtcIn
    }
}
