use stylist::yew::styled_component;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct MessageInputProps {
    pub current_group: Option<String>,
    pub current_user: String,
    pub on_send_message: Callback<(String, String)>,
    pub disabled: bool,
}

#[styled_component(MessageInput)]
pub fn message_input(props: &MessageInputProps) -> Html {
    let message_text = use_state(|| String::new());
    let is_typing = use_state(|| false);

    let on_input_change = {
        let message_text = message_text.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let value = input.value();
            message_text.set(value);
        })
    };

    let on_submit = {
        let message_text = message_text.clone();
        let current_group = props.current_group.clone();
        let on_send_message = props.on_send_message.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let message = (*message_text).clone().trim().to_string();

            if !message.is_empty() {
                if let Some(group_name) = &current_group {
                    on_send_message.emit((group_name.clone(), message));
                    message_text.set(String::new());
                }
            }
        })
    };

    let on_keydown = {
        let on_submit = on_submit.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" && !e.shift_key() {
                e.prevent_default();
                on_submit.emit(e.into());
            }
        })
    };

    let is_disabled = props.disabled || props.current_group.is_none();
    let placeholder = match props.current_group {
        Some(ref group) => format!("Message #{}", group),
        None => "Select a group to start messaging...".to_string(),
    };

    let message_input_style = stylist::Style::new(
        r#"
        .message-input-container {
            background-color: var(--color-bg-secondary);
            border-top: 1px solid var(--color-border-primary);
            padding: var(--spacing-md);
        }
        
        .input-wrapper {
            display: flex;
            align-items: flex-end;
            gap: var(--spacing-sm);
            max-width: 100%;
        }
        
        .message-textarea {
            flex: 1;
            background-color: var(--color-bg-input);
            border: 1px solid var(--color-border-input);
            border-radius: var(--border-radius-md);
            padding: var(--spacing-sm) var(--spacing-md);
            color: var(--color-text-primary);
            font-size: var(--font-size-md);
            font-family: inherit;
            resize: none;
            min-height: 44px;
            max-height: 120px;
            line-height: 1.4;
            transition: var(--transition-fast);
            overflow-y: auto;
        }
        
        .message-textarea:focus {
            outline: none;
            border-color: var(--color-primary);
            box-shadow: 0 0 0 2px rgba(88, 101, 242, 0.2);
        }
        
        .message-textarea:disabled {
            opacity: 0.5;
            cursor: not-allowed;
        }
        
        .message-textarea::placeholder {
            color: var(--color-text-muted);
        }
        
        .send-button {
            background-color: var(--color-primary);
            color: white;
            border: none;
            border-radius: var(--border-radius-sm);
            padding: var(--spacing-sm) var(--spacing-md);
            font-size: var(--font-size-sm);
            font-weight: var(--font-weight-medium);
            cursor: pointer;
            transition: var(--transition-fast);
            min-width: 60px;
            height: 44px;
            display: flex;
            align-items: center;
            justify-content: center;
            gap: var(--spacing-xs);
        }
        
        .send-button:hover:not(:disabled) {
            background-color: var(--color-primary-hover);
        }
        
        .send-button:disabled {
            background-color: var(--color-text-muted);
            cursor: not-allowed;
            opacity: 0.6;
        }
        
        .send-icon {
            font-size: var(--font-size-md);
        }
        
        .attachment-button {
            background: none;
            border: none;
            color: var(--color-text-secondary);
            cursor: pointer;
            padding: var(--spacing-sm);
            border-radius: var(--border-radius-sm);
            transition: var(--transition-fast);
            height: 44px;
            width: 44px;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        
        .attachment-button:hover:not(:disabled) {
            background-color: var(--color-bg-elevated);
            color: var(--color-text-primary);
        }
        
        .attachment-button:disabled {
            opacity: 0.5;
            cursor: not-allowed;
        }
        
        .attachment-icon {
            font-size: var(--font-size-lg);
        }
        
        .character-count {
            font-size: var(--font-size-xs);
            color: var(--color-text-muted);
            margin-top: var(--spacing-xs);
            text-align: right;
        }
        
        .character-count.warning {
            color: var(--color-warning);
        }
        
        .character-count.error {
            color: var(--color-danger);
        }
        
        .emoji-button {
            background: none;
            border: none;
            color: var(--color-text-secondary);
            cursor: pointer;
            padding: var(--spacing-sm);
            border-radius: var(--border-radius-sm);
            transition: var(--transition-fast);
            height: 44px;
            width: 44px;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        
        .emoji-button:hover:not(:disabled) {
            background-color: var(--color-bg-elevated);
            color: var(--color-text-primary);
        }
        
        .emoji-button:disabled {
            opacity: 0.5;
            cursor: not-allowed;
        }
        
        .emoji-icon {
            font-size: var(--font-size-lg);
        }
    "#,
    )
    .expect("Failed to create message input styles");

    let character_count = (*message_text).len();
    let max_characters = 2000;
    let character_class = if character_count > max_characters * 9 / 10 {
        "character-count error"
    } else if character_count > max_characters * 8 / 10 {
        "character-count warning"
    } else {
        "character-count"
    };

    let can_send =
        !is_disabled && !(*message_text).trim().is_empty() && character_count <= max_characters;

    html! {
        <div class={message_input_style}>
            <div class="message-input-container">
                <form onsubmit={on_submit}>
                    <div class="input-wrapper">
                        <button
                            type="button"
                            class="attachment-button"
                            disabled={is_disabled}
                            title="Attach file (coming soon)"
                        >
                            <span class="attachment-icon">{"📎"}</span>
                        </button>

                        <textarea
                            class="message-textarea"
                            placeholder={placeholder}
                            value={(*message_text).clone()}
                            oninput={on_input_change}
                            onkeydown={on_keydown}
                            disabled={is_disabled}
                            rows="1"
                            autocomplete="off"
                            spellcheck="false"
                        />

                        <button
                            type="button"
                            class="emoji-button"
                            disabled={is_disabled}
                            title="Emoji picker (coming soon)"
                        >
                            <span class="emoji-icon">{"😊"}</span>
                        </button>

                        <button
                            type="submit"
                            class="send-button"
                            disabled={!can_send}
                        >
                            <span class="send-icon">{"➤"}</span>
                            {"Send"}
                        </button>
                    </div>

                    {if character_count > max_characters * 7 / 10 {
                        html! {
                            <div class={character_class}>
                                {format!("{}/{}", character_count, max_characters)}
                            </div>
                        }
                    } else {
                        html! {}
                    }}
                </form>
            </div>
        </div>
    }
}
