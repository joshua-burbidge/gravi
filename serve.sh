rustup target add wasm32-unknown-unknown
cargo build --target=wasm32-unknown-unknown
cargo install wasm-bindgen-cli
wasm-bindgen ./target/wasm32-unknown-unknown/debug/grav.wasm --out-dir web/generated --target web
cd web
python3 -m http.server
