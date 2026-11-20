use grav::Orbital;

pub fn load_preset(preset_idx: usize) -> Orbital {
    let mut app = Orbital::new();
    app.load_preset(preset_idx);
    app.set_velocities();
    app.refresh_hierarchy();
    app.set_velocities();
    app
}
