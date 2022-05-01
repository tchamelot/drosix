use reqwasm::http::Request;
use std::future::Future;

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
