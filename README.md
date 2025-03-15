# gravi

2D orbital mechanics simulations, made with Rust.

Runs natively and compiles to WebAssembly to run in a browser.

## Usage
1. Select a preset in the top left.
  - ![preset-options](image.png)
2. Configure initial conditions further in the left panel.
  - ![more-configuration](image-1.png)
3. Press the Start button to lock in initial conditions and start the simulation.
4. Press the right arrow to progress forwards.
5. Select another preset and start again.

![demo](image-2.png)

#### Building and Running

To run natively: `cargo run`

To run in a browser with WebAssembly: `./serve.sh` then go to localhost:8000

#### Dependencies

Uses [`wgpu`](https://github.com/gfx-rs/wgpu) (graphics) + [`femtovg`](https://github.com/femtovg/femtovg) (2D vector drawing) + [`egui`](https://github.com/femtovg/femtovg) (UI).

#### Credits

Created from my femtovg+wgpu+egui boilerplate here: https://github.com/joshua-burbidge/femtovg-wgpu, which references the [femtovg examples](https://github.com/femtovg/femtovg/tree/master/examples) for femtovg integration and [egui-wgpu-demo](https://github.com/ejb004/egui-wgpu-demo) by [ejb004](https://github.com/ejb004) for the egui-wgpu integration.

#### TODO
- disable/don't render ui after starting to improve performance?
  - don't clear canvas?
- use types for units
- why is the frame time so low even when it's lagging
- Binary system - 2 equally-sized bodies
- performance gets worse when there are long trajectories
- show distance

#### Verify
- new energy implementation