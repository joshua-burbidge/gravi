use std::ops::RangeInclusive;

pub struct CustomSlider<'a> {
    value: &'a mut f32,
    range: RangeInclusive<f32>,
    label: Option<String>,
    full_width: bool,
}

impl<'a> CustomSlider<'a> {
    pub fn new(value: &'a mut f32, range: RangeInclusive<f32>) -> Self {
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

impl<'a> egui::Widget for CustomSlider<'a> {
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
                    ui.style_mut().spacing.slider_width = ui.available_width() - 50.0;
                }

                let slider = egui::Slider::new(self.value, self.range);

                response = Some(ui.add(slider));
            })
        });

        response.unwrap()
    }
}
