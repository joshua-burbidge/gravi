use super::{Accel, App, Velocity};

pub struct UiState {
    pub panel_width: f32,
    pub start_pos: i32,
    accel: f32,
    vx: f32,
    vy: f32,
}

impl Default for UiState {
    fn default() -> Self {
        UiState {
            panel_width: 300.,
            start_pos: 0,
            accel: 0.,
            vx: 0.,
            vy: 0.,
        }
    }
}

impl App {
    pub fn ui(&mut self, ctx: &egui::Context) {
        let panel = egui::SidePanel::left("main-ui-panel")
            .exact_width(self.ui_state.panel_width)
            .resizable(false);

        panel.show(ctx, |ui| {
            ui.style_mut().spacing.slider_width = ui.available_width() - 50.0;

            if self.started {
                ui.disable();
            }
            ui.label("Choose starting position");
            ui.add(egui::Slider::new(&mut self.ui_state.start_pos, 0..=1000));
            ui.label("Choose starting velocity");
            ui.horizontal(|ui| {
                ui.label("X:");
                ui.style_mut().spacing.slider_width = ui.available_width() - 50.0;
                ui.add(egui::Slider::new(&mut self.ui_state.vx, 0.0..=100.));
            });
            ui.horizontal(|ui| {
                ui.label("Y:");
                ui.style_mut().spacing.slider_width = ui.available_width() - 50.0;
                ui.add(egui::Slider::new(&mut self.ui_state.vy, 0.0..=100.));
            });
            ui.label("Apply additional constant y acceleration");
            ui.add(egui::Slider::new(&mut self.ui_state.accel, -99.9..=99.9));
            if ui.button("Start").clicked() {
                self.start();
            }
        });
    }

    fn start(&mut self) {
        let start_pos = super::Position::new(self.ui_state.start_pos as f32, 0.);
        self.hist = vec![start_pos];
        self.started = true;

        self.a = Accel {
            x: self.a.x,
            y: self.a.y + self.ui_state.accel,
        };
        self.v = Velocity {
            x: self.ui_state.vx,
            y: self.ui_state.vy,
        }
    }
}
