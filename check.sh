export RUSTFLAGS="-Awarnings"
cargo check
cargo check --target wasm32-unknown-unknown
