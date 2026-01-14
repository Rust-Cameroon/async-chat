use yew::prelude::*;
use gloo_net::websocket::futures::WebSocket;
use gloo_net::websocket::Message;
use futures_util::{StreamExt, SinkExt};
use wasm_bindgen_futures::spawn_local;
use async_chat::{FromClient, FromServer};
use std::sync::Arc;

#[function_component(App)]
pub fn app() -> Html {
    let messages = use_state(|| Vec::new());
    let input_ref = use_node_ref();
    let group_ref = use_node_ref();
    let ws_state = use_state(|| None::<WebSocket>);

    // Handle incoming messages
    {
        let messages = messages.clone();
        let ws_state = ws_state.clone();
        use_effect_with(ws_state, move |ws| {
            if let Some(ws) = &**ws {
                let mut ws = ws.clone();
                let messages = messages.clone();
                spawn_local(async move {
                    while let Some(msg) = ws.next().await {
                        if let Ok(Message::Text(text)) = msg {
                            if let Ok(server_msg) = serde_json::from_str::<FromServer>(&text) {
                                let mut new_messages = (*messages).clone();
                                match server_msg {
                                    FromServer::Message { group_name, message } => {
                                        new_messages.push(format!("[{}]: {}", group_name, message));
                                    }
                                    FromServer::Error(err) => {
                                        new_messages.push(format!("Error: {}", err));
                                    }
                                }
                                messages.set(new_messages);
                            }
                        }
                    }
                });
            }
            || ()
        });
    }

    let on_join = {
        let group_ref = group_ref.clone();
        let ws_state = ws_state.clone();
        let messages = messages.clone();
        Callback::from(move |_| {
            let group_name = group_ref.cast::<web_sys::HtmlInputElement>().unwrap().value();
            if group_name.is_empty() { return; }
            
            let ws_state = ws_state.clone();
            let messages = messages.clone();
            spawn_local(async move {
                if ws_state.is_none() {
                    let ws = WebSocket::open("ws://127.0.0.1:8000").unwrap();
                    ws_state.set(Some(ws.clone()));
                    
                    let join_msg = FromClient::Join { group_name: Arc::new(group_name) };
                    let mut ws = ws.clone();
                    ws.send(Message::Text(serde_json::to_string(&join_msg).unwrap())).await.unwrap();
                } else if let Some(ws) = &*ws_state {
                     let join_msg = FromClient::Join { group_name: Arc::new(group_name) };
                     let mut ws = ws.clone();
                     ws.send(Message::Text(serde_json::to_string(&join_msg).unwrap())).await.unwrap();
                }
            });
        })
    };

    let on_send = {
        let input_ref = input_ref.clone();
        let group_ref = group_ref.clone();
        let ws_state = ws_state.clone();
        Callback::from(move |_| {
            let message = input_ref.cast::<web_sys::HtmlInputElement>().unwrap().value();
            let group_name = group_ref.cast::<web_sys::HtmlInputElement>().unwrap().value();
            if message.is_empty() || group_name.is_empty() { return; }
            
            if let Some(ws) = &*ws_state {
                let ws = ws.clone();
                let post_msg = FromClient::Post { 
                    group_name: Arc::new(group_name),
                    message: Arc::new(message)
                };
                spawn_local(async move {
                    let mut ws = ws.clone();
                    ws.send(Message::Text(serde_json::to_string(&post_msg).unwrap())).await.unwrap();
                });
                input_ref.cast::<web_sys::HtmlInputElement>().unwrap().set_value("");
            }
        })
    };

    html! {
        <div style="padding: 20px; font-family: sans-serif; max-width: 600px; margin: 0 auto;">
            <h1 style="color: #e67e22;">{"🔥 Async Chat"}</h1>
            
            <div style="margin-bottom: 20px; display: flex; gap: 10px;">
                <input ref={group_ref} placeholder="Group Name (e.g. Dogs)" style="padding: 8px; flex: 1;" />
                <button onclick={on_join} style="padding: 8px 16px; background: #3498db; color: white; border: none; cursor: pointer;">{"Join"}</button>
            </div>

            <div style="border: 1px solid #ddd; height: 300px; overflow-y: auto; padding: 10px; margin-bottom: 20px; background: #f9f9f9;">
                { for (*messages).iter().map(|m| html! { <div style="margin-bottom: 5px;">{ m }</div> }) }
            </div>

            <div style="display: flex; gap: 10px;">
                <input ref={input_ref} placeholder="Type a message..." style="padding: 8px; flex: 1;" onkeypress={move |e: KeyboardEvent| {
                    if e.key() == "Enter" {
                        // In a real app we'd trigger on_send here
                    }
                }} />
                <button onclick={on_send} style="padding: 8px 16px; background: #2ecc71; color: white; border: none; cursor: pointer;">{"Send"}</button>
            </div>
            
            <p style="font-size: 0.8em; color: #7f8c8d; margin-top: 20px;">
                {"Status: "}{ if ws_state.is_some() { "Connected 🟢" } else { "Disconnected 🔴" } }
            </p>
        </div>
    }
}
