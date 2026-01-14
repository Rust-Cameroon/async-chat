use yew::prelude::*;
use gloo_net::websocket::futures::WebSocket;
use gloo_net::websocket::Message;
use futures_util::{StreamExt, SinkExt};
use wasm_bindgen_futures::spawn_local;
use async_chat::{FromClient, FromServer};
use std::sync::Arc;
use futures::channel::mpsc;

#[function_component(App)]
pub fn app() -> Html {
    let messages = use_state(|| Vec::new());
    let input_ref = use_node_ref();
    let group_ref = use_node_ref();
    let connected = use_state(|| false);
    
    // Channel for sending messages to the WebSocket task
    let tx = use_state(|| None::<mpsc::UnboundedSender<FromClient>>);

    let on_join = {
        let group_ref = group_ref.clone();
        let messages = messages.clone();
        let connected = connected.clone();
        let tx = tx.clone();
        
        Callback::from(move |_: MouseEvent| {
            let group_name = group_ref.cast::<web_sys::HtmlInputElement>().unwrap().value();
            if group_name.is_empty() { return; }
            
            if let Some(sender) = &*tx {
                // If already connected, just send join msg
                let _ = sender.unbounded_send(FromClient::Join { group_name: Arc::new(group_name) });
                return;
            }

            let messages = messages.clone();
            let connected = connected.clone();
            let tx = tx.clone();
            
            spawn_local(async move {
                let ws = match WebSocket::open("ws://127.0.0.1:8000") {
                    Ok(ws) => ws,
                    Err(e) => {
                        let mut new_messages = (*messages).clone();
                        new_messages.push(format!("Connection error: {:?}", e));
                        messages.set(new_messages);
                        return;
                    }
                };
                
                connected.set(true);
                
                let (mut sink, mut stream) = ws.split();
                let (sender, mut receiver) = mpsc::unbounded::<FromClient>();
                tx.set(Some(sender));

                // Send initial join
                let join_msg = FromClient::Join { group_name: Arc::new(group_name) };
                sink.send(Message::Text(serde_json::to_string(&join_msg).unwrap())).await.unwrap();

                // Background listener (read)
                let messages_listener = messages.clone();
                spawn_local(async move {
                    while let Some(msg) = stream.next().await {
                        if let Ok(Message::Text(text)) = msg {
                            if let Ok(server_msg) = serde_json::from_str::<FromServer>(&text) {
                                let mut new_messages = (*messages_listener).clone();
                                match server_msg {
                                    FromServer::Message { group_name, message } => {
                                        new_messages.push(format!("[{}]: {}", group_name, message));
                                    }
                                    FromServer::Error(err) => {
                                        new_messages.push(format!("Error: {}", err));
                                    }
                                }
                                messages_listener.set(new_messages);
                            }
                        }
                    }
                });

                // Background sender (write)
                spawn_local(async move {
                    while let Some(msg) = receiver.next().await {
                        let json = serde_json::to_string(&msg).unwrap();
                        let _ = sink.send(Message::Text(json)).await;
                    }
                });
            });
        })
    };

    let on_send = {
        let input_ref = input_ref.clone();
        let group_ref = group_ref.clone();
        let tx = tx.clone();
        Callback::from(move |_: MouseEvent| {
            let message = input_ref.cast::<web_sys::HtmlInputElement>().unwrap().value();
            let group_name = group_ref.cast::<web_sys::HtmlInputElement>().unwrap().value();
            if message.is_empty() || group_name.is_empty() { return; }
            
            if let Some(sender) = &*tx {
                let post_msg = FromClient::Post { 
                    group_name: Arc::new(group_name),
                    message: Arc::new(message)
                };
                let _ = sender.unbounded_send(post_msg);
                input_ref.cast::<web_sys::HtmlInputElement>().unwrap().set_value("");
            }
        })
    };

    html! {
        <div style="padding: 20px; font-family: sans-serif; max-width: 600px; margin: 0 auto;">
            <h1 style="color: #e67e22;">{"🔥 Async Chat"}</h1>
            
            <div style="margin-bottom: 20px; display: flex; gap: 10px;">
                <input ref={group_ref} placeholder="Group Name (e.g. Dogs)" style="padding: 8px; flex: 1;" />
                <button onclick={on_join} style="padding: 8px 16px; background: #3498db; color: white; border: none; cursor: pointer;">
                    { if *connected { "Switch Group" } else { "Join" } }
                </button>
            </div>

            <div style="border: 1px solid #ddd; height: 300px; overflow-y: auto; padding: 10px; margin-bottom: 20px; background: #f9f9f9;">
                { for (*messages).iter().map(|m| html! { <div style="margin-bottom: 5px;">{ m }</div> }) }
            </div>

            <div style="display: flex; gap: 10px;">
                <input ref={input_ref} placeholder="Type a message..." style="padding: 8px; flex: 1;" />
                <button onclick={on_send} style="padding: 8px 16px; background: #2ecc71; color: white; border: none; cursor: pointer;">{"Send"}</button>
            </div>
            
            <p style="font-size: 0.8em; color: #7f8c8d; margin-top: 20px;">
                {"Status: "}{ if *connected { "Connected 🟢" } else { "Disconnected 🔴" } }
            </p>
        </div>
    }
}
