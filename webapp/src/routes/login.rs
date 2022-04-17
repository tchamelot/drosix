use yew::prelude::*;
use reqwasm::http::Request;

#[function_component(Login)]
pub fn login() -> Html {
    let onsubmit = Callback::from(|e: FocusEvent| {
        e.prevent_default();
        log::info!("todo: impl login request");
            wasm_bindgen_futures::spawn_local(async move {
                let auth = Request::post("/api/login").body("username=root&password=toor")
                    .send()
                    .await
                    .map(|response| response.ok())
                    .unwrap_or_default();
                log::info!("auth: {}", auth);
        });
        log::info!("Request sent");
    });

    html! {
        <div class="wrapper">
            <form class="login-form" method="post" action="/api/login" {onsubmit}>
                <h2 class="login-heading">{"Please login"}</h2>
                <input type="text" class="login-input" name="username" placeholder="User" required=true autofocus=true />
                <input type="password" class="login-inout" name="password" placeholder="Password" required=true/>
                <button type="submit">{"Login"}</button>
            </form>
        </div>
    }
}
