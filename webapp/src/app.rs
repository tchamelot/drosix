use circular_queue::CircularQueue;
use std::cell::RefCell;
use std::rc::Rc;
use yew::agent::{AgentScope, Bridged};
use yew::{html, services, Bridge, Component, ComponentLink, Html, NodeRef, ShouldRender};

use crate::agents::drosix::{Action, DrosixAgent};
use crate::components::{chart, joystick};

pub type WeakComponentLink<COMP> = Rc<RefCell<Option<ComponentLink<COMP>>>>;

pub struct Measure {
    pub data: Vec<f32>,
}

impl From<yew::format::Text> for Measure {
    fn from(msg: yew::format::Text) -> Self {
        match msg {
            Ok(text) => Measure {
                data: text
                    .lines()
                    .map(|line| line.parse::<f32>().unwrap_or(0.0))
                    .collect(),
            },
            Err(_) => Measure { data: Vec::new() },
        }
    }
}

pub struct App {
    link: ComponentLink<Self>,
    _evt_resize: services::resize::ResizeTask,
    dimension: Option<services::resize::WindowDimensions>,
    chart_link: WeakComponentLink<chart::Chart>,
    data: Vec<Rc<CircularQueue<f32>>>,
    drosix: Box<dyn Bridge<DrosixAgent>>,
    subscribed: bool,
}

pub enum Msg {
    Resize(services::resize::WindowDimensions),
    Subscribe,
    Drosix([f32; 3]),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let resize_cb = link.callback(|size: services::resize::WindowDimensions| Msg::Resize(size));
        let evt_resize = services::ResizeService::new().register(resize_cb);
        let drosix_cb = link.callback(|data| Msg::Drosix(data));
        let drosix = DrosixAgent::bridge(drosix_cb);
        App {
            link: link,
            _evt_resize: evt_resize,
            chart_link: WeakComponentLink::<chart::Chart>::default(),
            dimension: None,
            data: Vec::new(),
            drosix: drosix,
            subscribed: false,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
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
                let link = self.chart_link.borrow().clone().unwrap();
                link.callback(|data: Vec<Rc<CircularQueue<f32>>>| chart::Msg::NewData(data))
                    .emit(self.data.iter().map(|x| x.clone()).collect());
                false
            }
            Msg::Resize(dimension) => {
                self.dimension = Some(dimension);
                true
            }
            Msg::Subscribe => {
                if self.subscribed {
                    self.drosix.send(Action::Unsubscribe);
                } else {
                    self.drosix.send(Action::Subscribe);
                }
                self.subscribed = !self.subscribed;
                true
            }
        }
    }

    fn mounted(&mut self) -> ShouldRender {
        self.dimension = Some(services::resize::WindowDimensions::get_dimensions(
            &web_sys::window().unwrap(),
        ));
        true
    }

    fn view(&self) -> Html {
        let node = NodeRef::default();
        html! {
            <div ref=node.clone() class="main">
                <chart::Chart
                    width={self.dimension.as_ref().and_then(|d| Some(d.width * 60/100))}
                    height={self.dimension.as_ref().and_then(|d| Some(d.height * 60/100))}
                    style="canvas"
                    labels=Some("alpha beta gamma")
                    link=&self.chart_link />
                { self.view_subscribe() }
                <joystick::Joystick parent=node.clone() agent=AgentScope::<DrosixAgent>new()/>
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
}
