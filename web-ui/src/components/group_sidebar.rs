use crate::types::ChatGroup;
use stylist::yew::styled_component;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct GroupSidebarProps {
    pub groups: Vec<ChatGroup>,
    pub current_group: Option<String>,
    pub on_group_select: Callback<Option<String>>,
    pub on_join_group: Callback<String>,
}

#[styled_component(GroupSidebar)]
pub fn GroupSidebar(props: &GroupSidebarProps) -> Html {
    let sidebar_style = stylist::Style::new(
        r#"
        .sidebar {
            width: 240px;
            background-color: var(--color-bg-secondary);
            border-right: 1px solid var(--color-border-primary);
            display: flex;
            flex-direction: column;
            height: 100%;
        }
        
        .sidebar-header {
            padding: var(--spacing-md);
            border-bottom: 1px solid var(--color-border-primary);
        }
        
        .sidebar-title {
            font-size: var(--font-size-md);
            font-weight: var(--font-weight-semibold);
            color: var(--color-text-primary);
            margin-bottom: var(--spacing-md);
        }
        
        .join-group-form {
            display: flex;
            gap: var(--spacing-xs);
        }
        
        .join-input {
            flex: 1;
            padding: var(--spacing-xs) var(--spacing-sm);
            border: 1px solid var(--color-border-input);
            border-radius: var(--border-radius-sm);
            background-color: var(--color-bg-input);
            color: var(--color-text-primary);
            font-size: var(--font-size-sm);
        }
        
        .join-input:focus {
            outline: none;
            border-color: var(--color-primary);
        }
        
        .join-button {
            padding: var(--spacing-xs) var(--spacing-sm);
            background-color: var(--color-primary);
            color: white;
            border: none;
            border-radius: var(--border-radius-sm);
            font-size: var(--font-size-sm);
            cursor: pointer;
            transition: var(--transition-fast);
        }
        
        .join-button:hover {
            background-color: var(--color-primary-hover);
        }
        
        .join-button:disabled {
            background-color: var(--color-text-muted);
            cursor: not-allowed;
        }
        
        .groups-list {
            flex: 1;
            overflow-y: auto;
            padding: var(--spacing-sm);
        }
        
        .group-item {
            padding: var(--spacing-sm) var(--spacing-md);
            border-radius: var(--border-radius-sm);
            cursor: pointer;
            transition: var(--transition-fast);
            margin-bottom: var(--spacing-xs);
            display: flex;
            align-items: center;
            justify-content: space-between;
        }
        
        .group-item:hover {
            background-color: var(--color-bg-elevated);
        }
        
        .group-item.active {
            background-color: var(--color-primary);
            color: white;
        }
        
        .group-info {
            display: flex;
            flex-direction: column;
            flex: 1;
            min-width: 0;
        }
        
        .group-name {
            font-size: var(--font-size-sm);
            font-weight: var(--font-weight-medium);
            color: inherit;
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
        }
        
        .group-meta {
            font-size: var(--font-size-xs);
            color: var(--color-text-muted);
            margin-top: 2px;
        }
        
        .group-item.active .group-meta {
            color: rgba(255, 255, 255, 0.7);
        }
        
        .group-badges {
            display: flex;
            align-items: center;
            gap: var(--spacing-xs);
        }
        
        .member-count {
            font-size: var(--font-size-xs);
            color: var(--color-text-muted);
        }
        
        .group-item.active .member-count {
            color: rgba(255, 255, 255, 0.7);
        }
        
        .unread-badge {
            background-color: var(--color-danger);
            color: white;
            font-size: var(--font-size-xs);
            font-weight: var(--font-weight-semibold);
            padding: 2px 6px;
            border-radius: 10px;
            min-width: 18px;
            text-align: center;
        }
        
        .empty-state {
            padding: var(--spacing-lg);
            text-align: center;
            color: var(--color-text-muted);
        }
        
        .empty-icon {
            font-size: var(--font-size-xl);
            margin-bottom: var(--spacing-sm);
            opacity: 0.5;
        }
        
        .empty-text {
            font-size: var(--font-size-sm);
        }
    "#,
    )
    .expect("Failed to create sidebar styles");

    let groups = props.groups.clone();
    let current_group = props.current_group.clone();
    let on_group_select = props.on_group_select.clone();
    let on_join_group = props.on_join_group.clone();

    let on_input_change = Callback::from(move |e: Event| {
        let input: web_sys::HtmlInputElement = e.target_unchecked_into();
        // Handle input change if needed
    });

    let on_submit = Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        let input = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("join-group-input")
            .unwrap()
            .dyn_into::<web_sys::HtmlInputElement>()
            .unwrap();

        let group_name = input.value().trim().to_string();
        if !group_name.is_empty() {
            on_join_group.emit(group_name);
            input.set_value("");
        }
    });

    html! {
        <aside class={sidebar_style}>
            <div class="sidebar">
                <div class="sidebar-header">
                    <div class="sidebar-title">{"Groups"}</div>
                    <form class="join-group-form" onsubmit={on_submit}>
                        <input
                            type="text"
                            id="join-group-input"
                            class="join-input"
                            placeholder="Join or create..."
                            oninput={on_input_change}
                        />
                        <button type="submit" class="join-button">
                            {"+"}
                        </button>
                    </form>
                </div>

                <div class="groups-list">
                    if groups.is_empty() {
                        <div class="empty-state">
                            <div class="empty-icon">{"📭"}</div>
                            <div class="empty-text">{"No groups yet. Join or create one above!"}</div>
                        </div>
                    } else {
                        {for groups.iter().map(|group| {
                            let group_name = group.name.clone();
                            let is_active = current_group.as_ref().map_or(false, |g| g == &group.name);

                            let on_click = {
                                let on_group_select = on_group_select.clone();
                                let group_name = group_name.clone();
                                Callback::from(move |_| {
                                    on_group_select.emit(Some(group_name.clone()));
                                })
                            };

                            html! {
                                <div
                                    key={group.name.clone()}
                                    class={classes!("group-item", if is_active { "active" } else { "" })}
                                    onclick={on_click}
                                >
                                    <div class="group-info">
                                        <div class="group-name">{&group.name}</div>
                                        <div class="group-meta">
                                            {if let Some(last_msg) = &group.last_message {
                                                if last_msg.len() > 30 {
                                                    format!("{}...", &last_msg[..30])
                                                } else {
                                                    last_msg.clone()
                                                }
                                            } else {
                                                "No messages yet".to_string()
                                            }}
                                        </div>
                                    </div>
                                    <div class="group-badges">
                                        <span class="member-count">{format!("👤 {}", group.member_count)}</span>
                                        {if group.unread_count > 0 {
                                            html! {
                                                <span class="unread-badge">{group.unread_count}</span>
                                            }
                                        } else {
                                            html! {}
                                        }}
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()}
                    }
                </div>
            </div>
        </aside>
    }
}
