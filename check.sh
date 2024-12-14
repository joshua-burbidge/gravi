export RUSTFLAGS="-Awarnings"
cargo check
cargo check --features wgpu
cargo check --target wasm32-unknown-unknown
cargo check --target wasm32-unknown-unknown --features wgpu
