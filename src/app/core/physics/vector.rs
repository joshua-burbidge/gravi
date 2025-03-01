#[derive(Clone, Copy, Debug, Default)]
pub struct Pos;
#[derive(Clone, Copy, Debug, Default)]
pub struct Vel;
#[derive(Clone, Copy, Debug, Default)]
pub struct Acc;

pub trait VectorType {}
impl VectorType for Pos {}
impl VectorType for Vel {}
impl VectorType for Acc {}

pub type Position = Vector<Pos>;
pub type Velocity = Vector<Vel>;
pub type Acceleration = Vector<Acc>;

#[derive(Clone, Copy, Debug, Default)]
pub struct Vector<T: VectorType> {
    _type: T,
    pub x: f32,
    pub y: f32,
}

impl<T: VectorType> Vector<T> {
    pub fn mag(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
    pub fn scale(self, factor: f32) -> Self {
        Self::new_vec(self._type, self.x * factor, self.y * factor)
    }
    pub fn add(self, vec2: Vector<T>) -> Self {
        Self::new_vec(self._type, self.x + vec2.x, self.y + vec2.y)
    }
    pub fn minus(self, vec2: Vector<T>) -> Self {
        self.add(vec2.scale(-1.))
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
