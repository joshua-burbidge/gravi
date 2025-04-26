pub mod draw;
pub mod physics;

pub fn midpoint(a: f32, b: f32) -> f32 {
    (a + b) / 2.
}

pub fn eq_tolerance(a: f32, b: f32, tol: f32) -> bool {
    (a - b).abs() < tol
}

pub fn remove_indices<T>(vec: Vec<T>, indices_to_remove: Vec<usize>) -> Vec<T> {
    vec.into_iter()
        .enumerate()
        .filter_map(|(i, item)| {
            if indices_to_remove.contains(&i) {
                None
            } else {
                Some(item)
            }
        })
        .collect()
}
