// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::{chroma::*, hcv::*, rgb::*, ColourComponent, Degrees, HueIfce};

pub enum RotationPolicy {
    FavourChroma,
    FavourValue,
}

impl Default for RotationPolicy {
    fn default() -> Self {
        RotationPolicy::FavourChroma
    }
}

#[derive(Default)]
pub struct ColourManipulator<F: ColourComponent + ChromaTolerance> {
    hcv: HCV<F>,
    clamped: bool,
    rotation_policy: RotationPolicy,
    saved_hue_data: Option<HueData<F>>,
}

impl<F: ColourComponent + ChromaTolerance> ColourManipulator<F> {
    pub fn rgb(&self) -> RGB<F> {
        (&self.hcv).into()
    }

    pub fn set_hcv(&mut self, hcv: &HCV<F>) {
        self.hcv = *hcv;
        if let Some(hue_data) = self.hcv.hue_data() {
            self.saved_hue_data = Some(hue_data);
        } else {
            self.saved_hue_data = None
        }
    }

    pub fn set_rgb(&mut self, rgb: &RGB<F>) {
        self.set_hcv(&rgb.into());
    }

    pub fn decr_chroma(&mut self, delta: F) -> bool {
        debug_assert!(delta.is_proportion());
        if self.hcv.chroma == F::ZERO {
            false
        } else {
            let cur_chroma = self.hcv.chroma;
            let new_chroma = (cur_chroma - delta).max(F::ZERO);
            if new_chroma == F::ZERO {
                let hue_data = self.hcv.hue_data.expect("chroma is non zero");
                self.saved_hue_data = Some(hue_data);
                self.hcv.hue_data = None;
            }
            self.hcv.chroma = new_chroma;
            cur_chroma != self.hcv.chroma
        }
    }

    pub fn incr_chroma(&mut self, delta: F) -> bool {
        debug_assert!(delta.is_proportion());
        if self.hcv.chroma == F::ONE || self.hcv.sum == F::ZERO {
            false
        } else {
            let hue_data = if let Some(hue_data) = self.hcv.hue_data {
                hue_data
            } else if let Some(hue_data) = self.saved_hue_data {
                self.hcv.hue_data = Some(hue_data);
                self.hcv.hue_data.expect("we just set it to some")
            } else {
                // Set the hue data to an arbitrary value
                self.hcv.hue_data = Some(HueData::default());
                self.hcv.hue_data.expect("we just set it to some")
            };
            let cur_chroma = self.hcv.chroma;
            self.hcv.chroma = if self.clamped {
                let max_chroma = hue_data.max_chroma_for_sum(self.hcv.sum);
                (cur_chroma + delta).min(max_chroma)
            } else {
                let new_chroma = (cur_chroma + delta).min(F::ONE);
                let (min_sum, max_sum) = hue_data.sum_range_for_chroma(new_chroma);
                if self.hcv.sum < min_sum {
                    self.hcv.sum = min_sum;
                } else if self.hcv.sum > max_sum {
                    self.hcv.sum = max_sum;
                }
                new_chroma
            };
            cur_chroma != self.hcv.chroma
        }
    }

    pub fn rotate(&mut self, angle: Degrees<F>) -> bool {
        if let Some(hue_data) = self.hcv.hue_data {
            let hue_angle = hue_data.hue_angle();
            let new_angle = hue_angle + angle;
            let new_hue_data = HueData::<F>::from(new_angle);
            if hue_data == new_hue_data {
                false
            } else {
                match self.rotation_policy {
                    RotationPolicy::FavourChroma => {
                        let (min_sum, max_sum) = new_hue_data.sum_range_for_chroma(self.hcv.chroma);
                        if self.hcv.sum < min_sum {
                            self.hcv.sum = min_sum;
                        } else if self.hcv.sum > max_sum {
                            self.hcv.sum = max_sum;
                        }
                    }
                    RotationPolicy::FavourValue => {
                        let max_chroma = new_hue_data.max_chroma_for_sum(self.hcv.sum);
                        if self.hcv.chroma > max_chroma {
                            self.hcv.chroma = max_chroma;
                        }
                    }
                }
                self.hcv.hue_data = Some(new_hue_data);
                self.saved_hue_data = Some(new_hue_data);
                true
            }
        } else {
            false
        }
    }
}
