pub use vector::{Acceleration, Position, Velocity};

mod vector;

pub const G: f32 = 6.674e-11; // N m^2 / kg^2
pub const G_KM: f32 = G * 1e-6; // N km^2 / kg^2 (converted to km)
pub const R_EARTH_KM: f32 = 6378.;
pub const R_MOON_KM: f32 = 1740.;

// calculate the magnitude of the circular velocity
// v = sqrt(GM/r)
fn circular_velocity_magnitude(central_mass: f32, r: f32) -> f32 {
    (G * central_mass / (r * 1e9)).sqrt()
}
// The velocity that results in a perfect circular orbit
// given the central mass and the radius
pub fn circular_velocity(
    central_pos: Position,
    central_mass: f32,
    orbital_pos: Position,
) -> Velocity {
    let r = orbital_pos.minus(central_pos);
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
pub fn escape_velocity(
    central_pos: Position,
    central_mass: f32,
    orbital_pos: Position,
) -> Velocity {
    let circular_velocity = circular_velocity(central_pos, central_mass, orbital_pos);

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

// normal euler method
// Use current acc to update vel, and current vel to update pos.
pub fn _euler_update(
    cur_r: Position,
    cur_v: Velocity,
    cur_a: Acceleration,
    dt: f32,
) -> (Position, Velocity) {
    // v(t + dt) = v(t) + a(t)*dt
    let next_v = cur_v.update(&cur_a, dt);

    // r(t + dt) = r(t) + v(t)*dt
    let next_r = cur_r.update_const_v(&cur_v, dt);

    (next_r, next_v)
}

// symplectic euler method
// Use current acc to update vel, and NEXT vel to update pos.
// This incorporates some information about acceleration into the position update,
// making it conserve energy better over long periods than the standard euler method.
pub fn symplectic_euler_calc(
    cur_r: Position,
    cur_v: Velocity,
    cur_a: Acceleration,
    dt: f32,
) -> (Position, Velocity) {
    // v(t + dt) = v(t) + a(t)*dt
    let next_v = cur_v.update(&cur_a, dt);

    // r(t + dt) = r(t) + v(t + dt)*dt
    let next_r = cur_r.update_const_v(&next_v, dt);

    (next_r, next_v)
}

// Ek = .5mv^2
pub fn kinetic_energy(mass: f32, v: Velocity) -> f32 {
    let body_kinetic_mj = 0.5 * mass * v.mag().powi(2); // MJ

    body_kinetic_mj
}

pub fn gravitational_potential_energy(m1: f32, m2: f32, pos1: Position, pos2: Position) -> f32 {
    // Gravitational energy between two masses
    // Eg = -G * M * m / r
    let r = pos1.minus(pos2).mag();
    let grav_potential_kj = -G_KM * m1 * m2 / r; // KJ
    let grav_potential_mj = grav_potential_kj * 1e-3; // MJ

    grav_potential_mj
}

// Compute the barycenter - the point around which two bodies both orbit
// Same as center of mass for spherical bodies in normal conditions.
pub fn _barycenter(m1: f32, m2: f32, pos1: Position, pos2: Position) -> f32 {
    // barycenter of two masses, distance d apart
    // Rb = m2 / (m1 + m2) * d
    let d: f32 = pos1.minus(pos2).mag();

    let barycenter_position = m2 / (m1 + m2) * d;

    barycenter_position
}
