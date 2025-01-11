// use super::{Acceleration, ConstAccel, Velocity};

// pub struct UiState {
//     pub panel_width: f32,
//     pub start_pos: i32,
//     pub accel: f32,
//     pub vx: f32,
//     pub vy: f32,
// }

// impl Default for UiState {
//     fn default() -> Self {
//         UiState {
//             panel_width: 300.,
//             start_pos: 0,
//             accel: 0.,
//             vx: 0.,
//             vy: 0.,
//         }
//     }
// }

// impl ConstAccel {
//     pub fn start(&mut self) {
//         let start_pos = super::Position::new(self.ui_state.start_pos as f32, 0.);
//         self.hist = vec![start_pos];
//         self.started = true;

//         self.a = Acceleration {
//             x: self.a.x,
//             y: self.a.y + self.ui_state.accel,
//         };
//         self.v = Velocity {
//             x: self.ui_state.vx,
//             y: self.ui_state.vy,
//         }
//     }
// }
