use std::ops::RangeInclusive;

use egui::emath;

use super::CustomSlider;

pub struct XYInput<'a, Num: emath::Numeric> {
    x: &'a mut Num,
    y: &'a mut Num,
    x_range: RangeInclusive<Num>,
    y_range: RangeInclusive<Num>,
    full_width: bool,
}

impl<'a, Num: emath::Numeric> XYInput<'a, Num> {
    pub fn new(
        x: &'a mut Num,
        y: &'a mut Num,
        x_range: RangeInclusive<Num>,
        y_range: RangeInclusive<Num>,
    ) -> Self {
        Self {
            x,
            y,
            x_range,
            y_range,
            full_width: true,
        }
    }

    #[allow(dead_code)]
    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }
}

impl<'a, Num: emath::Numeric> egui::Widget for XYInput<'a, Num> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let x_slider = CustomSlider::new(self.x, self.x_range)
            .label("X: ")
            .full_width(self.full_width);
        let y_slider = CustomSlider::new(self.y, self.y_range)
            .label("Y: ")
            .full_width(self.full_width);

        ui.add(x_slider);
        ui.add(y_slider)
    }
}
