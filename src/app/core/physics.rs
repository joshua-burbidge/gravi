pub const G: f32 = 6.674e-11; // N m^2 / kg^2
pub const G_KM: f32 = G * 1e-6; // N km^2 / kg^2 (converted to km)
pub const R_EARTH_KM: f32 = 6378.;

#[derive(Clone, Debug, Default)]
pub struct Pos;
#[derive(Clone, Debug, Default)]
pub struct Vel;
#[derive(Clone, Debug, Default)]
pub struct Acc;

pub trait VectorType {}
impl VectorType for Pos {}
impl VectorType for Vel {}
impl VectorType for Acc {}

pub type Position = Vector<Pos>;
pub type Velocity = Vector<Vel>;
pub type Acceleration = Vector<Acc>;

#[derive(Clone, Debug, Default)]
pub struct Vector<T: VectorType> {
    _type: T,
    pub x: f32,
    pub y: f32,
}

impl<T: VectorType> Vector<T> {
    pub fn mag(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
    fn new_vec(vec_type: T, x: f32, y: f32) -> Self {
        Self {
            _type: vec_type,
            x,
            y,
        }
    }
}

impl Vector<Pos> {
    pub fn new(x: f32, y: f32) -> Self {
        Vector::new_vec(Pos, x, y)
    }

    // position after t seconds given constant acceleration
    pub fn update(&self, v: &Velocity, a: &Acceleration, t: f32) -> Self {
        // px + vx t + 1/2 ax t^2
        Self::new(
            self.x + v.x * t + 0.5 * a.x * t.powi(2),
            self.y + v.y * t + 0.5 * a.x * t.powi(2),
        )
    }

    pub fn update_const_v(&self, v: &Velocity, t: f32) -> Self {
        let zero_a = Acceleration::default();

        self.update(v, &zero_a, t)
    }
}

impl Vector<Vel> {
    pub fn new(x: f32, y: f32) -> Self {
        Vector::new_vec(Vel, x, y)
    }

    // velocity update after t seconds given constant acceleration
    pub fn update(&self, a: &Acceleration, t: f32) -> Self {
        // vx + ax t
        Self::new(self.x + a.x * t, self.y + a.y * t)
    }
}

impl Vector<Acc> {
    pub fn new(x: f32, y: f32) -> Self {
        Vector::new_vec(Acc, x, y)
    }
}

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
