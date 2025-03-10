use crate::app::core::physics::{
    circ_velocity_barycenter, circular_velocity, Acceleration, Position, Velocity, R_EARTH_KM,
    R_MOON_KM,
};

#[derive(Default, Clone, Debug)]
pub struct Body {
    pub pos: Position,
    pub v: Velocity,
    pub mass: f32,
    pub radius: f32,
    pub trajectory: Vec<Body>,
    pub computed_a: Acceleration,
    pub is_fixed: bool,
    pub lock_to_circular_velocity: bool,
    pub lock_to_escape_velocity: bool,
    pub selected_vel_lock: usize,
}
impl Body {
    // returns a version of this struct to be used for the trajectory history
    // maybe make this a separate struct?
    pub fn new_history_entry(&self) -> Self {
        Self {
            pos: self.pos,
            v: self.v,
            mass: self.mass,
            radius: self.radius,
            computed_a: self.computed_a,
            is_fixed: self.is_fixed,
            lock_to_circular_velocity: self.lock_to_circular_velocity,
            lock_to_escape_velocity: self.lock_to_escape_velocity,
            selected_vel_lock: self.selected_vel_lock,
            trajectory: vec![],
        }
    }

    pub fn update(
        &mut self,
        new_pos: Position,
        new_vel: Velocity,
        new_acc: Acceleration,
    ) -> &mut Self {
        self.pos = new_pos;
        self.v = new_vel;
        self.computed_a = new_acc;
        self.trajectory.push(self.new_history_entry());

        self
    }

    // starting conditions for a low earth orbit, modeled after the ISS
    pub fn outer_low() -> Self {
        let earth_mass = Self::earth().mass;
        let earth_pos = Self::earth().pos;

        let r = 400. + R_EARTH_KM;
        let x = 3000_f32;
        let y = (r.powi(2) - x.powi(2)).sqrt();
        let position = Position::new(x, y);

        Self {
            mass: 400000., // kg
            pos: position,
            v: circular_velocity(earth_pos, earth_mass, position), // km/s
            trajectory: Vec::new(),
            ..Default::default()
        }
    }
    fn _outer_med() -> Self {
        Self {
            mass: 5000.,
            pos: Position::new(5000., 15000.),
            v: Velocity::new(3.9, 0.),
            ..Default::default()
        }
    }
    pub fn earth() -> Self {
        Self {
            mass: 5.97e24,      // kg
            radius: R_EARTH_KM, // km
            ..Default::default()
        }
    }
    pub fn moon() -> Self {
        let earth_mass = Self::earth().mass;
        let earth_pos = Self::earth().pos;
        let position = Position::new(0., 3.844e5 + R_EARTH_KM);
        let moon_mass = 7.34e22;

        Self {
            mass: moon_mass,
            radius: R_MOON_KM,
            pos: position,
            v: circ_velocity_barycenter(moon_mass, position, earth_mass, earth_pos).0, // km/s
            ..Default::default()
        }
    }
}

// if the object being pulled is 1000x more massive than the source of the gravity,
// then the gravitational force is negligible
pub fn is_mass_significant(source_body: &Body, body_under_effect: &Body) -> bool {
    let ratio_threshold = 1000.;
    (body_under_effect.mass / source_body.mass) < ratio_threshold
}

pub struct Preset {
    pub bodies: Vec<Body>,
    pub name: String,
}

impl Preset {
    pub fn defaults() -> Vec<Self> {
        let fixed_earth = Body {
            is_fixed: true,
            lock_to_circular_velocity: false,
            lock_to_escape_velocity: false,
            ..Body::earth()
        };
        let barycenter_earth = Body {
            is_fixed: false,
            lock_to_circular_velocity: true,
            selected_vel_lock: 1,
            ..Body::earth()
        };
        vec![
            Preset {
                bodies: vec![fixed_earth, Body::outer_low()],
                name: String::from("Small object orbiting Earth"),
            },
            Preset {
                bodies: vec![barycenter_earth, Body::moon()],
                name: String::from("Moon orbiting Earth"),
            },
        ]
    }
}
