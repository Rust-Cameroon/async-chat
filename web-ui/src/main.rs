mod components;
mod services;
mod styles;
mod types;

use components::app::app;

fn main() {
    // Initialize logger for debugging
    wasm_logger::init(wasm_logger::Config::new(log::Level::Debug));
    console_error_panic_hook::set_once();

    // Mount the app to the DOM
    yew::Renderer::<app_component>::new().render();
}
