use yew::prelude::*;
use gloo_net::websocket::futures::WebSocket;
use gloo_net::websocket::Message;
use futures_util::{StreamExt, SinkExt};
use wasm_bindgen_futures::spawn_local;
use async_chat::{FromClient, FromServer};
use std::sync::Arc;
use futures::channel::mpsc;
use web_sys::HtmlInputElement;
use stylist::yew::styled_component;

#[derive(Clone, PartialEq)]
struct ChatMessage {
    author: String,
    text: String,
    is_self: bool,
    is_error: bool,
}

enum ChatAction {
    AddMessage(ChatMessage),
    Clear,
}

struct ChatState {
    messages: Vec<ChatMessage>,
}

impl Reducible for ChatState {
    type Action = ChatAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut messages = self.messages.clone();
        match action {
            ChatAction::AddMessage(msg) => {
                messages.push(msg);
            }
            ChatAction::Clear => {
                messages.clear();
            }
        }
        Self { messages }.into()
    }
}

use std::rc::Rc;

#[styled_component(App)]
pub fn app() -> Html {
    let chat_state = use_reducer(|| ChatState { messages: Vec::new() });
    let input_ref = use_node_ref();
    let group_ref = use_node_ref();
    let name_ref = use_node_ref();
    let chat_box_ref = use_node_ref();
    let connected = use_state(|| false);
    
    let tx = use_state(|| None::<mpsc::UnboundedSender<FromClient>>);

    // Auto-scroll effect
    {
        let chat_box_ref = chat_box_ref.clone();
        let messages_len = chat_state.messages.len();
        use_effect_with(messages_len, move |_| {
            if let Some(div) = chat_box_ref.cast::<web_sys::HtmlElement>() {
                div.set_scroll_top(div.scroll_height());
            }
            || ()
        });
    }

    let on_join = {
        let group_ref = group_ref.clone();
        let chat_state = chat_state.clone();
        let connected = connected.clone();
        let tx = tx.clone();
        
        Callback::from(move |_: MouseEvent| {
            let group_name = group_ref.cast::<HtmlInputElement>().expect("input exists").value();
            if group_name.is_empty() { return; }
            
            if let Some(sender) = &*tx {
                let _ = sender.unbounded_send(FromClient::Join { group_name: Arc::new(group_name) });
                return;
            }

            let chat_state = chat_state.clone();
            let connected = connected.clone();
            let tx_handle = tx.clone();
            
            spawn_local(async move {
                let ws = match WebSocket::open("ws://127.0.0.1:8000") {
                    Ok(ws) => ws,
                    Err(e) => {
                        chat_state.dispatch(ChatAction::AddMessage(ChatMessage {
                            author: "System".to_string(),
                            text: format!("Connection error: {:?}", e),
                            is_self: false,
                            is_error: true,
                        }));
                        return;
                    }
                };
                
                connected.set(true);
                
                let (mut sink, mut stream) = ws.split();
                let (sender, mut receiver) = mpsc::unbounded::<FromClient>();
                tx_handle.set(Some(sender));

                let join_msg = FromClient::Join { group_name: Arc::new(group_name) };
                let _ = sink.send(Message::Text(serde_json::to_string(&join_msg).unwrap())).await;

                let chat_state_listener = chat_state.clone();
                let connected_listener = connected.clone();
                let tx_listener = tx_handle.clone();
                spawn_local(async move {
                    while let Some(msg_result) = stream.next().await {
                        match msg_result {
                            Ok(Message::Text(text)) => {
                                if let Ok(server_msg) = serde_json::from_str::<FromServer>(&text) {
                                    match server_msg {
                                        FromServer::Message { group_name, message } => {
                                            chat_state_listener.dispatch(ChatAction::AddMessage(ChatMessage {
                                                author: group_name.to_string(),
                                                text: message.to_string(),
                                                is_self: false, // In a real app we'd compare with our own name
                                                is_error: false,
                                            }));
                                        }
                                        FromServer::Error(err) => {
                                            chat_state_listener.dispatch(ChatAction::AddMessage(ChatMessage {
                                                author: "Error".to_string(),
                                                text: err,
                                                is_self: false,
                                                is_error: true,
                                            }));
                                        }
                                    }
                                }
                            }
                            Ok(_) => (),
                            Err(_) => break,
                        }
                    }
                    connected_listener.set(false);
                    tx_listener.set(None);
                    chat_state_listener.dispatch(ChatAction::AddMessage(ChatMessage {
                        author: "System".to_string(),
                        text: "Connection lost.".to_string(),
                        is_self: false,
                        is_error: true,
                    }));
                });

                spawn_local(async move {
                    while let Some(msg) = receiver.next().await {
                        let json = serde_json::to_string(&msg).unwrap();
                        if let Err(_) = sink.send(Message::Text(json)).await {
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
        let name_ref = name_ref.clone();
        let tx = tx.clone();
        Callback::from(move |_: MouseEvent| {
            let input_el = input_ref.cast::<HtmlInputElement>().expect("input exists");
            let group_el = group_ref.cast::<HtmlInputElement>().expect("group exists");
            let name_el = name_ref.cast::<HtmlInputElement>().expect("name exists");
            
            let message = input_el.value();
            let group_name = group_el.value();
            let user_name = name_el.value();
            
            if message.is_empty() || group_name.is_empty() { return; }
            
            if let Some(sender) = &*tx {
                let msg_text = if user_name.is_empty() { 
                    message 
                } else { 
                    format!("{}: {}", user_name, message) 
                };

                let post_msg = FromClient::Post { 
                    group_name: Arc::new(group_name),
                    message: Arc::new(msg_text)
                };
                if let Ok(_) = sender.unbounded_send(post_msg) {
                    input_el.set_value("");
                }
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

    let container_style = css!(r#"
        display: flex;
        flex-direction: column;
        height: 100vh;
        max-width: 800px;
        margin: 0 auto;
        font-family: 'Inter', system-ui, -apple-system, sans-serif;
        background-color: #f8f9fa;
        color: #212529;
    "#);

    let header_style = css!(r#"
        padding: 20px;
        background: linear-gradient(135deg, #e67e22, #d35400);
        color: white;
        box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        display: flex;
        justify-content: space-between;
        align-items: center;
    "#);

    let chat_area_style = css!(r#"
        flex: 1;
        overflow-y: auto;
        padding: 20px;
        display: flex;
        flex-direction: column;
        gap: 12px;
    "#);

    let input_area_style = css!(r#"
        padding: 20px;
        background: white;
        border-top: 1px solid #dee2e6;
        display: flex;
        gap: 10px;
        align-items: flex-end;
    "#);

    let controls_style = css!(r#"
        padding: 10px 20px;
        background: #fff;
        border-bottom: 1px solid #dee2e6;
        display: flex;
        gap: 10px;
        flex-wrap: wrap;
    "#);

    let bubble_base = css!(r#"
        max-width: 70%;
        padding: 10px 16px;
        border-radius: 18px;
        font-size: 0.95rem;
        line-height: 1.4;
        position: relative;
        animation: fadeIn 0.3s ease-out;

        @keyframes fadeIn {
            from { opacity: 0; transform: translateY(10px); }
            to { opacity: 1; transform: translateY(0); }
        }
    "#);

    let my_bubble = css!(r#"
        align-self: flex-end;
        background-color: #0084ff;
        color: white;
        border-bottom-right-radius: 4px;
    "#);

    let other_bubble = css!(r#"
        align-self: flex-start;
        background-color: #e4e6eb;
        color: black;
        border-bottom-left-radius: 4px;
    "#);

    let error_bubble = css!(r#"
        align-self: center;
        background-color: #fce4e4;
        color: #c0392b;
        font-size: 0.85rem;
        border: 1px solid #f5c6cb;
    "#);

    let input_style = css!(r#"
        padding: 12px 16px;
        border: 1px solid #ced4da;
        border-radius: 24px;
        outline: none;
        flex: 1;
        transition: border-color 0.2s;
        &:focus { border-color: #e67e22; }
    "#);

    let btn_style = css!(r#"
        padding: 10px 24px;
        border-radius: 24px;
        border: none;
        font-weight: 600;
        cursor: pointer;
        transition: transform 0.1s, filter 0.2s;
        &:active { transform: scale(0.96); }
        &:hover { filter: brightness(1.1); }
    "#);

    let join_btn_style = css!(r#"
        background: #3498db;
        color: white;
    "#);

    let send_btn_style = css!(r#"
        background: #2ecc71;
        color: white;
        font-size: 1.2rem;
        padding: 10px;
    "#);

    html! {
        <div class={container_style}>
            <header class={header_style}>
                <div style="display: flex; align-items: center; gap: 10px;">
                    <span style="font-size: 1.5rem;">{"🔥"}</span>
                    <h1 style="margin: 0; font-size: 1.25rem; font-weight: 800; letter-spacing: -0.5px;">{"Async Chat"}</h1>
                </div>
                <div style="font-size: 0.8rem; background: rgba(255,255,255,0.2); padding: 4px 12px; border-radius: 12px;">
                    { if *connected { "Online" } else { "Offline" } }
                </div>
            </header>

            <div class={controls_style}>
                <input ref={name_ref} class={input_style.clone()} placeholder="Your Name" style="flex: 0 1 150px;" />
                <input ref={group_ref} class={input_style.clone()} placeholder="Group Name" style="flex: 1;" />
                <button onclick={on_join} class={classes!(btn_style.clone(), join_btn_style)}>
                    { if *connected { "Switch" } else { "Join" } }
                </button>
            </div>

            <main ref={chat_box_ref} class={chat_area_style}>
                { for chat_state.messages.iter().map(|m| {
                    let class = if m.is_error {
                        classes!(bubble_base.clone(), error_bubble.clone())
                    } else if m.is_self {
                        classes!(bubble_base.clone(), my_bubble.clone())
                    } else {
                        classes!(bubble_base.clone(), other_bubble.clone())
                    };
                    html! {
                        <div {class}>
                            if !m.is_error {
                                <div style="font-size: 0.7rem; font-weight: 700; margin-bottom: 2px; opacity: 0.8;">{ &m.author }</div>
                            }
                            { &m.text }
                        </div>
                    }
                })}
            </main>

            <footer class={input_area_style}>
                <input ref={input_ref} onkeypress={on_keypress} class={input_style} placeholder="Type a message..." />
                <button onclick={on_send} class={classes!(btn_style.clone(), send_btn_style)}>
                    {"↑"}
                </button>
            </footer>
        </div>
    }
}
