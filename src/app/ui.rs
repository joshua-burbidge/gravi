use super::App;

impl App {
    pub fn ui(&mut self, ctx: &egui::Context) {
        let panel = egui::SidePanel::left("main-ui-panel")
            .exact_width(self.ui_state.panel_width)
            .resizable(false);

        panel.show(ctx, |ui| {
            ui.label("Hello, egui!");
            ui.label("Hello, egui!");
            ui.add(egui::Slider::new(&mut self.ui_state.start_pos, 0..=1000));
            if ui.button("Start").clicked() {
                self.start();
            }
        });
    }
}
