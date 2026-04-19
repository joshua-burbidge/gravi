use grav::{
    app::{
        core::physics::Position,
        orbital::{body::Body, Orbital},
    },
    App,
};

/// Test that Earth completes approximately one orbit around the Sun
/// and returns to approximately its starting location within a tolerance.
#[test]
fn test_earth_completes_one_orbit() {
    let mut app = load_preset(1);

    let initial_bodies = get_bodies_snapshot(&app);
    let earth_initial_pos = find_body_position(initial_bodies, "Earth");

    app.start();

    // Make the seconds per run equal to one quarter of a day,
    // so that it will line up with 365.25 days.
    // TODO: run until 365.25 days reached instead.
    let seconds_in_day = 24.0 * 60.0 * 60.0;
    let seconds_per_run = seconds_in_day / 4.0;
    let dt = 50.0;
    let ticks_per_run = seconds_per_run / dt;

    app.dt = dt;
    app.num_ticks = ticks_per_run as i32;

    let seconds_in_year = 365.25_f32 * seconds_in_day;
    let runs_needed = ((seconds_in_year / seconds_per_run).ceil()) as usize;

    println!(
        "Running {} iterations ({:.2} days)...",
        runs_needed,
        (runs_needed as f32 * seconds_per_run) / seconds_in_day
    );

    for _ in 0..runs_needed {
        app.run();
    }

    let final_bodies = get_bodies_snapshot(&app);
    let earth_final_pos = find_body_position(final_bodies, "Earth");

    let earth_position_delta = earth_final_pos.abs_diff(earth_initial_pos);

    let earth_orbit_radius = earth_initial_pos.mag();
    let tolerance = earth_orbit_radius * 0.001;

    println!("Earth initial position: {:?}", earth_initial_pos);
    println!("Earth final position: {:?}", earth_final_pos);
    println!(
        "Distance from initial: {:.2} km ({:.4}% of orbital radius)",
        earth_position_delta,
        (earth_position_delta / earth_orbit_radius) * 100.0
    );
    println!("Tolerance: {:.2} km", tolerance);

    assert!(
        earth_position_delta < tolerance,
        "Earth did not return to starting position within tolerance. \
         Distance: {:.2} km, Tolerance: {:.2} km",
        earth_position_delta,
        tolerance
    );
}

/// Test that energy is conserved before and after
#[test]
fn test_energy_conservation() {
    // Use earth+moon preset
    let mut app = load_preset(2);
    app.start();

    app.analyze();
    let initial_e = app.analysis.initial_e;

    println!("Initial total energy: {:.4e} MJ", initial_e);

    for _ in 0..20 {
        app.run();
    }

    app.analyze();
    let final_e = app.analysis.total_e;

    let energy_diff_2 = (final_e - initial_e).abs();
    let percent_diff_2 = if initial_e != 0.0 {
        (energy_diff_2 / initial_e.abs()) * 100.0
    } else {
        0.0
    };

    println!("Final total energy: {:.4e} MJ", final_e);
    println!(
        "Energy difference: {:.4e} MJ ({:.6}%)",
        energy_diff_2, percent_diff_2
    );

    assert!(
        percent_diff_2 < 0.1, // .1% threshold
        "Energy diverged too much: {:.4}% change",
        percent_diff_2
    );
}

/// Test that the app can load and run presets
#[test]
fn test_all_presets_runnable() {
    for preset_idx in 0..5 {
        println!("Testing preset {}...", preset_idx);
        let mut app = load_preset(preset_idx);

        // run a couple iterations as a smoke test
        for _ in 0..2 {
            app.run();
        }

        println!("Preset {} passed", preset_idx);
    }
}

fn load_preset(preset_idx: usize) -> Orbital {
    let mut app = Orbital::new();

    app.load_preset(preset_idx);

    app.set_velocities();
    app.refresh_hierarchy();
    app.set_velocities();

    app
}

fn get_bodies_snapshot(app: &Orbital) -> Vec<&Body> {
    let bodies_vec = app.bodies_vec();

    bodies_vec
}

fn find_body_position(bodies: Vec<&Body>, name: &str) -> Position {
    let body_opt = bodies.iter().find(|b| b.name == name);

    assert!(body_opt.is_some(), "Body {} not found", name);

    body_opt.unwrap().absolute_pos
}
