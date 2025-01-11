use super::App;

pub struct UiState {
    pub panel_width: f32,
    pub start_pos: i32,
}

impl Default for UiState {
    fn default() -> Self {
        UiState {
            panel_width: 200.,
            start_pos: 0,
        }
    }
}

impl App {
    pub fn ui(&mut self, ctx: &egui::Context) {
        let panel = egui::SidePanel::left("main-ui-panel")
            .exact_width(self.ui_state.panel_width)
            .resizable(false);

        panel.show(ctx, |ui| {
            ui.label("Choose starting position");
            if self.started {
                ui.disable();
            }
            ui.add(egui::Slider::new(&mut self.ui_state.start_pos, 0..=1000));
            if ui.button("Start").clicked() {
                self.start();
            }
        });
    }

    fn start(&mut self) {
        let start_pos = super::Position::new(self.ui_state.start_pos as f32, 0.);
        self.hist = vec![start_pos];
        self.started = true;
    }
}
