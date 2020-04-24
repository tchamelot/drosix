use gloo::events::{EventListener, EventListenerOptions};
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlElement, MouseEvent, TouchEvent};
use yew::Callback;

pub struct TouchService {}

pub struct TouchTask {
    listeners: [EventListener; 3],
}

impl TouchService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn touchscreen(self, element: &HtmlElement, cb: Callback<TouchEvent>) -> TouchTask {
        let notify = cb.clone();
        let start_event = move |event: &Event| {
            notify.emit(event.unchecked_ref::<TouchEvent>().clone());
        };
        let notify = cb.clone();
        let stop_event = move |event: &Event| {
            notify.emit(event.unchecked_ref::<TouchEvent>().clone());
        };
        let notify = cb.clone();
        let move_event = move |event: &Event| {
            notify.emit(event.unchecked_ref::<TouchEvent>().clone());
        };

        let opt = EventListenerOptions::enable_prevent_default();
        let start =
            EventListener::new_with_options(element.as_ref(), "touchstart", opt, start_event);
        let stop = EventListener::new_with_options(element.as_ref(), "touchend", opt, stop_event);
        let move_ = EventListener::new_with_options(element.as_ref(), "touchmove", opt, move_event);
        TouchTask {
            listeners: [start, stop, move_],
        }
    }

    pub fn mouse(self, element: &HtmlElement, cb: Callback<MouseEvent>) -> TouchTask {
        let notify = cb.clone();
        let start_event =
            move |event: &Event| notify.emit(event.unchecked_ref::<MouseEvent>().clone());
        let notify = cb.clone();
        let stop_event = move |event: &Event| {
            notify.emit(event.unchecked_ref::<MouseEvent>().clone());
        };
        let notify = cb.clone();
        let move_event = move |event: &Event| {
            notify.emit(event.unchecked_ref::<MouseEvent>().clone());
        };

        let opt = EventListenerOptions::enable_prevent_default();
        let start =
            EventListener::new_with_options(element.as_ref(), "mousedown", opt, start_event);
        let stop = EventListener::new_with_options(element.as_ref(), "mouseup", opt, stop_event);
        let move_ = EventListener::new_with_options(element.as_ref(), "mousemove", opt, move_event);
        TouchTask {
            listeners: [start, stop, move_],
        }
    }
}
