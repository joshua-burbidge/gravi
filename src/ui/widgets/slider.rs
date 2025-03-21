use std::ops::RangeInclusive;

use egui::emath;

use crate::app::core::draw::large_number_formatter;

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
    let abs = num.abs();
    if abs <= 100000. {
        default_formatter.format(num, range)
    } else {
        large_number_formatter(num)
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
            ui.scope(|ui: &mut egui::Ui| {
                let available_width = ui.available_width();
                let style = ui.style_mut();

                if self.full_width {
                    style.spacing.slider_width = available_width - 70.0;
                }

                let def_formatter = &style.number_formatter.clone();
                let formatter = |num, decimal_places_range| -> String {
                    custom_formatter(def_formatter, num, decimal_places_range)
                };

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

    static DEFAULT_RANGE: RangeInclusive<usize> = 0..=0;
    fn def_formatter() -> egui::style::NumberFormatter {
        egui::style::NumberFormatter::new(|num, _| num.to_string())
    }

    #[test]
    fn should_use_default_if_num_pos_and_lt_100000() {
        let num = 99999.9_f64;

        let actual = custom_formatter(&def_formatter(), num, DEFAULT_RANGE.clone());

        assert_eq!(actual, num.to_string());
    }

    #[test]
    fn should_use_default_if_num_neg_and_gt_neg_100000() {
        let num = -99999.9_f64;

        let actual = custom_formatter(&def_formatter(), num, DEFAULT_RANGE.clone());

        assert_eq!(actual, num.to_string());
    }

    #[test]
    fn should_use_default_if_num_eq_100000() {
        let num = 100000_f64;

        let actual = custom_formatter(&def_formatter(), num, DEFAULT_RANGE.clone());

        assert_eq!(actual, num.to_string());
    }

    #[test]
    fn should_use_default_if_num_eq_neg_100000() {
        let num = -100000_f64;

        let actual = custom_formatter(&def_formatter(), num, DEFAULT_RANGE.clone());

        assert_eq!(actual, num.to_string());
    }

    #[test]
    fn should_use_e_if_num_gt_100000() {
        let num = 100001_f64;

        let actual = custom_formatter(&def_formatter(), num, DEFAULT_RANGE.clone());

        assert_eq!(actual, "1e5");
    }

    #[test]
    fn should_use_e_if_num_lt_neg_100000() {
        let num = -100001_f64;

        let actual = custom_formatter(&def_formatter(), num, DEFAULT_RANGE.clone());

        assert_eq!(actual, "-1e5");
    }

    #[test]
    fn should_have_3_decimals() {
        let num = 123456_f64;

        let actual = custom_formatter(&def_formatter(), num, DEFAULT_RANGE.clone());

        assert_eq!(actual, "1.235e5");
    }

    #[test]
    fn should_display_big_number() {
        let num = 2394871239847290000000000_f64;

        let actual = custom_formatter(&def_formatter(), num, DEFAULT_RANGE.clone());

        assert_eq!(actual, "2.395e24");
    }

    #[test]
    fn should_display_big_neg_number() {
        let num = -2394871239847290000000000_f64;

        let actual = custom_formatter(&def_formatter(), num, DEFAULT_RANGE.clone());

        assert_eq!(actual, "-2.395e24");
    }
}
