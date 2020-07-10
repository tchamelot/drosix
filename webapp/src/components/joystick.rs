use crate::services::touch::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlElement, TouchEvent};
use yew::prelude::*;

pub struct Joystick {
    link: ComponentLink<Self>,
    parent: NodeRef,
    onmove: Option<Callback<(f64, f64)>>,
    #[allow(dead_code)]
    task: TouchTask,
    active: Option<i32>,
    position: (i32, i32),
    delta: (f64, f64),
    scale: Option<f64>,
}

pub enum Msg {
    TouchEvent(TouchEvent),
}

#[derive(Clone, Properties)]
pub struct Props {
    pub parent: NodeRef,
    #[prop_or_default]
    pub onmove: Option<Callback<(f64, f64)>>,
}

impl Component for Joystick {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let cb = link.callback(|event| Msg::TouchEvent(event));
        let container = props.parent.cast::<HtmlElement>().unwrap();
        let task = TouchService::new().touchscreen(&container, cb);
        Self {
            link,
            parent: props.parent,
            onmove: props.onmove,
            task,
            active: None,
            position: (0, 0),
            delta: (0.0, 0.0),
            scale: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::TouchEvent(event) => match event
                .unchecked_ref::<Event>()
                .type_()
                .as_str()
            {
                "touchstart" => {
                    self.scale = self.scale.or(self
                        .parent
                        .cast::<HtmlElement>()
                        .map(|el| f64::from(el.offset_width()) / 500.0));
                    let touch = event.changed_touches().get(0).unwrap();
                    self.active = Some(touch.identifier());
                    let x = touch.client_x();
                    let y = touch.client_y();
                    self.position = (x, y);
                    false
                },
                "touchend" => {
                    if let Some(id) = self.active {
                        let touches = event.changed_touches();
                        // if only iterator was implemented :/
                        for i in 0..touches.length() {
                            if id == touches.get(i).unwrap().identifier() {
                                self.active = None;
                                self.delta = (0.0, 0.0);
                                break;
                            }
                        }
                    }
                    self.active.is_none()
                },
                "touchmove" => {
                    if let Some(id) = self.active {
                        let touches = event.changed_touches();
                        let mut touch = None;
                        // if only iterator was implemented :/
                        for i in 0..touches.length() {
                            let touch_it = touches.get(i).unwrap();
                            if id == touch_it.identifier() {
                                touch = Some(touch_it);
                                break;
                            }
                        }
                        let touch = touch.unwrap();

                        let scale = self.scale.unwrap_or(100.0);
                        let dx = f64::from(self.position.0 - touch.client_x())
                            / scale;
                        let dy = f64::from(self.position.1 - touch.client_y())
                            / scale;
                        let (dx, dy) = if dx.powi(2) + dy.powi(2) > 2500.0 {
                            let alpha = dx.atan2(dy);
                            (50.0 * alpha.sin(), 50.0 * alpha.cos())
                        } else {
                            (dx, dy)
                        };
                        self.delta = (dx, dy);
                        if let Some(cb) = self.onmove.as_ref() {
                            cb.emit(self.delta);
                        }
                        true
                    } else {
                        false
                    }
                },
                _ => false,
            },
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        self.scale = None;
        false
    }

    fn view(&self) -> Html {
        if true {
            html! {
                <div class="joystick_base">
                    <div class="joystick_top" style=self.position().as_str()/>
                </div>
            }
        } else {
            html! {}
        }
    }
}

impl Joystick {
    fn position(&self) -> String {
        format!(
            "left:{:.2}%;top:{:.2}%;",
            50.0 - self.delta.0,
            50.0 - self.delta.1,
        )
    }
}
