use crate::app::core::physics::{
    circ_velocity_barycenter, circ_velocity_bodies, circular_velocity, Acceleration, Position,
    Velocity, R_EARTH_KM, R_MOON_KM, SUN_EARTH_R_KM,
};

#[derive(Clone, Debug)]
pub struct Body {
    pub name: String,
    pub pos: Position,
    pub v: Velocity,
    pub mass: f32,
    pub radius: f32,
    pub trajectory: Vec<Body>,
    pub computed_a: Acceleration,
    pub absolute_pos: Position,
    pub is_fixed: bool,
    pub is_barycenter: bool,
    pub lock_to_circular_velocity: bool,
    pub lock_to_escape_velocity: bool,
    pub selected_vel_lock: usize,
    pub color: (u8, u8, u8),
    pub default_expanded: bool,
}
impl Default for Body {
    fn default() -> Self {
        Body {
            pos: Position::default(),
            v: Velocity::default(),
            mass: Default::default(),
            radius: Default::default(),
            trajectory: Default::default(),
            computed_a: Acceleration::default(),
            absolute_pos: Position::default(),
            is_fixed: Default::default(),
            is_barycenter: false,
            lock_to_circular_velocity: Default::default(),
            lock_to_escape_velocity: Default::default(),
            selected_vel_lock: Default::default(),
            name: Default::default(),
            color: (0, 255, 0),
            default_expanded: Default::default(),
        }
    }
}
impl Body {
    // returns a version of this struct to be used for the trajectory history
    // maybe make this a separate struct?
    pub fn copy(&self) -> Self {
        Self {
            pos: self.pos,
            v: self.v,
            mass: self.mass,
            radius: self.radius,
            computed_a: self.computed_a,
            absolute_pos: self.absolute_pos,
            is_fixed: self.is_fixed,
            is_barycenter: self.is_barycenter,
            lock_to_circular_velocity: self.lock_to_circular_velocity,
            lock_to_escape_velocity: self.lock_to_escape_velocity,
            selected_vel_lock: self.selected_vel_lock,
            name: self.name.clone(),
            color: self.color.clone(),
            default_expanded: self.default_expanded,
            trajectory: vec![],
        }
    }

    pub fn update(
        &mut self,
        new_pos: Position,
        new_vel: Velocity,
        new_acc: Acceleration,
        parent_abs_pos: Position,
    ) {
        self.pos = new_pos;
        self.absolute_pos = parent_abs_pos.add(new_pos);
        self.v = new_vel;
        self.computed_a = new_acc;
        self.trajectory.push(self.copy());
    }

    // --------------- Constructors ------------------
    // starting conditions for a low earth orbit, modeled after the ISS
    pub fn outer_low() -> Self {
        let earth_mass = Self::earth().mass;
        let earth_pos = Self::earth().absolute_pos;

        let r = 400. + R_EARTH_KM;
        let x = 3000_f32;
        let y = (r.powi(2) - x.powi(2)).sqrt();
        let position = Position::new(x, y);

        Self {
            name: "Orbiting Object".to_string(),
            mass: 400000., // kg
            absolute_pos: position,
            v: circular_velocity(earth_pos, earth_mass, position), // km/s
            trajectory: Vec::new(),
            default_expanded: true,
            color: (255, 0, 0),
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
        let earth_pos = Self::earth().absolute_pos;
        let position = Position::new(0., 3.844e5 + R_EARTH_KM);
        let moon_mass = 7.34e22;

        Self {
            name: "Moon".to_string(),
            mass: moon_mass,
            radius: R_MOON_KM,
            absolute_pos: position,
            v: circ_velocity_barycenter(moon_mass, position, earth_mass, earth_pos).0, // km/s
            default_expanded: true,
            color: (160, 160, 160),
            ..Default::default()
        }
    }
    pub fn sun() -> Self {
        Self {
            name: "Sun".to_string(),
            absolute_pos: Position::new(0., 0.),
            mass: 1.989e30,
            radius: 6.963e5,
            is_fixed: false,
            color: (255, 255, 0),
            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct Preset {
    pub bodies: Vec<Body>,
    pub name: String,
    pub distance_per_px: i32,
    pub dt: f32,
    pub ticks_per_press: i32,
    pub draw_frequency: u32,
}

impl Preset {
    fn default() -> Self {
        Self {
            dt: 1.0,
            ticks_per_press: 1000,
            draw_frequency: 100,
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

        vec![
            Preset {
                bodies: vec![fixed_earth, Body::outer_low()],
                name: String::from("Small object orbiting Earth"),
                distance_per_px: 150,
                ..Preset::default()
            },
            Self::sun_earth_moon(),
            Self::earth_moon(),
            Self::three_body(),
            Self::equal_binary(),
            Self::unequal_binary(),
            Self::hierarchy_test(),
        ]
    }

    pub fn earth_moon() -> Self {
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

        Preset {
            bodies: vec![barycenter_earth, moon_orbiting_earth],
            name: String::from("Moon orbiting Earth"),
            distance_per_px: 4000,
            ticks_per_press: 100000,
            ..Preset::default()
        }
    }
    pub fn three_body() -> Self {
        let b1 = Body {
            name: String::from("1"),
            radius: 1000.,
            mass: 1e21,
            absolute_pos: Position::new(-5000., -5000.),
            lock_to_circular_velocity: true,
            selected_vel_lock: 1,
            color: (0, 255, 0),
            ..Default::default()
        };
        let b2 = Body {
            name: String::from("2"),
            absolute_pos: Position::new(0., 5000.),
            lock_to_circular_velocity: true,
            selected_vel_lock: 0,
            color: (255, 0, 0),
            ..b1.clone()
        };
        let b3: Body = Body {
            name: String::from("3"),
            absolute_pos: Position::new(7000., -5000.),
            v: Velocity::new(-0.2, 0.08),
            lock_to_circular_velocity: false,
            color: (0, 70, 180),
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
            absolute_pos: Position::new(0., SUN_EARTH_R_KM),
            lock_to_circular_velocity: true,
            selected_vel_lock: 2,
            ..Body::earth()
        };

        let default_moon = Body::moon();
        let moon = Body {
            absolute_pos: earth.absolute_pos.add(default_moon.absolute_pos),
            lock_to_circular_velocity: true,
            selected_vel_lock: 1,
            ..default_moon
        };

        Self {
            name: "Sun + Earth + Moon".to_string(),
            bodies: vec![sun, earth, moon],
            distance_per_px: 1400000,
            dt: 50.,
            ticks_per_press: 100000,
            draw_frequency: 24 * 60 * 60,
        }
    }

    pub fn hierarchy_test() -> Self {
        let base = Self::sun_earth_moon();

        let cluster_y = -599_597_870_f32;
        let earth_2 = Body {
            name: "Earth 2".to_string(),
            absolute_pos: Position::new(0., cluster_y - 5_978_700.),
            ..Body::earth()
        };
        let moon_2 = Body {
            name: "Moon 2".to_string(),
            absolute_pos: earth_2.absolute_pos.add(Body::moon().absolute_pos),
            ..Body::moon()
        };

        let earth_3 = Body {
            name: "Earth 3".to_string(),
            absolute_pos: Position::new(0., cluster_y + 9_597_870.),
            ..Body::earth()
        };
        let moon_3 = Body {
            name: "Moon 3".to_string(),
            absolute_pos: earth_3.absolute_pos.add(Body::moon().absolute_pos),
            ..Body::moon()
        };
        let third = Body {
            name: "Third body".to_string(),
            absolute_pos: earth_3.absolute_pos.minus(Body::moon().absolute_pos),
            ..Body::moon()
        };
        let small = Body {
            name: "Small test particle".to_string(),
            mass: 10.,
            absolute_pos: third.absolute_pos.add(Position::new(100., 100.)),
            ..Body::default()
        };

        let bodies = [
            base.bodies,
            vec![earth_2, moon_2, earth_3, moon_3, third, small],
        ]
        .concat();

        Self {
            name: "hierarchy_test".to_string(),
            bodies,
            ..base
        }
    }

    // binary system with equal masses and circular velocities
    // both bodies will move in the exact same circle
    pub fn equal_binary() -> Self {
        let body1 = Body {
            name: "1".to_string(),
            mass: 1.23e22,
            radius: 8000.,
            absolute_pos: Position::new(50000., 0.),
            lock_to_circular_velocity: true,
            selected_vel_lock: 1,
            default_expanded: true,
            ..Body::default()
        };
        let body2 = Body {
            name: "2".to_string(),
            absolute_pos: Position::new(-50000., 0.),
            selected_vel_lock: 0,
            color: (220, 0, 0),
            ..body1.clone()
        };

        Self {
            name: "Equal circular binary system".to_string(),
            bodies: vec![body1, body2],
            distance_per_px: 1000,
            dt: 10.,
            ticks_per_press: 10000,
            draw_frequency: 24 * 60 * 60,
            ..Preset::default()
        }
    }

    // binary system with unequal masses
    // If circular velocity is enabled, both bodies will travel in different-sized circles.
    // If velocities are slightly changed, both bodies will travel in elliptical orbits.
    // If the overall system velocity is changed, then both bodies will be orbiting a moving barycenter.
    pub fn unequal_binary() -> Self {
        let body1_pos = Position::new(-100000., 150000.);
        let body2_pos = Position::new(-200000., 150000.);

        let mut body1 = Body {
            name: "1".to_string(),
            mass: 6.23e22,
            radius: 8000.,
            absolute_pos: body1_pos,
            lock_to_circular_velocity: false,
            selected_vel_lock: 1,
            default_expanded: true,
            ..Body::default()
        };
        let body2 = Body {
            name: "2".to_string(),
            mass: 1.23e22,
            absolute_pos: body2_pos,
            lock_to_circular_velocity: true,
            selected_vel_lock: 0,
            color: (220, 0, 0),
            ..body1.clone()
        };

        let (body1_circ_v, _body2_circ_v) = circ_velocity_bodies(&body1, &body2);
        body1.v = body1_circ_v.add(Velocity::new(0.02, -0.02));

        Self {
            name: "Unequal binary system".to_string(),
            bodies: vec![body1, body2],
            distance_per_px: 2000,
            dt: 10.,
            ticks_per_press: 10000,
            draw_frequency: 24 * 60 * 60,
            ..Preset::default()
        }
    }
}
