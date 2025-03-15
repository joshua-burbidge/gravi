# gravi

Doing some physics simulation stuff with Rust.

Uses [`wgpu`](https://github.com/gfx-rs/wgpu) (graphics) + [`femtovg`](https://github.com/femtovg/femtovg) (2D vector drawing) + [`egui`](https://github.com/femtovg/femtovg) (UI).

Created from my boilerplate here: https://github.com/joshua-burbidge/femtovg-wgpu

#### WASM

```sh
cargo install wasm-bindgen-cli
cargo build --target=wasm32-unknown-unknown
wasm-bindgen ./target/wasm32-unknown-unknown/debug/examples/demo.wasm --out-dir examples/generated --target web

cd examples/
python3 -m http.server
```

#### TODO
- disable/don't render ui after starting to improve performance?
  - don't clear canvas?
- use types for units
- why is the frame time so low even when it's lagging
- Binary system - 2 equally-sized bodies
- wgpu + WASM and/or OpenGL + WASM

#### Verify
- new energy implementation