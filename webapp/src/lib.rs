#![feature(get_mut_unchecked)]
#![feature(new_uninit)]
#![recursion_limit = "512"]

//mod agents;
mod app;
//mod components;
mod routes;
//mod services;
mod utils;

use wasm_bindgen::prelude::*;

pub use app::App;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// This is the entry point for the web app
#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    utils::set_panic_hook();
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<app::App>();
    Ok(())
}
