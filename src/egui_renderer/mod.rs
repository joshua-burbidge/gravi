use egui;
use egui_wgpu;
use egui_winit;
use wgpu::{
    CommandEncoderDescriptor, Operations, RenderPassColorAttachment, RenderPassDescriptor,
    TextureViewDescriptor,
};
use winit::{event::WindowEvent, window::Window};

use crate::app::Ui;

pub struct EguiRenderer {
    state: egui_winit::State,
    _context: egui::Context,
    renderer: egui_wgpu::Renderer,
}
impl EguiRenderer {
    pub fn new(
        window: &Window,
        device: &wgpu::Device,
        output_color_format: wgpu::TextureFormat,
    ) -> Self {
        let egui_context = egui::Context::default();
        let viewport_id = egui_context.viewport_id();
        let egui_winit_state =
            egui_winit::State::new(egui_context.clone(), viewport_id, window, None, None, None);

        let egui_renderer = egui_wgpu::Renderer::new(device, output_color_format, None, 1, false);

        Self {
            state: egui_winit_state,
            _context: egui_context,
            renderer: egui_renderer,
        }
    }

    pub fn render_ui(
        &mut self,
        ui: &mut Ui,
        window: &Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_texture: &wgpu::SurfaceTexture,
    ) {
        let state = &mut self.state;

        let raw_input = state.take_egui_input(&window);
        let egui_context = state.egui_ctx();

        let full_output = egui_context.run(raw_input, |ctx| {
            ui.ui(ctx);
        });

        let platform_output = full_output.platform_output;
        let clipped_primitives =
            egui_context.tessellate(full_output.shapes, full_output.pixels_per_point);

        state.handle_platform_output(&window, platform_output);

        let egui_renderer = &mut self.renderer;

        for (id, image_delta) in &full_output.textures_delta.set {
            egui_renderer.update_texture(device, &queue, *id, &image_delta);
        }

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("My render encoder"),
        });
        let size = window.inner_size();
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            pixels_per_point: full_output.pixels_per_point,
            size_in_pixels: [size.width, size.height],
        };
        egui_renderer.update_buffers(
            device,
            queue,
            &mut encoder,
            &clipped_primitives,
            &screen_descriptor,
        );

        let texture_view = surface_texture.texture.create_view(&TextureViewDescriptor {
            label: None,
            format: None,
            dimension: None,
            aspect: wgpu::TextureAspect::All, // this is default
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });
        {
            // wgpu example uses a block like this - maybe it's an alternative to dropping render_pass
            let render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("My render pass"),
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
            });

            let mut static_render_pass = render_pass.forget_lifetime();

            egui_renderer.render(
                &mut static_render_pass,
                &clipped_primitives,
                &screen_descriptor,
            );
        }
        for x in &full_output.textures_delta.free {
            egui_renderer.free_texture(x)
        }

        queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn handle_input(&mut self, window: &Window, event: &WindowEvent) -> bool {
        let egui_winit_state = &mut self.state;
        let event_response = egui_winit_state.on_window_event(window, &event);

        // println!("{:?}", event);
        // println!("{:?}", event_response);

        if event_response.repaint {
            window.request_redraw();
        }

        event_response.consumed
    }
}
