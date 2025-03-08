use egui::RichText;

use crate::ui::widgets::{CustomSlider, XYInput};

use super::Orbital;

pub fn ui(app: &mut Orbital, ctx: &egui::Context) {
    let (kinetic, potential, diff_percent) = (
        app.analysis.kinetic_e,
        app.analysis.gravitational_e,
        app.analysis.diff_percentage,
    );
    let presets: Vec<String> = app.presets.iter().map(|p| p.name.clone()).collect();

    let panel = egui::SidePanel::left("main-ui-panel")
        .exact_width(app.ui_state.panel_width)
        .resizable(false);
    panel.show(ctx, |ui| {
        egui::CollapsingHeader::new(RichText::new("Select preset simulation").heading())
            .default_open(true)
            .show(ui, |ui| {
                for (i, preset) in presets.iter().enumerate() {
                    if ui.button(preset).clicked() {
                        app.load_preset(i);
                    }
                }
            });

        ui.add_space(20.);

        ui.add_enabled_ui(!app.started, |ui| {
            ui.label(RichText::new("General").heading());
            ui.add(CustomSlider::new(&mut app.dt, 0.01..=10.0).label("dt:"));
            ui.add(CustomSlider::new(&mut app.num_ticks, 100..=100000).label("ticks per press:"));
            ui.add_space(20.);

            ui.input(|i| {
                if i.key_pressed(egui::Key::A) {
                    app.reset();
                }
                if !app.started && i.key_pressed(egui::Key::Enter) {
                    app.start();
                }
            });

            let len = app.bodies.len();

            for (i, body) in app.bodies.iter_mut().enumerate() {
                let x_range = -10000.0..=10000.;
                let y_range = -10000.0..=10000.;

                egui::CollapsingHeader::new(RichText::new(format!("Body {}:", i + 1)).heading())
                    .show(ui, |ui| {
                        ui.label("Position");
                        ui.add(XYInput::new(
                            &mut body.pos.x,
                            &mut body.pos.y,
                            x_range,
                            y_range,
                        ));
                        ui.label(format!("|r|: {} km", body.pos.mag()));

                        if !body.is_fixed {
                            ui.label("Velocity");
                            ui.checkbox(
                                &mut body.lock_to_circular_velocity,
                                "lock to circular velocity",
                            );
                            ui.checkbox(
                                &mut body.lock_to_escape_velocity,
                                "lock to escape velocity",
                            );

                            let vel_lock_enabled =
                                body.lock_to_circular_velocity || body.lock_to_escape_velocity;
                            ui.add_enabled_ui(vel_lock_enabled, |ui| {
                                egui::ComboBox::from_label("Around body").show_index(
                                    ui,
                                    &mut body.selected_vel_lock,
                                    len,
                                    |i| format!("Body {}", i + 1),
                                );
                            });
                            ui.add_enabled_ui(!vel_lock_enabled, |ui| {
                                ui.add(XYInput::new(
                                    &mut body.v.x,
                                    &mut body.v.y,
                                    -50.0..=50.0,
                                    -50.0..=50.0,
                                ));
                            });
                        }

                        ui.label("Mass");
                        ui.add(CustomSlider::new(&mut body.mass, 1.0..=5e10).label("M:"));

                        ui.monospace("Acceleration (km/s^2)");
                        ui.monospace(format!("Ax:    {:+.4e}", body.computed_a.x));
                        ui.monospace(format!("Ay:    {:+.4e}", body.computed_a.y));
                    });
                ui.add_space(20.);
            }

            app.set_velocities();

            if ui.button("Start").clicked() {
                app.start();
            }
        });

        let t = app.t();
        let days = t / (60 * 60 * 24) as f32;
        ui.monospace(format!("t: {:.4e} s, {:.2} d", t, days));
        ui.monospace("Energy (MJ)");
        ui.monospace(format!("Kinetic:    {:+.4e}", kinetic));
        ui.monospace(format!("Potential:  {:+.4e}", potential));
        ui.monospace(format!("Total:      {:+.4e}", kinetic + potential));
        ui.monospace(format!("Initial:    {:+.4e}", app.analysis.initial_e));
        ui.monospace(format!("Diff:        {:.2}%", diff_percent));
        ui.monospace(format!("Diff per t:  {:.2e}%", (100. - diff_percent) / t));
    });
}
