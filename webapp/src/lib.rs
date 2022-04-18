#![recursion_limit = "512"]
use wasm_bindgen::prelude::*;
use yewdux::prelude::WithDispatch;

mod app;
mod routes;
mod store;
mod utils;

pub use app::App;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    utils::set_panic_hook();
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<WithDispatch<App>>();
    Ok(())
}
