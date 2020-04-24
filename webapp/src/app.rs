use circular_queue::CircularQueue;
use futures::future::ready;
use futures_signals::signal::SignalExt;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::agent::Bridged;
use yew::services::resize::WindowDimensions;
use yew::{html, Bridge, Component, ComponentLink, Html, NodeRef, ShouldRender};

use crate::agents::store::*;
use crate::components::{chart, joystick};

pub struct App {
    link: ComponentLink<Self>,
    dimension: Option<(i32, i32)>,
    chart_cb: chart::ChartCallback,
    data: Vec<Rc<CircularQueue<f32>>>,
    store: Box<dyn Bridge<Store>>,
    state: Option<ArcState>,
    subscribed: bool,
}

pub enum Msg {
    Resize((i32, i32)),
    Subscribe,
    Drosix([f32; 3]),
    FromStore(StoreOutput),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let store = Store::bridge(link.callback(|d| Msg::FromStore(d)));
        App {
            link: link,
            chart_cb: chart::ChartCallback::default(),
            dimension: None,
            data: Vec::new(),
            store: store,
            state: None,
            subscribed: false,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::FromStore(s) => match s {
                StoreOutput::StateInstance(state) => {
                    self.state = Some(state);
                    self.register_state_handlers();
                    false
                }
            },
            Msg::Drosix(data) => {
                if data.len() > self.data.len() {
                    for _ in 0..data.len() - self.data.len() {
                        self.data.push(Rc::new(CircularQueue::with_capacity(100)));
                    }
                }

                for (measure, val) in self.data.iter_mut().zip(data.iter()) {
                    if let Some(measure) = Rc::get_mut(measure) {
                        measure.push(*val);
                    }
                }
                self.chart_cb
                    .as_ref()
                    .borrow()
                    .emit(self.data.iter().map(|x| x.clone()).collect());
                false
            }
            Msg::Resize(dimension) => {
                self.dimension = Some(dimension);
                true
            }
            Msg::Subscribe => {
                if self.subscribed {
                    self.store.send(StoreInput::Unsubscribe);
                } else {
                    self.store.send(StoreInput::Subscribe);
                }
                self.subscribed = !self.subscribed;
                true
            }
        }
    }

    fn mounted(&mut self) -> ShouldRender {
        let size = WindowDimensions::get_dimensions(&web_sys::window().unwrap());
        self.dimension = Some((size.width, size.height));
        true
    }

    fn view(&self) -> Html {
        let node = NodeRef::default();
        html! {
            <div ref=node.clone() class="main">
                <chart::Chart
                    width={self.dimension.as_ref().and_then(|size| Some(size.0 * 60/100))}
                    height={self.dimension.as_ref().and_then(|size| Some(size.1 * 60/100))}
                    style="canvas"
                    labels=Some("alpha beta gamma")
                    cb=&self.chart_cb/>
                { self.view_subscribe() }
                <joystick::Joystick parent=node.clone()/>
            </div>
        }
    }
}

impl App {
    fn view_subscribe(&self) -> Html {
        let action = if self.subscribed {
            "unsubscribed"
        } else {
            "subscribed"
        };
        html! {
            <button class="button" type="button" disabled=false
                onclick=&self.link.callback(|_| Msg::Subscribe)>{action}</button>
        }
    }

    fn register_state_handlers(&self) {
        let state = self.state.as_ref().unwrap();

        // Listen for new measures
        let callback = self.link.callback(|data| Msg::Drosix(data));
        let handler = state.measures.signal_cloned().for_each(move |u| {
            callback.emit(u);
            ready(())
        });
        spawn_local(handler);

        // Listen for resize event
        let callback = self.link.callback(|size| Msg::Resize(size));
        let handler = state.size.signal_cloned().for_each(move |u| {
            callback.emit(u);
            ready(())
        });
        spawn_local(handler);
    }
}
