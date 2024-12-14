cargo build --target=wasm32-unknown-unknown
cargo install wasm-bindgen-cli
wasm-bindgen ./target/wasm32-unknown-unknown/debug/femtovg-wgpu.wasm --out-dir wasm/generated --target web
cd wasm
python3 -m http.server
