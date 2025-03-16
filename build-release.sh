rustup target add wasm32-unknown-unknown
cargo build --target=wasm32-unknown-unknown --release
cargo install wasm-bindgen-cli --version 0.2.100
wasm-bindgen ./target/wasm32-unknown-unknown/release/grav.wasm --out-dir web/generated --target web
