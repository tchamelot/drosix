use js_sys::Promise;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{RtcDataChannel, MessageEvent};
use yew::{Callback, format};

#[wasm_bindgen(module = "/webrtc.js")]
extern "C" {
    type WebrtcBinding;

    #[wasm_bindgen(constructor)]
    fn new() -> WebrtcBinding;

    #[wasm_bindgen(method)]
    fn connect(this: &WebrtcBinding, url: &str) -> Promise;

    #[wasm_bindgen(method, getter)]
    fn channel(this: &WebrtcBinding) -> RtcDataChannel;

    #[wasm_bindgen(method, setter)]
    fn set_channel(this: &WebrtcBinding, channel: RtcDataChannel) -> WebrtcBinding;

    #[wasm_bindgen(method)]
    fn close(this: &WebrtcBinding);
}

pub enum WebrtcStatus {
    Error,
    Opened,
}

pub struct WebrtcTask {
    handle: WebrtcBinding,
    channel: RtcDataChannel,
    _onmessage: Closure<dyn FnMut(MessageEvent) -> ()>,
    _onopen: Closure<dyn FnMut(MessageEvent) -> ()>,
    _onerror: Closure<dyn FnMut(MessageEvent) -> ()>,
}

pub struct WebrtcService {
}

impl WebrtcService {
    pub fn new() -> Self {
        Self{}
    }

    pub fn connect<OUT: 'static>(self, url: &'static str, cb: Callback<OUT>, signal_cb: Callback<WebrtcStatus>) -> WebrtcTask 
    where
        OUT: From<format::Binary>,
    {
        let handle = WebrtcBinding::new();

        let _onmessage = Closure::wrap(Box::new(move |event: MessageEvent| {
            let data = js_sys::Uint8Array::new(&event.data()).to_vec();
            let out = OUT::from(Ok(data));
            cb.emit(out);
        }) as Box<dyn FnMut(MessageEvent)>);

        let open_cb = signal_cb.clone();
        let _onopen = Closure::wrap(Box::new(move |_event: MessageEvent| {
            open_cb.emit(WebrtcStatus::Opened);
        }) as Box<dyn FnMut(MessageEvent)>);

        let error_cb = signal_cb.clone();
        let _onerror = Closure::wrap(Box::new(move |_event: MessageEvent| {
            log::info!("channel error");
            error_cb.emit(WebrtcStatus::Error);
        }) as Box<dyn FnMut(MessageEvent)>);

        let channel = handle.channel();
        channel.set_onmessage(Some(_onmessage.as_ref().unchecked_ref()));
        channel.set_onopen(Some(_onopen.as_ref().unchecked_ref()));
        channel.set_onerror(Some(_onerror.as_ref().unchecked_ref()));

        let prom = handle.connect(url);
        spawn_local(async move {
            let _ = JsFuture::from(prom).await;
        });

        WebrtcTask {handle, channel, _onmessage, _onopen, _onerror}
    }
}

impl WebrtcTask {
    pub fn send<OUT: 'static>(&self, data: OUT)
    where OUT: Into<format::Binary>,
    {
        if let Ok(data) = data.into() {
            let _ = self.channel.send_with_u8_array(&data);
        }
    }
}

impl Drop for WebrtcTask {
    fn drop(&mut self) {
        self.channel.close();
        self.handle.close();
    }
}
