use std::sync::Arc;

use femtovg::{Canvas, Color, Paint, Path};
use winit::{
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::EventLoop,
    window::Window,
};

mod helpers;
use helpers::WindowSurface;

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    helpers::start(1000, 600, "my demo", true);
    #[cfg(target_arch = "wasm32")]
    helpers::start();
}

fn run<W: WindowSurface>(
    mut canvas: Canvas<W::Renderer>,
    el: EventLoop<()>,
    mut surface: W,
    window: Arc<Window>,
) {
    let mut mousex = 0.0;
    let mut mousey = 0.0;
    let mut dragging = false;

    el.run(move |event, event_loop_window_target| {
        event_loop_window_target.set_control_flow(winit::event_loop::ControlFlow::Poll);

        match event {
            Event::LoopExiting => event_loop_window_target.exit(),
            Event::WindowEvent { ref event, .. } => match event {
                #[cfg(not(target_arch = "wasm32"))]
                WindowEvent::Resized(physical_size) => {
                    surface.resize(physical_size.width, physical_size.height);
                }
                WindowEvent::CursorMoved {
                    device_id: _,
                    position,
                    ..
                } => {
                    if dragging {
                        let p0 = canvas.transform().inverse().transform_point(mousex, mousey);
                        let p1 = canvas
                            .transform()
                            .inverse()
                            .transform_point(position.x as f32, position.y as f32);

                        canvas.translate(p1.0 - p0.0, p1.1 - p0.1);
                    }

                    mousex = position.x as f32;
                    mousey = position.y as f32;
                }
                WindowEvent::MouseWheel {
                    device_id: _,
                    delta: winit::event::MouseScrollDelta::LineDelta(_, y),
                    ..
                } => {
                    let pt = canvas.transform().inverse().transform_point(mousex, mousey);
                    canvas.translate(pt.0, pt.1);
                    canvas.scale(1.0 + (y / 10.0), 1.0 + (y / 10.0));
                    canvas.translate(-pt.0, -pt.1);
                }
                WindowEvent::MouseInput {
                    button: MouseButton::Left,
                    state,
                    ..
                } => match state {
                    ElementState::Pressed => dragging = true,
                    ElementState::Released => dragging = false,
                },
                WindowEvent::RedrawRequested { .. } => {
                    let size = window.inner_size();
                    let dpi_factor = window.scale_factor();

                    canvas.set_size(size.width, size.height, dpi_factor as f32);
                    canvas.clear_rect(0, 0, size.width, size.height, Color::black());

                    let mut path = Path::new();
                    path.move_to(0., 0.);
                    path.line_to(100., 100.);
                    canvas.stroke_path(&path, &Paint::color(Color::white()));

                    surface.present(&mut canvas);
                    // this is calling flush_to_surface and swap_buffers
                }
                WindowEvent::CloseRequested => event_loop_window_target.exit(),
                // _ => println!("{:?}", event)
                _ => {}
            },
            Event::AboutToWait => window.request_redraw(),
            _ => (),
        }
    })
    .unwrap()
}
