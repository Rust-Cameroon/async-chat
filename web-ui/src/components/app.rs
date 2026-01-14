use yew::prelude::*;
use gloo_net::websocket::futures::WebSocket;
use gloo_net::websocket::Message;
use futures_util::{StreamExt, SinkExt};
use wasm_bindgen_futures::spawn_local;
use async_chat::{FromClient, FromServer};
use std::sync::Arc;
use std::rc::Rc;

#[function_component(App)]
pub fn app() -> Html {
    let messages = use_state(|| Vec::new());
    let input_ref = use_node_ref();
    let group_ref = use_node_ref();
    // We'll store the WebSocket in a Ref to avoid PartialEq issues with use_state handles
    let ws_ref = use_mut_ref(|| None::<WebSocket>);
    let connected = use_state(|| false);

    // Effect to handle join/connection
    let on_join = {
        let group_ref = group_ref.clone();
        let ws_ref = ws_ref.clone();
        let messages = messages.clone();
        let connected = connected.clone();
        Callback::from(move |_| {
            let group_name = group_ref.cast::<web_sys::HtmlInputElement>().unwrap().value();
            if group_name.is_empty() { return; }
            
            let ws_ref = ws_ref.clone();
            let messages = messages.clone();
            let connected = connected.clone();
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

                // Join the group
                let join_msg = FromClient::Join { group_name: Arc::new(group_name) };
                let join_json = serde_json::to_string(&join_msg).unwrap();
                ws.send(Message::Text(join_json)).await.unwrap();

                // Split the websocket so we can store the write half and listen with the read half
                let (mut write, mut read) = ws.split();
                
                // Store the write half for sending messages
                // Wait, WebSocket doesn't support easy splitting into independent Send/Sync parts easily without wrapping.
                // But gloo-net WebSocket implements Sink and Stream.
                // We'll just keep the whole thing in the ref if we need to send later.
                // Actually, let's use a simpler approach for this demo.
                
                // For now, we'll store the whole thing.
                // To allow background listening while also sending, we need a way to share it.
                // Gloo-net WebSocket is NOT Clone.
                
                // Let's use the split version.
                // But how to store 'write'? It's a SplitSink.
                
                // Actually, let's just use a message queue or a simpler architecture.
            });
        })
    };
    
    // RETHINK: Gloo-net WebSocket (futures) is hard to share.
    // Let's use the callback-based WebSocket if we want easier sharing, 
    // or just manage everything in a single spawn_local loop.
    
    // I will rewrite this to be simpler and actually work.
    html! {
        <div style="padding: 20px; font-family: sans-serif; max-width: 600px; margin: 0 auto;">
            <p>{"WebSocket integration in progress... check logs"}</p>
        </div>
    }
}
