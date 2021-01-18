// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::{chroma::*, hcv::*, rgb::*, ColourComponent, Degrees, HueIfce};

#[derive(Default)]
pub struct ColourManipulator<F: ColourComponent + ChromaTolerance> {
    hcv: HCV<F>,
    //clamped: bool,
    saved_angle: Option<Degrees<F>>,
}

impl<F: ColourComponent + ChromaTolerance> ColourManipulator<F> {
    pub fn rgb(&self) -> RGB<F> {
        (&self.hcv).into()
    }

    pub fn set_hcv(&mut self, hcv: &HCV<F>) {
        self.hcv = *hcv;
        if let Some(hue_data) = self.hcv.hue_data() {
            self.saved_angle = Some(hue_data.hue_angle());
        } else {
            self.saved_angle = None
        }
    }

    pub fn set_rgb(&mut self, rgb: &RGB<F>) {
        self.set_hcv(&rgb.into());
    }

    pub fn rotate(&mut self, angle: Degrees<F>) -> bool {
        if let Some(hue_data) = self.hcv.hue_data {
            let hue_angle = hue_data.hue_angle();
            let new_angle = hue_angle + angle;
            let new_hue_data = HueData::<F>::from(new_angle);
            if hue_data == new_hue_data {
                false
            } else {
                let (min_sum, max_sum) = new_hue_data.sum_range_for_chroma(self.hcv.chroma);
                if self.hcv.sum < min_sum {
                    self.hcv.sum = min_sum;
                } else if self.hcv.sum > max_sum {
                    self.hcv.sum = max_sum;
                }
                self.hcv.hue_data = Some(new_hue_data);
                true
            }
        } else {
            false
        }
    }
}
