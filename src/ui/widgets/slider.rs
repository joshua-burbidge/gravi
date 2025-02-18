use std::ops::RangeInclusive;

use egui::emath::{self, round_to_decimals};

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

fn custom_formatter(
    default_formatter: &egui::style::NumberFormatter,
    num: f64,
    range: RangeInclusive<usize>,
) -> String {
    if num <= 100000. {
        default_formatter.format(num, range)
    } else {
        let exponent = num.log10().floor() as i32;

        let decimal = num / (10_f64.powi(exponent));

        let rounded = round_to_decimals(decimal, 3);

        String::from(format!("{}e{}", rounded, exponent))
    }
}

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
                let available_width = ui.available_width();
                let style = ui.style_mut();

                if self.full_width {
                    style.spacing.slider_width = available_width - 70.0;
                }

                let def_formatter = &style.number_formatter.clone();

                let formatter =
                    |num, range| -> String { custom_formatter(&def_formatter, num, range) };

                let slider = egui::Slider::new(self.value, self.range)
                    .clamping(egui::SliderClamping::Never)
                    .custom_formatter(formatter);

                response = Some(ui.add(slider));
            })
        });

        response.unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatter() {
        let num = 0_f64;

        let exponent = num.log10().floor() as i32;

        assert_eq!(exponent, 1);
    }
}
