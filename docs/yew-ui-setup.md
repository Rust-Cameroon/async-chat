# 🎨 Yew UI Setup Guide

## 🚀 Quick Start for Yew Frontend Development

### Prerequisites
```bash
# Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WebAssembly target
rustup target add wasm32-unknown-unknown

# Install Trunk (build tool for Yew)
cargo install trunk

# Install wasm-bindgen-cli
cargo install wasm-bindgen-cli
```

### Project Structure (Proposed)
```
async-chat/
├── server/                 # Existing server code
├── web-ui/                # New Yew frontend
│   ├── src/
│   │   ├── components/
│   │   │   ├── app.rs
│   │   │   ├── chat_room.rs
│   │   │   ├── message.rs
│   │   │   └── mod.rs
│   │   ├── services/
│   │   │   ├── websocket.rs
│   │   │   └── mod.rs
│   │   ├── styles/
│   │   │   └── main.css
│   │   └── main.rs
│   ├── static/
│   │   └── index.html
│   ├── Cargo.toml
│   └── Trunk.toml
└── README.md
```

### Initial Cargo.toml for web-ui
```toml
[package]
name = "async-chat-web"
version = "0.1.0"
edition = "2021"

[dependencies]
yew = { version = "0.21", features = ["csr"] }
yewdux = "0.10"
gloo = "0.11"
gloo-net = "0.5"
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
web-sys = "0.3"
js-sys = "0.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
log = "0.4"
wasm-logger = "0.2"

[dependencies.web-sys]
version = "0.3"
features = [
  "console",
  "WebSocket",
  "MessageEvent",
  "CloseEvent",
  "ErrorEvent",
]
```

### Development Commands
```bash
# Navigate to web-ui directory
cd web-ui

# Start development server with hot reload
trunk serve

# Build for production
trunk build --release

# Run server (in separate terminal)
cd ../server
cargo run --release --bin server -- localhost:8000
```

### Getting Started Steps

1. **Create the web-ui directory structure**
2. **Set up basic Yew app with WebSocket connection**
3. **Implement message sending/receiving**
4. **Add styling with TailwindCSS**
5. **Create responsive chat interface**

### Useful Resources
- [Yew Examples](https://github.com/yewstack/yew/tree/master/examples)
- [WebSocket Example](https://github.com/yewstack/yew/tree/master/examples/websocket)
- [Trunk Configuration](https://trunkrs.dev/#configuration)
- [TailwindCSS with Trunk](https://trunkrs.dev/assets/#tailwind-css)

---

This will be an exciting project showcasing Rust's full-stack capabilities! 🦀✨
