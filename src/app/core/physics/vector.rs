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

impl<T: VectorType + Default> Vector<T> {
    pub fn mag(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
    pub fn scale(self, factor: f32) -> Self {
        Self::new_vec(self._type, self.x * factor, self.y * factor)
    }
    pub fn divide(self, divisor: f32) -> Self {
        self.scale(1. / divisor)
    }
    pub fn add(self, vec2: Vector<T>) -> Self {
        Self::new_vec(self._type, self.x + vec2.x, self.y + vec2.y)
    }
    pub fn minus(self, vec2: Vector<T>) -> Self {
        self.add(vec2.scale(-1.))
    }
    pub fn abs_diff(self, vec2: Vector<T>) -> f32 {
        self.minus(vec2).mag()
    }
    pub fn perpendicular_cw(self) -> Self {
        Self::new_vec(self._type, self.y, -self.x)
    }

    pub fn from<SourceType: VectorType>(v: Vector<SourceType>) -> Self {
        Vector::new_vec(T::default(), v.x, v.y)
    }
    pub fn to_tuple(self) -> (f32, f32) {
        (self.x, self.y)
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
        let vel_term = Position::from(v.scale(t));
        let acc_term = Position::from(a.scale(0.5 * t.powi(2)));
        self.add(vel_term).add(acc_term)
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
        let vel_change = Velocity::from(a.scale(t));
        self.add(vel_change)
    }
}

impl Vector<Acc> {
    pub fn new(x: f32, y: f32) -> Self {
        Vector::new_vec(Acc, x, y)
    }
}
