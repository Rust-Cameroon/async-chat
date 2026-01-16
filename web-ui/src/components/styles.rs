use stylist::css;

/// Generate all styles based on dark mode state
pub struct AppStyles {
    pub container: String,
    pub sidebar_left: String,
    pub sidebar_right: String,
    pub chat_main: String,
    pub chat_header: String,
    pub chat_messages: String,
    pub chat_footer: String,
    pub input_wrapper: String,
    pub avatar: String,
    pub search_input: String,
    pub contact_item: String,
    pub contact_item_active: String,
    pub bubble_base: String,
    pub self_bubble: String,
    pub other_bubble: String,
    pub error_bubble: String,
    pub send_btn: String,
    pub icon_btn: String,
    pub connection_pill: String,
    pub action_grid: String,
    pub action_card: String,
}

impl AppStyles {
    pub fn new(
        dark_mode: bool,
        left_visible: bool,
        right_visible: bool,
    ) -> Self {
        // Color scheme based on mode
        let bg_color = if dark_mode { "#1a1a1a" } else { "white" };
        let text_color = if dark_mode { "#e0e0e0" } else { "#1a1a1a" };
        let sidebar_bg = if dark_mode { "#2d2d2d" } else { "#f7f9fa" };
        let border_color = if dark_mode { "#3a3a3a" } else { "#e1e4e8" };
        let input_bg = if dark_mode { "#2d2d2d" } else { "white" };
        let hover_bg = if dark_mode { "#3a3a3a" } else { "#edf2f7" };
        let footer_bg = if dark_mode { "#2d2d2d" } else { "#e3f2fd" };
        
        let left_w = if left_visible { "300px" } else { "0" };
        let right_w = if right_visible { "350px" } else { "0px" };

        Self {
            container: css!(r#"
                display: grid;
                grid-template-columns: ${left} 1fr ${right};
                height: 100vh;
                width: 100vw;
                font-family: 'Inter', sans-serif;
                background-color: ${bg};
                color: ${text};
                overflow: hidden;
                transition: all 0.3s ease;
                position: relative;

                @media (max-width: 1200px) {
                    grid-template-columns: ${left} 1fr 0px;
                }
                @media (max-width: 800px) {
                    grid-template-columns: 0px 1fr 0px;
                }
                @media (max-width: 600px) {
                    font-size: 14px;
                }
            "#, left=left_w, right=right_w, bg=bg_color, text=text_color).to_string(),

            sidebar_left: css!(r#"
                background-color: ${sidebar_bg};
                border-right: 1px solid ${border};
                display: flex;
                flex-direction: column;
                padding: 20px 0;
                overflow: hidden;
                transition: all 0.3s ease;
                min-width: 0;
            "#, sidebar_bg=sidebar_bg, border=border_color).to_string(),

            sidebar_right: css!(r#"
                background-color: ${sidebar_bg};
                border-left: 1px solid ${border};
                display: flex;
                flex-direction: column;
                padding: 20px;
                overflow-y: auto;
                transition: all 0.3s ease;
                @media (max-width: 1200px) { display: none; }
            "#, sidebar_bg=sidebar_bg, border=border_color).to_string(),

            chat_main: css!(r#"
                display: flex;
                flex-direction: column;
                background-color: ${bg};
                overflow: hidden;
                transition: all 0.3s ease;
            "#, bg=bg_color).to_string(),

            chat_header: css!(r#"
                display: flex;
                justify-content: space-between;
                align-items: center;
                padding: 15px 25px;
                border-bottom: 1px solid ${border};
                transition: all 0.3s ease;
                
                @media (max-width: 600px) {
                    padding: 12px 15px;
                }
            "#, border=border_color).to_string(),

            chat_messages: css!(r#"
                flex: 1;
                overflow-y: auto;
                padding: 20px 30px;
                display: flex;
                flex-direction: column;
                gap: 20px;
                background-color: ${bg};
                transition: all 0.3s ease;
                scroll-behavior: smooth;
                -webkit-overflow-scrolling: touch;
                
                @media (max-width: 600px) {
                    padding: 15px;
                    gap: 15px;
                }
            "#, bg=bg_color).to_string(),

            chat_footer: css!(r#"
                padding: 15px 25px 25px;
                background-color: ${footer_bg};
                transition: all 0.3s ease;
                position: relative;
                
                @media (max-width: 600px) {
                    padding: 10px 15px 15px;
                }
            "#, footer_bg=footer_bg).to_string(),

            input_wrapper: css!(r#"
                background-color: ${input_bg};
                border-radius: 30px;
                display: flex;
                align-items: center;
                padding: 5px 10px 5px 20px;
                box-shadow: 0 2px 5px rgba(0,0,0,0.1);
                gap: 15px;
                transition: all 0.3s ease;
                input {
                    flex: 1;
                    border: none;
                    outline: none;
                    padding: 10px 0;
                    font-size: 0.95rem;
                    background: transparent;
                    color: ${text};
                }
                
                @media (max-width: 600px) {
                    padding: 5px 8px 5px 15px;
                    gap: 10px;
                    input {
                        font-size: 0.9rem;
                        padding: 8px 0;
                    }
                }
            "#, input_bg=input_bg, text=text_color).to_string(),

            avatar: css!(r#"
                width: 50px;
                height: 50px;
                border-radius: 50%;
                object-fit: cover;
            "#).to_string(),

            search_input: css!(r#"
                width: 100%;
                padding: 10px 15px 10px 40px;
                border-radius: 20px;
                border: 1px solid ${border};
                background-color: ${input_bg};
                color: ${text};
                font-size: 0.9rem;
                outline: none;
                transition: all 0.2s ease;
                &:focus { border-color: #3498db; box-shadow: 0 0 0 3px rgba(52, 152, 219, 0.1); }
            "#, border=border_color, input_bg=input_bg, text=text_color).to_string(),

            contact_item: css!(r#"
                display: flex;
                align-items: center;
                padding: 12px 20px;
                gap: 15px;
                cursor: pointer;
                transition: all 0.2s ease;
                border-radius: 8px;
                margin: 0 10px;
                &:hover { 
                    background-color: ${hover}; 
                    transform: translateX(5px);
                }
            "#, hover=hover_bg).to_string(),

            contact_item_active: css!(r#"
                background-color: ${hover};
                border-left: 3px solid #0084ff;
            "#, hover=hover_bg).to_string(),

            bubble_base: css!(r#"
                max-width: 70%;
                padding: 12px 18px;
                border-radius: 20px;
                font-size: 0.95rem;
                line-height: 1.5;
                position: relative;
                animation: slideIn 0.3s ease;
                word-wrap: break-word;
                
                @keyframes slideIn {
                    from {
                        opacity: 0;
                        transform: translateY(10px);
                    }
                    to {
                        opacity: 1;
                        transform: translateY(0);
                    }
                }
                
                @media (max-width: 600px) {
                    max-width: 85%;
                    padding: 10px 14px;
                    font-size: 0.9rem;
                }
            "#).to_string(),

            self_bubble: css!(r#"
                align-self: flex-end;
                background: linear-gradient(135deg, #0084ff 0%, #00c6ff 100%);
                color: white;
                border-bottom-right-radius: 4px;
            "#).to_string(),

            other_bubble: css!(r#"
                align-self: flex-start;
                background-color: ${other_bg};
                color: ${other_text};
                border-bottom-left-radius: 4px;
            "#, other_bg=if dark_mode { "#3a3a3a" } else { "#f1f3f4" }, 
               other_text=if dark_mode { "#e0e0e0" } else { "#1a1a1a" }).to_string(),

            error_bubble: css!(r#"
                background-color: #ffebee;
                color: #c62828;
                border-left: 4px solid #c62828;
            "#).to_string(),

            send_btn: css!(r#"
                width: 45px;
                height: 45px;
                background-color: #0084ff;
                color: white;
                border-radius: 50%;
                display: flex;
                align-items: center;
                justify-content: center;
                border: none;
                cursor: pointer;
                box-shadow: 0 4px 10px rgba(0, 132, 255, 0.3);
                transition: transform 0.2s;
                &:hover { transform: scale(1.05); }
                &:active { transform: scale(0.95); }
                
                @media (max-width: 600px) {
                    width: 40px;
                    height: 40px;
                    font-size: 0.9rem;
                }
            "#).to_string(),

            icon_btn: css!(r#"
                font-size: 1.3rem;
                cursor: pointer;
                opacity: 0.6;
                transition: opacity 0.2s, transform 0.2s;
                &:hover { opacity: 1; transform: scale(1.1); }
            "#).to_string(),

            connection_pill: css!(r#"
                font-size: 0.75rem;
                background: ${pill_bg};
                color: ${pill_text};
                padding: 4px 12px;
                border-radius: 12px;
                display: flex;
                align-items: center;
                gap: 6px;
                transition: all 0.3s ease;
                .dot { 
                    width: 8px; 
                    height: 8px; 
                    border-radius: 50%;
                    animation: pulse 2s infinite;
                }
                .online { background-color: #2ecc71; }
                .offline { background-color: #e74c3c; }
                
                @keyframes pulse {
                    0%, 100% { opacity: 1; }
                    50% { opacity: 0.5; }
                }
            "#, pill_bg=if dark_mode { "#3a3a3a" } else { "#edf2f7" }, 
               pill_text=if dark_mode { "#a0aec0" } else { "#4a5568" }).to_string(),

            action_grid: css!(r#"
                display: grid;
                grid-template-columns: 1fr 1fr;
                gap: 10px;
                margin-top: 20px;
            "#).to_string(),

            action_card: css!(r#"
                background: ${card_bg};
                padding: 15px;
                border-radius: 12px;
                display: flex;
                flex-direction: column;
                align-items: center;
                gap: 8px;
                border: 1px solid ${border};
                cursor: pointer;
                transition: all 0.2s ease;
                &:hover { 
                    box-shadow: 0 4px 12px rgba(0,0,0,0.15);
                    transform: translateY(-2px);
                }
                .icon { font-size: 1.5rem; color: #0084ff; }
                .label { font-size: 0.8rem; font-weight: 500; color: ${text}; }
            "#, card_bg=input_bg, border=border_color, text=text_color).to_string(),
        }
    }
}
