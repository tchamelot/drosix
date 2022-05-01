use yew::prelude::*;
use yewdux::prelude::*;
use wasm_bindgen::JsCast;

use crate::store::{Action, StoreProps};
use crate::api;

#[function_component(Login)]
pub fn login(props: &StoreProps) -> Html {
    let onsubmit = props.future_callback_with(|dispatch, e:FocusEvent| async move {
        e.prevent_default();
        let username = gloo::utils::document()
            .get_element_by_id("username")
            .and_then(|el| el.dyn_into::<web_sys::HtmlInputElement>().ok())
            .map(|input| input.value())
            .unwrap_or_default();
        let password = gloo::utils::document()
            .get_element_by_id("password")
            .and_then(|el| el.dyn_into::<web_sys::HtmlInputElement>().ok())
            .map(|input| input.value())
            .unwrap_or_default();
        let authenticated = api::authenticate(username, password).await;
        dispatch.send(Action::Authenticated(authenticated))
    });

    html! {
        <div class="wrapper">
            <form id="login-form" class="login-form" method="post" action="/api/login" {onsubmit}>
                <h2 class="login-heading">{"Please login"}</h2>
                <input id="username" type="text" class="login-input" name="username" placeholder="User" required=true autofocus=true />
                <input id="password" type="password" class="login-input" name="password" placeholder="Password" required=true/>
                <button type="submit" class="login-button">{"Login"}</button>
            </form>
        </div>
    }
}
