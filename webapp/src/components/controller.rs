use yew::agent::Bridged;
use yew::{html, Bridge, Component, ComponentLink, Html, NodeRef, ShouldRender};

use crate::agents::store::*;
use crate::components::joystick;

pub struct Controller {
    link: ComponentLink<Self>,
    store: Box<dyn Bridge<Store>>,
    state: Option<ArcState>,
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
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::FromStore(s) => match s {
                StoreOutput::StateInstance(state) => {
                    self.state = Some(state);
                    self.store.send(StoreInput::TakeControl);
                    //self.register_state_handlers();
                    false
                }
            },
            Msg::Left(data) => {
                let thrust = if data.1 >= 0.0 {
                    data.1 * 2.0
                } else {
                    0.0
                };
                let control = [thrust, 0.0, 0.0, data.0];
                self.state.as_ref().unwrap().control.set(control);
                false
            }
            Msg::Right(data) => {
                let control = [0.0, data.1, data.0, 0.0];
                self.state.as_ref().unwrap().control.set(control);
                false
            }
        }
    }

    fn mounted(&mut self) -> ShouldRender {
        false
    }

    fn destroy(&mut self) {
        log::info!("controller destroyed");
        self.store.send(StoreInput::ReleaseControl);
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
