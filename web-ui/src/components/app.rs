use yew::prelude::*;
use gloo_net::websocket::futures::WebSocket;
use gloo_net::websocket::Message;
use futures_util::{StreamExt, SinkExt};
use wasm_bindgen_futures::spawn_local;
use async_chat::{FromClient, FromServer};
use std::sync::Arc;
use futures::channel::mpsc;
use web_sys::HtmlInputElement;

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
            let group_name = group_ref.cast::<HtmlInputElement>().expect("input exists").value();
            if group_name.is_empty() { return; }
            
            if let Some(sender) = &*tx {
                let _ = sender.unbounded_send(FromClient::Join { group_name: Arc::new(group_name) });
                return;
            }

            let messages = messages.clone();
            let connected = connected.clone();
            let tx_handle = tx.clone();
            
            spawn_local(async move {
                let ws = match WebSocket::open("ws://127.0.0.1:8000") {
                    Ok(ws) => {
                        web_sys::console::log_1(&"WebSocket connection established".into());
                        ws
                    },
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
                tx_handle.set(Some(sender));

                // Send initial join
                let join_msg = FromClient::Join { group_name: Arc::new(group_name) };
                if let Err(e) = sink.send(Message::Text(serde_json::to_string(&join_msg).unwrap())).await {
                    web_sys::console::error_1(&format!("Failed to send join: {:?}", e).into());
                }

                // Background listener (read)
                let messages_listener = messages.clone();
                let connected_listener = connected.clone();
                let tx_listener = tx_handle.clone();
                spawn_local(async move {
                    while let Some(msg_result) = stream.next().await {
                        match msg_result {
                            Ok(Message::Text(text)) => {
                                match serde_json::from_str::<FromServer>(&text) {
                                    Ok(server_msg) => {
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
                                    Err(e) => {
                                        web_sys::console::error_1(&format!("Failed to parse server message: {:?}. Raw: {}", e, text).into());
                                    }
                                }
                            }
                            Ok(_) => (),
                            Err(e) => {
                                web_sys::console::error_1(&format!("WebSocket stream error: {:?}", e).into());
                                break;
                            }
                        }
                    }
                    connected_listener.set(false);
                    tx_listener.set(None);
                    let mut new_messages = (*messages_listener).clone();
                    new_messages.push("Connection lost or closed.".to_string());
                    messages_listener.set(new_messages);
                });

                // Background sender (write)
                spawn_local(async move {
                    while let Some(msg) = receiver.next().await {
                        let json = serde_json::to_string(&msg).unwrap();
                        if let Err(e) = sink.send(Message::Text(json)).await {
                            web_sys::console::error_1(&format!("WebSocket send error: {:?}", e).into());
                            break;
                        }
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
            let input_el = input_ref.cast::<HtmlInputElement>().expect("input exists");
            let group_el = group_ref.cast::<HtmlInputElement>().expect("group exists");
            let message = input_el.value();
            let group_name = group_el.value();
            
            if message.is_empty() || group_name.is_empty() { return; }
            
            if let Some(sender) = &*tx {
                let post_msg = FromClient::Post { 
                    group_name: Arc::new(group_name),
                    message: Arc::new(message)
                };
                if let Err(e) = sender.unbounded_send(post_msg) {
                    web_sys::console::error_1(&format!("Failed to queue message: {:?}", e).into());
                } else {
                    input_el.set_value("");
                }
            } else {
                web_sys::console::warn_1(&"Not connected. Cannot send message.".into());
            }
        })
    };

    let on_keypress = {
        let on_send = on_send.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                on_send.emit(MouseEvent::new("click").unwrap());
            }
        })
    };

    html! {
        <div style="padding: 20px; font-family: sans-serif; max-width: 600px; margin: 0 auto;">
            <h1 style="color: #e67e22;">{"🔥 Async Chat"}</h1>
            
            <div style="margin-bottom: 20px; display: flex; gap: 10px;">
                <input ref={group_ref} placeholder="Group Name (e.g. Dogs)" style="padding: 8px; flex: 1;" />
                <button onclick={on_join} style="padding: 8px 16px; border-radius: 4px; background: #3498db; color: white; border: none; cursor: pointer;">
                    { if *connected { "Switch Group" } else { "Join" } }
                </button>
            </div>

            <div style="border: 1px solid #ddd; border-radius: 4px; height: 300px; overflow-y: auto; padding: 10px; margin-bottom: 20px; background: #f9f9f9;">
                { for (*messages).iter().map(|m| html! { <div style="margin-bottom: 5px;">{ m }</div> }) }
            </div>

            <div style="display: flex; gap: 10px;">
                <input ref={input_ref} onkeypress={on_keypress} placeholder="Type a message..." style="padding: 8px; flex: 1;" />
                <button onclick={on_send} style="padding: 8px 16px; border-radius: 4px; background: #2ecc71; color: white; border: none; cursor: pointer;">{"Send"}</button>
            </div>
            
            <p style="font-size: 0.8em; color: #7f8c8d; margin-top: 20px;">
                {"Status: "}{ if *connected { "Connected 🟢" } else { "Disconnected 🔴" } }
            </p>
        </div>
    }
}
