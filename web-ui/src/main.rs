mod components;
mod services;
mod styles;
mod types;

use components::app::App;
use wasm_logger::init;
use yew::Renderer;

fn main() {
    // Initialize logger for debugging
    init(wasm_logger::Config::new(log::Level::Debug));
    console_error_panic_hook::set_once();

    // Mount the app to the DOM
    Renderer::<App>::new().render();
}