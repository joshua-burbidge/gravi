#[cfg(not(target_arch = "wasm32"))]
use winit::dpi::PhysicalSize;

use femtovg::{renderer::WGPURenderer, Canvas};
use std::sync::Arc;
use winit::{
    dpi::{PhysicalPosition, Position},
    event_loop::EventLoop,
    window::{Window, WindowAttributes},
};

use crate::{app::App, handler::AppHandler, ui::EguiRenderer};

pub struct WgpuWindowSurface {
    device: Arc<wgpu::Device>,
    surface_config: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface<'static>,
    queue: Arc<wgpu::Queue>,
}

impl WgpuWindowSurface {
    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface_config.width = width.max(1);
        self.surface_config.height = height.max(1);
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn present_canvas(
        &self,
        canvas: &mut Canvas<WGPURenderer>,
        surface_texture: &wgpu::SurfaceTexture,
    ) {
        canvas.flush_to_surface(&surface_texture.texture);
    }

    pub fn get_surface_texture(&self) -> wgpu::SurfaceTexture {
        let surface_texture = self
            .surface
            .get_current_texture()
            .expect(" failed to get current texture");
        surface_texture
    }

    pub fn get_device(&self) -> &Arc<wgpu::Device> {
        &self.device
    }
    pub fn get_queue(&self) -> &Arc<wgpu::Queue> {
        &self.queue
    }
}

pub fn init_wgpu_app<A: App>(
    app: A,
    event_loop: EventLoop<()>,
    canvas: Canvas<WGPURenderer>,
    surface: WgpuWindowSurface,
    window: Arc<Window>,
) {
    let surface_config = &surface.surface_config;
    let device = &surface.device;

    let egui = EguiRenderer::new(&window, device, surface_config.format);

    let mut app_handler = AppHandler::<A>::new(canvas, surface, window, egui, app);

    event_loop
        .run_app(&mut app_handler)
        .expect("failed to run app");
}

pub async fn start_wgpu<A: App>(
    app: A,
    #[cfg(not(target_arch = "wasm32"))] width: u32,
    #[cfg(not(target_arch = "wasm32"))] height: u32,
    #[cfg(not(target_arch = "wasm32"))] title: &'static str,
) {
    println!("using Wgpu...");

    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(all(debug_assertions, target_arch = "wasm32"))]
    console_error_panic_hook::set_once();

    let event_loop = EventLoop::new().unwrap();

    #[cfg(not(target_arch = "wasm32"))]
    let window = {
        let window_attrs = WindowAttributes::default()
            .with_inner_size(PhysicalSize::new(width, height))
            .with_position(PhysicalPosition::new(50, 50))
            .with_title(title);

        #[allow(deprecated)]
        event_loop.create_window(window_attrs).unwrap()
    };

    #[cfg(target_arch = "wasm32")]
    let (window, width, height) = {
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

        let window_attrs = WindowAttributes::default().with_canvas(Some(html_canvas.clone()));
        #[allow(deprecated)]
        let window = event_loop.create_window(window_attrs).unwrap();

        let _ = window.request_inner_size(winit::dpi::PhysicalSize::new(width, height));

        (window, width, height)
    };

    let window = Arc::new(window);

    let backends = wgpu::util::backend_bits_from_env().unwrap_or_default();
    let dx12_shader_compiler = wgpu::util::dx12_shader_compiler_from_env().unwrap_or_default();
    let gles_minor_version = wgpu::util::gles_minor_version_from_env().unwrap_or_default();

    let instance = wgpu::util::new_instance_with_webgpu_detection(wgpu::InstanceDescriptor {
        backends,
        flags: wgpu::InstanceFlags::from_build_config().with_env(),
        dx12_shader_compiler,
        gles_minor_version,
    })
    .await;

    let surface = instance.create_surface(window.clone()).unwrap();

    let adapter = wgpu::util::initialize_adapter_from_env_or_default(&instance, Some(&surface))
        .await
        .expect("Failed to find an appropriate adapter");

    // Create the logical device and command queue
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
            },
            None,
        )
        .await
        .expect("Failed to create device");

    let mut surface_config = surface.get_default_config(&adapter, width, height).unwrap();

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities
        .formats
        .iter()
        .find(|f| !f.is_srgb())
        .copied()
        .unwrap_or_else(|| swapchain_capabilities.formats[0]);
    surface_config.format = swapchain_format;
    surface.configure(&device, &surface_config);

    let device = Arc::new(device);
    let queue = Arc::new(queue);

    let demo_surface = WgpuWindowSurface {
        device: device.clone(),
        surface_config,
        surface,
        queue: queue.clone(),
    };

    let renderer = WGPURenderer::new(device, queue);

    let mut canvas = Canvas::new(renderer).expect("Cannot create canvas");
    canvas.set_size(width, height, window.scale_factor() as f32);

    init_wgpu_app(app, event_loop, canvas, demo_surface, window);
}
