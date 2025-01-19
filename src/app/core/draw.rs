use femtovg::{renderer::WGPURenderer, Canvas};

pub fn get_scale(canvas: &Canvas<WGPURenderer>) -> f32 {
    let transform_matrix = canvas.transform().0;
    let scale_opt = transform_matrix.get(0);

    match scale_opt {
        Some(scale) => *scale,
        None => 1.,
    }
}
