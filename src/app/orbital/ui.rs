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
                        app.reset();
                        app.load_preset(i);
                    }
                }
            });

        ui.add_space(20.);

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

        let bodies_list = app.bodies_list();
        let started = app.started;

        for (i, body) in app.bodies_vec_mut().iter_mut().enumerate() {
            let x_range = -10000.0..=10000.;
            let y_range = -10000.0..=10000.;

            let id = ui.make_persistent_id(format!("collapsing_{}", i));
            let collapsing = egui::collapsing_header::CollapsingState::load_with_default_open(
                ui.ctx(),
                id,
                body.default_expanded,
            );

            let response = collapsing
                .show_header(ui, |ui| {
                    ui.add(
                        egui::Label::new(RichText::new(bodies_list[i].clone()).heading().color(
                            egui::Color32::from_rgb(body.color.0, body.color.1, body.color.2),
                        ))
                        .wrap(),
                    )
                })
                .body(|ui| {
                    ui.add_enabled_ui(!started, |ui| {
                        text_sized(ui, "Position (km)", 14.);
                        ui.add(XYInput::new(
                            &mut body.absolute_pos.x,
                            &mut body.absolute_pos.y,
                            x_range,
                            y_range,
                        ));
                        ui.label(format!("|r|: {} km", body.pos.mag()));
                        ui.add_space(6.);

                        if !body.is_fixed {
                            text_sized(ui, "Velocity (km/s)", 14.);
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
                                    bodies_list.len(),
                                    |i| bodies_list[i].clone(),
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
                            ui.label(format!("|v|: {} km/s", body.v.mag()));

                            ui.add_space(4.);
                        }

                        text_sized(ui, "Mass (kg)", 14.);
                        ui.add(CustomSlider::new(&mut body.mass, 1.0..=5e10).label("M:"));
                        ui.add_space(6.);

                        text_sized(ui, "Acceleration (km/s^2)", 14.);
                        ui.monospace(format!("Ax:    {:+.4e}", body.computed_a.x));
                        ui.monospace(format!("Ay:    {:+.4e}", body.computed_a.y));
                    });
                });

            // TODO cursor icon when clicking
            if response.1.inner.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
            }

            if response.1.inner.clicked() {
                if let Some(mut col_state) =
                    egui::collapsing_header::CollapsingState::load(ui.ctx(), id)
                {
                    col_state.toggle(ui);
                    col_state.store(ui.ctx());
                }
            }

            ui.add_space(20.);
        }
        app.set_velocities();

        if ui.button("Start").clicked() {
            app.start();
        }

        ui.add_space(10.);
        ui.label(RichText::new("Analysis").heading());
        let t = app.t();
        let days = t / (60 * 60 * 24) as f32;
        ui.monospace(format!("t: {:.4e} s, {:.2} d", t, days));
        ui.monospace("Energy (MJ)");
        ui.monospace(format!("Kinetic:    {:+.4e}", kinetic));
        ui.monospace(format!("Potential:  {:+.4e}", potential));
        ui.monospace(format!("Total:      {:+.4e}", kinetic + potential));
        ui.monospace(format!("Initial:    {:+.4e}", app.analysis.initial_e));
        ui.monospace(format!("Diff:        {:.2}%", diff_percent));
        ui.monospace(format!(
            "Diff per t:  {:.2e}%",
            (100. - diff_percent) / t as f64
        ));
    });
}

fn text_sized(ui: &mut egui::Ui, text: &str, size: f32) {
    ui.monospace(RichText::new(text).size(size));
}

pub fn controls_panel(app: &mut Orbital, ctx: &egui::Context) {
    let available_rect = ctx.screen_rect();
    let x = available_rect.left() + app.ui_state.panel_width;
    let y = available_rect.top();

    let panel_color = ctx.style().visuals.panel_fill;
    let panel_stroke = ctx.style().visuals.window_stroke;

    egui::Area::new("right-side-panel".into())
        .fixed_pos([x, y])
        .default_size([200., 200.])
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(panel_color)
                .inner_margin(6.)
                .stroke(panel_stroke)
                .show(ui, |ui| {
                    ui.label(RichText::new("Controls").heading());

                    text_sized(ui, "drag and scroll with mouse", 12.);
                    text_sized(ui, &format!("{}: start", '\u{21B5}'), 14.);
                    text_sized(ui, &format!("{}: progress forwards", '\u{2192}'), 14.);
                    text_sized(ui, "a: reset", 14.);
                });
        });
}
