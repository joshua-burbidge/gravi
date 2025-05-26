pub use vector::{Acceleration, Position, Velocity};

use crate::app::orbital::body::Body;

mod vector;

pub const G: f32 = 6.674e-11; // N m^2 / kg^2
pub const G_KM: f32 = G * 1e-6; // N km^2 / kg^2 (converted to km)
pub const R_EARTH_KM: f32 = 6378.;
pub const R_MOON_KM: f32 = 1740.;
pub const SUN_EARTH_R_KM: f32 = 149597870.;

pub enum Axis {
    X,
    Y,
}

// calculate the magnitude of the circular velocity
// v = sqrt(GM/r)
fn circular_velocity_magnitude(central_mass: f32, r: f32) -> f32 {
    // 1e9 converts from m/s to km/s
    (G * central_mass / (r * 1e9)).sqrt()
}
// The velocity that results in a perfect circular orbit given the central mass and the radius,
// assumes that the central mass will not move
pub fn circular_velocity(
    central_pos: Position,
    central_mass: f32,
    orbital_pos: Position,
) -> Velocity {
    let r = orbital_pos.minus(central_pos);
    let r_mag = r.mag();

    let circular_velocity_magnitude = circular_velocity_magnitude(central_mass, r_mag);

    // Positions determine the direction that the velocity should point (perpendicular to the position vector).
    // Then multiply by the magnitude.
    let pos_unit_vector = r.scale(1. / r_mag);
    let vel_unit_vector = Velocity::from(pos_unit_vector.perpendicular_cw());
    vel_unit_vector.scale(circular_velocity_magnitude)
}

// return circular velocity of body 1, based on the influence of body 2
pub fn circ_velocity_barycenter(
    m1: f32,
    pos1: Position,
    m2: f32,
    pos2: Position,
) -> (Velocity, Velocity) {
    // calculate Vc of the whole sytem orbiting around the barycenter
    let overall_vc = circular_velocity(pos2, m1 + m2, pos1);

    // split the whole vc based on mass ratio to get individual velocities
    let v1 = overall_vc.scale(m2 / (m1 + m2));
    let v2 = overall_vc.scale(-m1 / (m1 + m2));

    (v1, v2)
}

pub fn circ_velocity_bodies(body1: &Body, body2: &Body) -> (Velocity, Velocity) {
    let r = body2.absolute_pos.minus(body1.absolute_pos);

    let circ_v_mag = circular_velocity_magnitude(body1.mass + body2.mass, r.mag());

    let r_unit_vector = r.divide(r.mag());
    let v_unit_vector = r_unit_vector.perpendicular_cw();

    let vc = Velocity::from(v_unit_vector.scale(circ_v_mag));

    let mass_sum = body1.mass + body2.mass;
    let v1 = vc.scale(-body2.mass / mass_sum);
    let v2 = vc.scale(body1.mass / mass_sum);

    (v1, v2)
}

// escape velocity = sqrt(2) * circular_velocity
pub fn _escape_velocity(
    central_pos: Position,
    central_mass: f32,
    orbital_pos: Position,
) -> Velocity {
    let circular_velocity = circular_velocity(central_pos, central_mass, orbital_pos);

    circular_velocity.scale(2_f32.sqrt())
}

// escape velocity = sqrt(2) * circular_velocity
pub fn escape_velocity_barycenter(m1: f32, pos1: Position, m2: f32, pos2: Position) -> Velocity {
    let (circular_velocity, _) = circ_velocity_barycenter(m1, pos1, m2, pos2);

    circular_velocity.scale(2_f32.sqrt())
}

pub fn gravitational_acceleration(
    central_pos: Position,
    orbital_pos: Position,
    central_mass: f32,
) -> Acceleration {
    let r = orbital_pos.minus(central_pos);

    // a = -G * m_central * r_vec / (|r_vec|^3)
    let cur_a = r.scale(-G_KM * central_mass / (r.mag().powi(3)));
    let cur_a_km = cur_a.scale(1e-3);

    Acceleration::from(cur_a_km)
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

pub fn gravitational_potential_energy(m1: f32, m2: f32, pos1: Position, pos2: Position) -> f64 {
    // Gravitational energy between two masses
    // Eg = -G * M * m / r
    let r = pos1.minus(pos2).mag();

    let (m1, m2, r) = (m1 as f64, m2 as f64, r as f64);

    let grav_potential_kj = -G_KM as f64 * m1 * m2 / r; // KJ
    let grav_potential_mj = grav_potential_kj * 1e-3; // MJ

    grav_potential_mj
}

// Compute the barycenter of multiple bodies - the point around which two bodies both orbit
// Same as center of mass for spherical bodies in normal conditions.
// Rb = m1*r1 + m2*r2 + ... +mn*rn / (m1 + m2 + ... + mn)
pub fn barycenter(bodies: &Vec<Body>) -> Position {
    let mass_sum = bodies.iter().fold(0., |acc, b| acc + b.mass);
    let weighted_pos_sum = bodies
        .iter()
        .fold(Position::default(), |acc, b| acc.add(b.pos.scale(b.mass)));

    weighted_pos_sum.divide(mass_sum)
}
pub fn barycenter_abs(bodies: &Vec<Body>) -> Position {
    let mass_sum = bodies.iter().fold(0., |acc, b| acc + b.mass);
    let weighted_pos_sum = bodies.iter().fold(Position::default(), |acc, b| {
        acc.add(b.absolute_pos.scale(b.mass))
    });

    weighted_pos_sum.divide(mass_sum)
}
