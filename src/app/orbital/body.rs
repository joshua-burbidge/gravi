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
    pub name: String,
    pub default_expanded: bool,
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
            name: self.name.clone(),
            default_expanded: self.default_expanded,
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
            name: "Orbiting Object".to_string(),
            mass: 400000., // kg
            pos: position,
            v: circular_velocity(earth_pos, earth_mass, position), // km/s
            trajectory: Vec::new(),
            default_expanded: true,
            ..Default::default()
        }
    }
    fn _outer_med() -> Self {
        Self {
            mass: 5000.,
            pos: Position::new(5000., 15000.),
            v: Velocity::new(3.9, 0.),
            default_expanded: true,
            ..Default::default()
        }
    }
    pub fn earth() -> Self {
        Self {
            name: "Earth".to_string(),
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
            name: "Moon".to_string(),
            mass: moon_mass,
            radius: R_MOON_KM,
            pos: position,
            v: circ_velocity_barycenter(moon_mass, position, earth_mass, earth_pos).0, // km/s
            default_expanded: true,
            ..Default::default()
        }
    }
    pub fn sun() -> Self {
        Self {
            name: "Sun".to_string(),
            pos: Position::new(0., 0.),
            mass: 1.989e30,
            radius: 6.963e5,
            is_fixed: true,
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

#[derive(Default)]
pub struct Preset {
    pub bodies: Vec<Body>,
    pub name: String,
    pub distance_per_px: i32,
    pub dt: f32,
    pub ticks_per_press: i32,
}

impl Preset {
    fn default() -> Self {
        Self {
            dt: 1.0,
            ticks_per_press: 1000,
            ..Default::default()
        }
    }
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
        let moon_orbiting_earth = Body {
            lock_to_circular_velocity: true,
            selected_vel_lock: 0,
            ..Body::moon()
        };
        vec![
            Preset {
                bodies: vec![fixed_earth, Body::outer_low()],
                name: String::from("Small object orbiting Earth"),
                distance_per_px: 150,
                ..Preset::default()
            },
            Preset {
                bodies: vec![barycenter_earth, moon_orbiting_earth],
                name: String::from("Moon orbiting Earth"),
                distance_per_px: 4000,
                ticks_per_press: 100000,
                ..Preset::default()
            },
            Self::three_body(),
            Self::sun_earth_moon(),
        ]
    }

    pub fn three_body() -> Self {
        let b1 = Body {
            name: String::from("1"),
            radius: 1000.,
            mass: 1e21,
            pos: Position::new(-5000., -5000.),
            lock_to_circular_velocity: true,
            selected_vel_lock: 1,
            ..Default::default()
        };
        let b2 = Body {
            name: String::from("2"),
            pos: Position::new(0., 5000.),
            lock_to_circular_velocity: true,
            selected_vel_lock: 0,
            ..b1.clone()
        };
        let b3: Body = Body {
            name: String::from("3"),
            pos: Position::new(7000., -5000.),
            v: Velocity::new(-0.2, 0.08),
            lock_to_circular_velocity: false,
            ..b1.clone()
        };

        Self {
            name: String::from("Three body"),
            distance_per_px: 300,
            bodies: vec![b1, b2, b3],
            ..Preset::default()
        }
    }

    pub fn sun_earth_moon() -> Self {
        let sun = Body::sun();
        let earth = Body {
            pos: Position::new(0., 149597870_f32),
            lock_to_circular_velocity: true,
            selected_vel_lock: 0,
            ..Body::earth()
        };
        let default_moon = Body::moon();
        let moon = Body {
            pos: earth.pos.add(default_moon.pos),
            lock_to_circular_velocity: true,
            selected_vel_lock: 1,
            ..default_moon
        };

        Self {
            name: "Sun + earth + moon".to_string(),
            bodies: vec![sun, earth, moon],
            distance_per_px: 1400000,
            dt: 50.,
            ticks_per_press: 100000,
        }
    }
}
