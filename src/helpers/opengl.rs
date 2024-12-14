#[cfg(not(target_arch = "wasm32"))]
mod non_wasm_imports {
    pub use glutin::{
        config::ConfigTemplateBuilder,
        context::{ContextApi, ContextAttributesBuilder},
        display::GetGlDisplay,
        prelude::*,
        surface::SurfaceAttributesBuilder,
    };
    pub use glutin_winit::DisplayBuilder;
    #[allow(deprecated)]
    pub use raw_window_handle::HasRawWindowHandle;
    pub use std::num::NonZeroU32;
}
#[cfg(not(target_arch = "wasm32"))]
use non_wasm_imports::*;

use super::{run, WindowSurface};

use femtovg::{renderer::OpenGl, Canvas};
use std::sync::Arc;
use winit::{dpi::PhysicalSize, event_loop::EventLoop, window::WindowAttributes};

pub struct DemoSurface {
    #[cfg(not(target_arch = "wasm32"))]
    context: glutin::context::PossiblyCurrentContext,
    #[cfg(not(target_arch = "wasm32"))]
    surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
}

impl WindowSurface for DemoSurface {
    type Renderer = OpenGl;

    fn resize(&mut self, width: u32, height: u32) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.surface.resize(
                &self.context,
                width.try_into().unwrap(),
                height.try_into().unwrap(),
            );
        }
    }
    fn present(&self, canvas: &mut femtovg::Canvas<Self::Renderer>) {
        canvas.flush_to_surface(&());
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.surface.swap_buffers(&self.context).unwrap();
        }
    }
}

pub async fn start_opengl(
    #[cfg(not(target_arch = "wasm32"))] width: u32,
    #[cfg(not(target_arch = "wasm32"))] height: u32,
    #[cfg(not(target_arch = "wasm32"))] title: &'static str,
    #[cfg(not(target_arch = "wasm32"))] resizeable: bool,
) {
    println!("using openGL...");
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(all(debug_assertions, target_arch = "wasm32"))]
    console_error_panic_hook::set_once();

    let event_loop = EventLoop::new().unwrap();

    #[cfg(not(target_arch = "wasm32"))]
    let (canvas, window, context, surface) = {
        let window_attr = WindowAttributes::default()
            .with_inner_size(PhysicalSize::new(width, height))
            .with_title(title)
            .with_resizable(resizeable);

        let template = ConfigTemplateBuilder::new().with_alpha_size(8);

        let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attr));

        let (window, gl_config) = display_builder
            .build(&event_loop, template, |mut configs| configs.next().unwrap())
            .unwrap();

        let window = window.unwrap();

        let gl_display = gl_config.display();

        #[allow(deprecated)]
        let raw_window_handle = window
            .raw_window_handle()
            .expect("raw window handle failed");

        let context_attributes = ContextAttributesBuilder::new().build(Some(raw_window_handle));
        let fallback_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::Gles(None))
            .build(Some(raw_window_handle));

        let mut not_current_gl_context = Some(unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .unwrap_or_else(|_| {
                    gl_display
                        .create_context(&gl_config, &fallback_context_attributes)
                        .expect("failed to create context")
                })
        });

        let (width, height): (u32, u32) = window.inner_size().into();

        let attrs = SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new().build(
            raw_window_handle,
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        );

        let surface = unsafe {
            gl_config
                .display()
                .create_window_surface(&gl_config, &attrs)
                .unwrap()
        };

        let gl_context = not_current_gl_context
            .take()
            .unwrap()
            .make_current(&surface)
            .unwrap();

        let renderer =
            unsafe { OpenGl::new_from_function_cstr(|s| gl_display.get_proc_address(s).cast()) }
                .expect("Cannot create renderer");

        let mut canvas = Canvas::new(renderer).expect("Cannot create canvas");
        canvas.set_size(width, height, window.scale_factor() as f32);

        (canvas, window, gl_context, surface)
    };

    #[cfg(target_arch = "wasm32")]
    let (canvas, window) = {
        use wasm_bindgen::JsCast;
        use winit::platform::web::WindowAttributesExtWebSys;

        let html_canvas = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();

        let width = html_canvas.width();
        let height = html_canvas.height();

        let renderer = OpenGl::new_from_html_canvas(&html_canvas).expect("Cannot create renderer");

        let window_attrs = WindowAttributes::default().with_canvas(Some(html_canvas));
        let window = event_loop.create_window(window_attrs).unwrap();

        let _ = window.request_inner_size(PhysicalSize::new(width, height));

        let canvas = Canvas::new(renderer).expect("Cannot create canvas");

        (canvas, window)
    };

    let demo_surface = DemoSurface {
        #[cfg(not(target_arch = "wasm32"))]
        context,
        #[cfg(not(target_arch = "wasm32"))]
        surface,
    };

    run(canvas, event_loop, demo_surface, Arc::new(window));
}
