use femtovg::{renderer::WGPURenderer, Canvas};

fn get_scale(canvas: &Canvas<WGPURenderer>) -> f32 {
    let transform_matrix = canvas.transform().0;
    let scale_opt = transform_matrix.get(0);

    match scale_opt {
        Some(scale) => *scale,
        None => 1.,
    }
}

pub fn scaled_width(canvas: &Canvas<WGPURenderer>, width_factor: f32) -> f32 {
    let canvas_scale = get_scale(canvas);

    // balance width when scale is small and large
    let equalized_scale = canvas_scale + 1. / canvas_scale;

    equalized_scale * width_factor
}
