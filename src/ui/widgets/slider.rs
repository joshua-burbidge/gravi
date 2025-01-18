use std::ops::RangeInclusive;

use egui::emath;

pub struct CustomSlider<'a, Num: emath::Numeric> {
    value: &'a mut Num,
    range: RangeInclusive<Num>,
    label: Option<String>,
    full_width: bool,
}

impl<'a, Num: emath::Numeric> CustomSlider<'a, Num> {
    pub fn new(value: &'a mut Num, range: RangeInclusive<Num>) -> Self {
        Self {
            value,
            range,
            label: None,
            full_width: true,
        }
    }

    pub fn label(mut self, str: impl ToString) -> Self {
        self.label = Some(str.to_string());
        self
    }

    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }
}

// fn custom_formatter(ui: egui::Ui, num: f64, range: RangeInclusive<usize>) -> String {
//     if num < 1000. {
//         ui.style().number_formatter.format(num, range)
//     } else {

//     }
// }

impl<'a, Num: emath::Numeric> egui::Widget for CustomSlider<'a, Num> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let mut response = None;

        ui.horizontal(|ui| {
            match self.label {
                Some(label) => {
                    ui.label(label);
                }
                None => {}
            };

            // scope to change width of this slider only
            ui.scope(|ui| {
                if self.full_width {
                    ui.style_mut().spacing.slider_width = ui.available_width() - 70.0;
                }

                let slider =
                    egui::Slider::new(self.value, self.range).clamping(egui::SliderClamping::Never);

                response = Some(ui.add(slider));
            })
        });

        response.unwrap()
    }
}
