use yew::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <div>
            <h1>{"🔥 Async Chat"}</h1>
            <p>{"A modern chat interface built with Rust and WebAssembly!"}</p>
            <p>{"Status: Basic component working ✅"}</p>
        </div>
    }
}
