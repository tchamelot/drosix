use yew::agent::Bridged;
use yew::{html, Bridge, Component, ComponentLink, Html, NodeRef, ShouldRender};

use crate::agents::store::*;
use crate::components::joystick;

pub struct Controller {
    link: ComponentLink<Self>,
    store: Box<dyn Bridge<Store>>,
    state: Option<ArcState>,
    control: [f64; 4],
}

pub enum Msg {
    FromStore(StoreOutput),
    Left((f64, f64)),
    Right((f64, f64)),
}

impl Component for Controller {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let store = Store::bridge(link.callback(|d| Msg::FromStore(d)));
        Controller {
            link: link,
            store: store,
            state: None,
            control: [0.0; 4],
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::FromStore(s) => match s {
                StoreOutput::StateInstance(state) => {
                    self.state = Some(state);
                    //self.register_state_handlers();
                    false
                }
            },
            Msg::Left(data) => {
                self.control[0] = data.1;
                self.control[3] = data.0;
                self.state.as_ref().unwrap().control.set(self.control);
                false
            }
            Msg::Right(data) => {
                self.control[1] = data.1;
                self.control[2] = data.0;
                self.state.as_ref().unwrap().control.set(self.control);
                false
            }
        }
    }

    fn mounted(&mut self) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let left = NodeRef::default();
        let right = NodeRef::default();
        html! {
            <div class="main row">
                <div class="joystick_containter" ref=left.clone()>
                    <joystick::Joystick parent=left.clone() onmove=self.link.callback(|u| Msg::Left(u))/>
                </div>
                <div class="joystick_containter" ref=right.clone()>
                    <joystick::Joystick parent=right.clone() onmove=self.link.callback(|u| Msg::Right(u))/>
                </div>
            </div>
        }
    }
}

impl Controller {
    // fn register_state_handlers(&self) {
    //     let state = self.state.as_ref().unwrap();

    //     let callback = self.link.callback(|size| Msg::Resize(size));
    //     let handler = state.size.signal_cloned().for_each(move |u| {
    //         callback.emit(u);
    //         ready(())
    //     });
    //     spawn_local(handler);
    // }
}
