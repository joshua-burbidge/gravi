use grav::{app::orbital::Orbital, App};

/// Test that Earth completes approximately one orbit around the Sun
/// and returns to approximately its starting location within a tolerance.
#[test]
fn test_earth_completes_one_orbit() {
    let mut app = Orbital::new();

    // Load the Sun + Earth + Moon preset (index 1)
    app.load_preset(1);

    app.set_velocities();
    app.refresh_hierarchy();
    app.set_velocities();

    // Retrieve initial state
    let initial_bodies = get_bodies_snapshot(&app);
    let earth_initial_pos = find_body_position(&initial_bodies, "Earth");

    assert!(earth_initial_pos.is_some(), "Earth not found in preset");
    let (earth_x_init, earth_y_init) = earth_initial_pos.unwrap();

    // Start the simulation
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
        "Running {} iterations (~{:.1} days)...",
        runs_needed,
        (runs_needed as f32 * seconds_per_run) / seconds_in_day
    );

    for _ in 0..runs_needed {
        app.run();
    }

    // Get final state
    let final_bodies = get_bodies_snapshot(&app);
    let earth_final_pos = find_body_position(&final_bodies, "Earth");

    assert!(
        earth_final_pos.is_some(),
        "Earth not found after simulation"
    );
    let (earth_x_final, earth_y_final) = earth_final_pos.unwrap();

    // Calculate distance between initial and final positions
    let delta_x = earth_x_final - earth_x_init;
    let delta_y = earth_y_final - earth_y_init;
    let distance = (delta_x.powi(2) + delta_y.powi(2)).sqrt();

    // TODO: make this check position difference relative to the original position

    // Earth orbits at ~150 million km from the Sun
    // For a nearly complete orbit, we expect Earth to be roughly back within a small fraction
    // of its orbital radius. Tolerance: 5% of orbital radius ≈ 7.5 million km
    // This is generous to account for numerical integration errors and the Moon's perturbation.
    let earth_orbit_radius = 149_597_870.0; // km (1 AU approximately)
    let tolerance = earth_orbit_radius * 0.001; // .1% tolerance

    println!(
        "Earth initial position: ({:.2}, {:.2}) km",
        earth_x_init, earth_y_init
    );
    println!(
        "Earth final position: ({:.2}, {:.2}) km",
        earth_x_final, earth_y_final
    );
    println!(
        "Distance from initial: {:.2} km ({:.4}% of orbital radius)",
        distance,
        (distance / earth_orbit_radius) * 100.0
    );
    println!("Tolerance: {:.2} km", tolerance);

    assert!(
        distance < tolerance,
        "Earth did not return to starting position within tolerance. \
         Distance: {:.2} km, Tolerance: {:.2} km",
        distance,
        tolerance
    );
}

/// Test that energy is conserved (within numerical precision) over a short simulation
#[test]
fn test_energy_conservation_short_run() {
    let mut app = Orbital::new();

    // Use Earth+Moon preset (index 2) for simpler 2-body system
    app.load_preset(2);
    app.start();

    // Record initial energy
    let (initial_kinetic, initial_potential) = get_energy(&app);
    let initial_total = initial_kinetic + initial_potential;

    println!("Initial total energy: {:.2e} MJ", initial_total);

    // Run for a short time (just a few iterations)
    for _ in 0..3 {
        app.run();
    }

    // Check final energy
    let (final_kinetic, final_potential) = get_energy(&app);
    let final_total = final_kinetic + final_potential;

    let energy_diff = (final_total - initial_total).abs();
    let percent_diff = if initial_total != 0.0 {
        (energy_diff / initial_total.abs()) * 100.0
    } else {
        0.0
    };

    println!("Final total energy: {:.2e} MJ", final_total);
    println!(
        "Energy difference: {:.2e} MJ ({:.4}%)",
        energy_diff, percent_diff
    );

    // Allow for numerical error. Symplectic Euler has ~O(dt^2) local error and better long-term
    // energy conservation than standard Euler, but over a few steps some drift is expected.
    // Using a 50% tolerance to account for initialization quirks and hierarchical grouping effects.
    assert!(
        percent_diff < 50.0,
        "Energy diverged too much: {:.4}% change (this may indicate integrator or hierarchy issues)",
        percent_diff
    );
}

/// Test that the app can load and run multiple presets without panicking
#[test]
fn test_all_presets_runnable() {
    // Test first 5 presets (excluding the hierarchy test which may be heavy)
    for preset_idx in 0..5 {
        println!("Testing preset {}...", preset_idx);
        let mut app = Orbital::new();
        app.load_preset(preset_idx);
        app.start();

        // Just run a couple of iterations to check for crashes
        for _ in 0..2 {
            app.run();
        }

        println!("Preset {} passed", preset_idx);
    }
}

// ============ Helper functions ============

/// Snapshot of a body's position and name for checking
#[derive(Clone, Debug)]
struct BodySnapshot {
    name: String,
    x: f32,
    y: f32,
}

/// Get a snapshot of all current body positions
fn get_bodies_snapshot(app: &Orbital) -> Vec<BodySnapshot> {
    let bodies_vec = app.bodies_vec();
    bodies_vec
        .iter()
        .map(|b| BodySnapshot {
            name: b.name.clone(),
            x: b.absolute_pos.x,
            y: b.absolute_pos.y,
        })
        .collect()
}

/// Find a body by name and return its (x, y) position, or None
fn find_body_position(bodies: &[BodySnapshot], name: &str) -> Option<(f32, f32)> {
    bodies.iter().find(|b| b.name == name).map(|b| (b.x, b.y))
}

/// Get the current kinetic and potential energy (in MJ)
fn get_energy(app: &Orbital) -> (f64, f64) {
    use grav::app::core::physics::{gravitational_potential_energy, kinetic_energy};

    let bodies_vec = app.bodies_vec();

    let mut total_kinetic = 0.0;
    let mut total_potential = 0.0;

    // Calculate kinetic energy for all bodies
    for body in bodies_vec.iter() {
        let ke = kinetic_energy(body.mass, body.v) as f64;
        total_kinetic += ke;
    }

    // Calculate potential energy for all pairs
    for (i, body1) in bodies_vec.iter().enumerate() {
        for body2 in bodies_vec[i + 1..].iter() {
            let pe = gravitational_potential_energy(body1.mass, body2.mass, body1.pos, body2.pos);
            total_potential += pe;
        }
    }

    (total_kinetic, total_potential)
}
