use femtovg::{Color, Paint, Path};

use crate::ui::widgets::{CustomSlider, XYInput};

use super::{
    core::{Acceleration, Position, Velocity},
    App,
};

pub struct Orbital {
    ui_state: UiState,
    started: bool,
    central: Body,
    outer: Body,
}

impl App for Orbital {
    fn run(&mut self) {}

    fn draw(&mut self, canvas: &mut femtovg::Canvas<femtovg::renderer::WGPURenderer>) {
        let mut path = Path::new();
        let paint = Paint::color(Color::white()).with_line_width(2.);

        let central_px = convert_pos_to_canvas(&self.central.pos);
        let outer_px = convert_pos_to_canvas(&self.outer.pos);
        path.circle(outer_px.x, outer_px.y, 5.);
        path.circle(central_px.x, central_px.y, 20.);

        canvas.fill_path(&path, &paint);
    }

    fn ui(&mut self, ctx: &egui::Context) {
        let panel = egui::SidePanel::left("main-ui-panel")
            .exact_width(self.ui_state.panel_width)
            .resizable(false);
        panel.show(ctx, |ui| {
            if self.started {
                ui.disable();
            }
            ui.label("Central body");
            let x_range = 0.0..=1000.;
            let y_range = -500.0..=500.;
            ui.add(XYInput::new(
                &mut self.central.pos.x,
                &mut self.central.pos.y,
                x_range,
                y_range,
            ));

            ui.add(
                CustomSlider::new(&mut self.central.mass, 0.0..=10000.)
                    .label("M:")
                    .full_width(true),
            );

            // add ui for outer
            if ui.button("Start").clicked() {
                self.start();
            }
        });
    }
    fn panel_width(&self) -> f32 {
        self.ui_state.panel_width
    }
}

impl Orbital {
    pub fn new() -> Self {
        Self {
            ui_state: UiState::new(),
            started: false,
            central: Body::default(),
            outer: Body::default(),
        }
    }

    fn start(&mut self) {
        self.started = true;
    }
}

#[derive(Default)]
struct Body {
    pos: Position,
    v: Velocity,
    a: Acceleration,
    mass: f32,
}
struct UiState {
    panel_width: f32,
}
impl UiState {
    fn new() -> Self {
        Self { panel_width: 300. }
    }
}

fn convert_pos_to_canvas(pos: &Position) -> Position {
    Position {
        x: pos.x,
        y: -pos.y,
    }
}
