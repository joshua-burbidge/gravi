pub use vector::{Acceleration, Position, Velocity};

mod vector;

pub const G: f32 = 6.674e-11; // N m^2 / kg^2
pub const G_KM: f32 = G * 1e-6; // N km^2 / kg^2 (converted to km)
pub const R_EARTH_KM: f32 = 6378.;

// calculate the magnitude of the circular velocity
// v = sqrt(GM/r)
fn circular_velocity_magnitude(central_mass: f32, r: f32) -> f32 {
    (G * central_mass / (r * 1e9)).sqrt()
}
// The velocity that results in a perfect circular orbit
// given the central mass and the radius
pub fn circular_velocity(central_mass: f32, r: Position) -> Velocity {
    let r_mag = r.mag();
    let circular_velocity_magnitude = circular_velocity_magnitude(central_mass, r_mag);

    // the y component is based on the x position, and vice versa
    let y_vel = if r.x == 0. {
        0.
    } else {
        // take the magnitude and multiply by the proportion that should go in each dimension
        -circular_velocity_magnitude * r.x / r_mag // negative y makes it go clockwise
    };
    let x_vel = if r.y == 0. {
        0.
    } else {
        circular_velocity_magnitude * r.y / r_mag
    };
    Velocity::new(x_vel, y_vel)
}

// escape velocity = sqrt(2) * circular_velocity
pub fn escape_velocity(central_mass: f32, r: Position) -> Velocity {
    let circular_velocity = circular_velocity(central_mass, r);

    Velocity::new(
        2_f32.sqrt() * circular_velocity.x,
        2_f32.sqrt() * circular_velocity.y,
    )
}

pub fn gravitational_acceleration(
    central_pos: Position,
    orbital_pos: Position,
    central_mass: f32,
) -> Acceleration {
    // a = -G * m_central * r_vec / (|r_vec|^3)

    let r = orbital_pos.minus(central_pos);

    let a_x = -G_KM * central_mass * r.x / r.mag().powi(3); // m/s^2
    let a_x_km = a_x * 1e-3; // km/s^2

    let a_y = -G_KM * central_mass * r.y / r.mag().powi(3); // m/s^2
    let a_y_km = a_y * 1e-3; // km/s^2

    let cur_a = Acceleration::new(a_x_km, a_y_km);

    cur_a
}
