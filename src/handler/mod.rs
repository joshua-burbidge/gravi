use femtovg::{renderer::WGPURenderer, Canvas, Color, Paint, Path};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard,
    window::{Window, WindowId},
};

use crate::{
    app::{
        core::draw::{get_scale, scaled_width},
        App,
    },
    helpers::wgpu::WgpuWindowSurface,
    ui::EguiRenderer,
};

pub struct AppHandler<A: App> {
    mousex: f32,
    mousey: f32,
    dragging: bool,
    close_requested: bool,
    next_tick: bool,
    app: A,
    window: Arc<Window>,
    canvas: Canvas<WGPURenderer>,
    surface: WgpuWindowSurface,
    egui: EguiRenderer,
}
impl<A: App> AppHandler<A> {
    pub fn new(
        canvas: Canvas<WGPURenderer>,
        surface: WgpuWindowSurface,
        window: Arc<Window>,
        egui: EguiRenderer,
        app: A,
    ) -> Self {
        AppHandler {
            canvas,
            surface,
            window,
            egui,
            mousex: 0.,
            mousey: 0.,
            dragging: false,
            close_requested: false,
            next_tick: false,
            app,
        }
    }
}

impl<A: App> ApplicationHandler for AppHandler<A> {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        self.canvas.reset_transform();

        let midpoint_y = (self.canvas.height() / 2) as f32;
        let midpoint_x = (self.app.panel_width() + self.canvas.width() as f32) / 2.;
        self.canvas.translate(midpoint_x, midpoint_y);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let is_consumed = self.egui.handle_input(&self.window, &event);
        if is_consumed {
            return ();
        }

        match event {
            #[cfg(not(target_arch = "wasm32"))]
            WindowEvent::Resized(physical_size) => {
                let surface = &mut self.surface;

                surface.resize(physical_size.width, physical_size.height);
            }
            WindowEvent::CursorMoved {
                device_id: _,
                position,
                ..
            } => {
                let canvas = &mut self.canvas;

                if self.dragging {
                    let p0 = canvas
                        .transform()
                        .inverse()
                        .transform_point(self.mousex, self.mousey);
                    let p1 = canvas
                        .transform()
                        .inverse()
                        .transform_point(position.x as f32, position.y as f32);

                    canvas.translate(p1.0 - p0.0, p1.1 - p0.1);

                    self.window.request_redraw();
                }

                self.mousex = position.x as f32;
                self.mousey = position.y as f32;
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let canvas = &mut self.canvas;

                let y = match delta {
                    MouseScrollDelta::LineDelta(_x_delta, y_delta) => y_delta,
                    MouseScrollDelta::PixelDelta(delta) => (delta.y * 0.01) as f32,
                };

                let scale_factor = 1.0 + (y / 10.0);
                let new_scale = scale_factor * get_scale(canvas);

                #[cfg(target_arch = "wasm32")]
                web_sys::console::log_1(&format!("scale: {}", new_scale).into());

                if new_scale <= 0.001 {
                    return;
                }

                let pt = canvas
                    .transform()
                    .inverse()
                    .transform_point(self.mousex, self.mousey);
                // when the determinant is close to 0, inverse() fails and returns the identity matrix
                // -> when a * d is close to 0 -> when scale is < 0.001

                // translate the canvas to center on the mouse, then scale, then translate back
                canvas.translate(pt.0, pt.1);
                canvas.scale(scale_factor, scale_factor);
                canvas.translate(-pt.0, -pt.1);

                self.window.request_redraw();
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => match state {
                ElementState::Pressed => self.dragging = true,
                ElementState::Released => self.dragging = false,
            },
            WindowEvent::KeyboardInput { event, .. } => {
                let key = event.logical_key;
                let state = event.state;

                match state {
                    ElementState::Pressed => match key {
                        keyboard::Key::Named(named_key) => match named_key {
                            keyboard::NamedKey::Escape => {
                                self.close_requested = true;
                            }
                            keyboard::NamedKey::ArrowRight => {
                                self.next_tick = true;
                                self.window.request_redraw();
                            }
                            _ => {}
                        },
                        keyboard::Key::Character(c) => match c.as_str() {
                            "r" => {
                                println!("pressed r");
                            }
                            _ => {}
                        },
                        _ => {}
                    },
                    ElementState::Released => {}
                }
            }
            WindowEvent::RedrawRequested { .. } => {
                let window = &self.window;
                let surface = &mut self.surface;
                let surface_texture = surface.get_surface_texture();

                // femtovg
                let canvas = &mut self.canvas;

                let size = window.inner_size();
                let dpi_factor = window.scale_factor();
                canvas.set_size(size.width, size.height, dpi_factor as f32);
                canvas.clear_rect(0, 0, size.width, size.height, Color::black());
                draw_base_canvas(canvas);

                if self.next_tick {
                    self.app.run();
                }
                self.app.draw(canvas);
                self.next_tick = false;
                surface.present_canvas(canvas, &surface_texture);

                // egui
                // On the first redraw, the window width/height are sometimes 0.
                // If we pass is 0 to render_ui it will crash.
                let egui_width = if size.width != 0 { size.height } else { 1600 };
                let egui_height = if size.height != 0 { size.height } else { 1000 };
                let device = surface.get_device();
                let queue = surface.get_queue();
                self.egui.render_ui(
                    &mut self.app,
                    window,
                    device,
                    queue,
                    &surface_texture,
                    egui_width,
                    egui_height,
                );

                // both
                surface_texture.present();
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            // _ => {
            //     println!("{:?}", event);
            //     #[cfg(target_arch = "wasm32")]
            //     web_sys::console::log_1(&format!("{:?}", event).into());
            // }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // exiting in wasm just makes it freeze and do nothing
        #[cfg(not(target_arch = "wasm32"))]
        if self.close_requested {
            _event_loop.exit();
        }
    }
}

fn draw_base_canvas(canvas: &mut Canvas<WGPURenderer>) {
    let mut path = Path::new();
    path.move_to(-100000., 0.);
    path.line_to(100000., 0.);

    path.move_to(0., -100000.);
    path.line_to(0., 100000.);

    let width = scaled_width(canvas, 1.);

    canvas.stroke_path(&path, &Paint::color(Color::white()).with_line_width(width));
}
