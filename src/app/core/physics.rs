pub const G: f32 = 6.674e-11; // N m^2 / kg^2
pub const G_KM: f32 = G * 1e-6; // N km^2 / kg^2 (converted to km)

#[derive(Clone, Debug, Default)]
pub struct Position;
#[derive(Clone, Debug, Default)]
pub struct Velocity;
#[derive(Clone, Debug, Default)]
pub struct Acceleration;
// type VecType = Position | Velocity | Acceleration;

#[derive(Clone, Debug, Default)]
pub struct Vector<T> {
    vec_type: T,
    pub x: f32,
    pub y: f32,
}
// TODO define trait VectorType, then do <T: VectorType> to restrict <T> to be only the valid vector types

// TODO do this
// pub type P = Vector<Position>;

impl<T> Vector<T> {
    pub fn mag(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
    fn new_vec(vec_type: T, x: f32, y: f32) -> Self {
        Self { vec_type, x, y }
    }
}

impl Vector<Position> {
    pub fn new(x: f32, y: f32) -> Self {
        Vector::new_vec(Position, x, y)
    }

    // position after t seconds given constant acceleration
    pub fn update(
        &self,
        v: &Vector<Velocity>,
        a: &Vector<Acceleration>,
        t: f32,
    ) -> Vector<Position> {
        // px + vx t + 1/2 ax t^2
        Vector::<Position>::new(
            self.x + v.x * t + 0.5 * a.x * t.powi(2),
            self.y + v.y * t + 0.5 * a.x * t.powi(2),
        )
    }

    pub fn update_const_v(&self, v: &Vector<Velocity>, t: f32) -> Vector<Position> {
        let zero_a = Vector::<Acceleration>::default();

        self.update(v, &zero_a, t)
    }
}

impl Vector<Velocity> {
    pub fn new(x: f32, y: f32) -> Self {
        Vector::new_vec(Velocity, x, y)
    }

    // velocity update after t seconds given constant acceleration
    pub fn update(&self, a: &Vector<Acceleration>, t: f32) -> Vector<Velocity> {
        // vx + ax t
        Vector::<Velocity>::new(self.x + a.x * t, self.y + a.y * t)
    }
}

impl Vector<Acceleration> {
    pub fn new(x: f32, y: f32) -> Self {
        Vector::new_vec(Acceleration, x, y)
    }
}
