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
    let connected = use_state(|| false);
    
    // We'll use a reducer or a state to queue messages to be sent
    let outgoing_messages = use_state(|| Vec::<FromClient>::new());

    // Effect to manage the WebSocket lifecycle
    {
        let messages = messages.clone();
        let connected = connected.clone();
        let outgoing_messages = outgoing_messages.clone();
        
        use_effect_with((), move |_| {
            // This effect runs once on mount. 
            // In a real app, we might want to trigger it on "Join" click.
            // Let's make it respond to the first Join.
            || ()
        });
    }

    // Callback to "Join" (triggers the connection if not active)
    let on_join = {
        let group_ref = group_ref.clone();
        let messages = messages.clone();
        let connected = connected.clone();
        let outgoing_messages = outgoing_messages.clone();
        
        Callback::from(move |_: MouseEvent| {
            let group_name = group_ref.cast::<web_sys::HtmlInputElement>().unwrap().value();
            if group_name.is_empty() { return; }
            
            if *connected {
                // Already connected, just send Join
                let mut new_outgoing = (*outgoing_messages).clone();
                new_outgoing.push(FromClient::Join { group_name: Arc::new(group_name) });
                outgoing_messages.set(new_outgoing);
                return;
            }

            let messages = messages.clone();
            let connected = connected.clone();
            let outgoing_messages_handle = outgoing_messages.clone();
            
            spawn_local(async move {
                let mut ws = match WebSocket::open("ws://127.0.0.1:8000") {
                    Ok(ws) => ws,
                    Err(e) => {
                        let mut new_messages = (*messages).clone();
                        new_messages.push(format!("Connection error: {:?}", e));
                        messages.set(new_messages);
                        return;
                    }
                };
                
                connected.set(true);
                
                // Send initial join
                let join_msg = FromClient::Join { group_name: Arc::new(group_name) };
                ws.send(Message::Text(serde_json::to_string(&join_msg).unwrap())).await.unwrap();

                let (mut sink, mut stream) = ws.split();
                
                // Background listener
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

                // Since we can't easily listen to `outgoing_messages` state changes here 
                // without setting up a complicated bridge, we'll use a simple loop 
                // and a channel if we wanted it to be truly reactive to other component parts.
                // But for this demo, we can just handle "Send" by adding to a local Ref.
            });
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
                <button style="padding: 8px 16px; background: #2ecc71; color: white; border: none; cursor: pointer;">{"Send"}</button>
            </div>
            
            <p style="font-size: 0.8em; color: #7f8c8d; margin-top: 20px;">
                {"Status: "}{ if *connected { "Connected 🟢" } else { "Disconnected 🔴" } }
            </p>
        </div>
    }
}
