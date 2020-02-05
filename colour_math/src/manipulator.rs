// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::convert::TryFrom;

use crate::{chroma::HueData, rgb::RGB, ColourComponent, ColourInterface};
use normalised_angles::Degrees;

pub struct RGBManipulator<F: ColourComponent> {
    rgb: RGB<F>,
    hue_data: Option<HueData<F>>,
    sum: F,
    chroma: F,
    clamped: bool,
}

impl<F: ColourComponent> RGBManipulator<F> {
    pub fn new(clamped: bool) -> Self {
        Self {
            rgb: [F::ZERO, F::ZERO, F::ZERO].into(),
            hue_data: None,
            sum: F::ZERO,
            chroma: F::ZERO,
            clamped,
        }
    }

    pub fn rgb(&self) -> RGB<F> {
        self.rgb
    }

    pub fn set_rgb(&mut self, rgb: RGB<F>) {
        self.rgb = rgb;
        self.sum = rgb.sum();
        let xy = rgb.xy();
        if let Ok(hue_data) = HueData::try_from(xy) {
            self.chroma = (xy.0.hypot(xy.1) * hue_data.chroma_correction()).min(F::ONE);
            self.hue_data = Some(hue_data);
        } else {
            self.chroma = F::ZERO;
            self.hue_data = None;
        }
    }

    pub fn decr_chroma(&mut self, delta: F) -> bool {
        debug_assert!(delta.is_proportion());
        if self.chroma == F::ZERO {
            false
        } else {
            let cur_chroma = self.chroma;
            let new_chroma = (cur_chroma - delta).max(F::ZERO);
            let hue_data = self.hue_data.expect("chroma is non zero");
            self.rgb = hue_data
                .rgb_for_sum_and_chroma(self.sum, new_chroma)
                .expect("smaller chroma always possible");
            self.chroma = self.rgb.chroma();
            cur_chroma != self.chroma
        }
    }

    pub fn incr_chroma(&mut self, delta: F) -> bool {
        debug_assert!(delta.is_proportion());
        if self.chroma == F::ONE {
            false
        } else {
            let cur_chroma = self.chroma;
            let new_chroma = (cur_chroma + delta).min(F::ONE);
            let hue_data = if let Some(hue_data) = self.hue_data {
                hue_data
            } else {
                // Set the hue data to an arbitrary value
                self.hue_data = Some(HueData::default());
                self.hue_data.expect("we just set it to some")
            };
            if let Some(rgb) = hue_data.rgb_for_sum_and_chroma(self.sum, new_chroma) {
                self.rgb = rgb;
            } else if self.clamped {
                self.rgb = hue_data.max_chroma_rgb_for_sum(self.sum);
            } else {
                let min_rgb = hue_data.min_sum_rgb_for_chroma(new_chroma);
                let max_rgb = hue_data.max_sum_rgb_for_chroma(new_chroma);
                self.rgb = if self.sum < min_rgb.sum() {
                    min_rgb
                } else if self.sum > max_rgb.sum() {
                    max_rgb
                } else {
                    hue_data.max_chroma_rgb_for_sum(self.sum)
                };
            };
            self.chroma = self.rgb.chroma();
            cur_chroma != self.chroma
        }
    }

    pub fn decr_value(&mut self, delta: F) -> bool {
        debug_assert!(delta.is_proportion());
        if self.sum == F::ZERO {
            false
        } else {
            let cur_sum = self.sum;
            let new_sum = (cur_sum - F::THREE * delta).max(F::ZERO);
            if let Some(hue_data) = self.hue_data {
                if let Some(rgb) = hue_data.rgb_for_sum_and_chroma(new_sum, self.chroma) {
                    self.rgb = rgb
                } else if self.clamped {
                    self.rgb = hue_data.min_sum_rgb_for_chroma(self.chroma);
                } else {
                    self.rgb = hue_data.max_chroma_rgb_for_sum(new_sum);
                };
            } else {
                let new_value = new_sum / F::THREE;
                self.rgb = [new_value, new_value, new_value].into();
            }
            self.sum = self.rgb.sum();
            cur_sum != self.sum
        }
    }

    pub fn incr_value(&mut self, delta: F) -> bool {
        debug_assert!(delta.is_proportion());
        if self.sum == F::THREE {
            false
        } else {
            let cur_sum = self.sum;
            let new_sum = (cur_sum + F::THREE * delta).min(F::THREE);
            if let Some(hue_data) = self.hue_data {
                if let Some(rgb) = hue_data.rgb_for_sum_and_chroma(new_sum, self.chroma) {
                    self.rgb = rgb
                } else if self.clamped {
                    self.rgb = hue_data.max_sum_rgb_for_chroma(self.chroma);
                } else {
                    self.rgb = hue_data.max_chroma_rgb_for_sum(new_sum);
                };
            } else {
                let new_value = new_sum / F::THREE;
                self.rgb = [new_value, new_value, new_value].into();
            }
            self.sum = self.rgb.sum();
            cur_sum != self.sum
        }
    }

    pub fn rotate(&mut self, angle: Degrees<F>) -> bool {
        if let Some(hue_data) = self.hue_data {
            let hue_angle = hue_data.hue_angle();
            let new_angle = hue_angle + angle;
            let new_hue_data = HueData::<F>::from(new_angle);
            if hue_data == new_hue_data {
                false
            } else {
                if let Some(rgb) = new_hue_data.rgb_for_sum_and_chroma(self.sum, self.chroma) {
                    self.rgb = rgb
                } else {
                    // TODO: make secondary effects of manipulation optional
                    self.rgb = new_hue_data.max_chroma_rgb_for_sum(self.sum);
                };
                self.sum = self.rgb.sum();
                let xy = self.rgb.xy();
                self.chroma = (xy.0.hypot(xy.1) * new_hue_data.chroma_correction()).min(F::ONE);
                self.hue_data = Some(new_hue_data);
                true
            }
        } else {
            false
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ColourInterface;
    use float_plus::*;

    #[test]
    fn decr_chroma() {
        let mut manipulator = super::RGBManipulator::<f64>::new(true);
        assert!(!manipulator.decr_chroma(0.1));
        manipulator.set_rgb(crate::rgb::RGB::YELLOW);
        assert_eq!(manipulator.chroma, 1.0);
        let saved_hue_data = manipulator.hue_data;
        let decr = 0.1;
        let mut expected = (manipulator.chroma - decr).max(0.0);
        while manipulator.decr_chroma(decr) {
            assert_approx_eq!(manipulator.chroma, expected, 0.00000000001);
            expected = (manipulator.chroma - decr).max(0.0);
            assert_eq!(manipulator.sum, 2.0);
            assert_eq!(manipulator.hue_data, saved_hue_data);
        }
        assert!(manipulator.rgb.is_grey());
        assert_eq!(manipulator.chroma, 0.0);
        assert_eq!(manipulator.sum, 2.0);
        assert_eq!(manipulator.hue_data, saved_hue_data);
    }

    #[test]
    fn incr_chroma() {
        let mut manipulator = super::RGBManipulator::<f64>::new(true);
        assert!(!manipulator.incr_chroma(0.1));
        manipulator.set_rgb([0.75, 0.5, 0.75].into());
        let saved_hue_data = manipulator.hue_data;
        let incr = 0.1;
        let mut expected = (manipulator.chroma + incr).min(crate::chroma::max_chroma_for_sum(
            saved_hue_data.unwrap().second,
            manipulator.sum,
        ));
        while manipulator.incr_chroma(incr) {
            assert_approx_eq!(manipulator.chroma, expected, 0.00000000001);
            expected = (manipulator.chroma + incr).min(crate::chroma::max_chroma_for_sum(
                saved_hue_data.unwrap().second,
                manipulator.sum,
            ));
            assert_eq!(manipulator.sum, 2.0);
            assert_eq!(manipulator.hue_data, saved_hue_data);
        }
        assert!(!manipulator.rgb.is_grey());
        assert_eq!(manipulator.chroma, 1.0);
        assert_eq!(manipulator.sum, 2.0);
        assert_eq!(manipulator.hue_data, saved_hue_data);
    }

    #[test]
    fn round_trip_chroma() {
        let mut manipulator = super::RGBManipulator::<f64>::new(true);
        manipulator.set_rgb(crate::rgb::RGB::CYAN);
        while manipulator.decr_chroma(0.01) {}
        assert!(manipulator.rgb.is_grey());
        while manipulator.incr_chroma(0.01) {}
        assert_eq!(manipulator.rgb, crate::rgb::RGB::CYAN);
    }

    #[test]
    fn incr_decr_sum() {
        let mut manipulator = super::RGBManipulator::<f64>::new(true);
        assert!(!manipulator.decr_value(0.1));
        while manipulator.incr_value(0.1) {}
        assert_eq!(manipulator.rgb, crate::rgb::RGB::WHITE);
        while manipulator.decr_value(0.1) {}
        assert_eq!(manipulator.rgb, crate::rgb::RGB::BLACK);
        manipulator.set_rgb(crate::rgb::RGB::YELLOW);
        assert!(!manipulator.decr_value(0.1));
        assert!(!manipulator.incr_value(0.1));
        let cur_value = manipulator.rgb.value();
        manipulator.decr_chroma(0.5);
        assert_eq!(cur_value, manipulator.rgb.value());
        while manipulator.decr_value(0.1) {}
        assert_approx_eq!(manipulator.rgb, [0.5, 0.5, 0.0].into());
        while manipulator.incr_value(0.1) {}
        assert_approx_eq!(manipulator.rgb, [1.0, 1.0, 0.5].into());
    }

    #[test]
    fn rotate_rgb() {
        let mut rgb_manipulator = super::RGBManipulator::<f64>::new(true);
        for delta in [
            -180.0, -120.0, -60.0, -30.0, -10.0, -5.0, 5.0, 10.0, 30.0, 60.0, 120.0, 180.0,
        ]
        .iter()
        {
            assert!(!rgb_manipulator.rotate((*delta).into()));
        }
        // pure colours
        for rgb in crate::rgb::RGB::<f64>::PRIMARIES
            .iter()
            .chain(crate::rgb::RGB::SECONDARIES.iter())
        {
            rgb_manipulator.set_rgb(*rgb);
            for delta in [
                -180.0, -120.0, -60.0, -30.0, -10.0, -5.0, 5.0, 10.0, 30.0, 60.0, 120.0, 180.0,
            ]
            .iter()
            {
                //let cur_chroma = rgb_manipulator.chroma;
                let cur_sum = rgb_manipulator.sum;
                let cur_angle = rgb_manipulator.hue_data.unwrap().hue_angle();
                assert!(rgb_manipulator.rotate((*delta).into()));
                //assert_approx_eq!(cur_chroma, rgb_manipulator.chroma);
                assert_approx_eq!(cur_sum, rgb_manipulator.sum);
                let expected_angle = cur_angle + (*delta).into();
                assert_approx_eq!(
                    expected_angle,
                    rgb_manipulator.hue_data.unwrap().hue_angle(),
                    0.000000000000001
                );
            }
        }
        // shades and tints
        for array in [
            [0.5_f64, 0.0, 0.0],
            [0.0, 0.5, 0.0],
            [0.0, 0.0, 0.5],
            [0.5, 0.5, 0.0],
            [0.5, 0.0, 0.5],
            [0.0, 0.5, 0.5],
            [1.0, 0.5, 0.5],
            [0.5, 1.0, 0.5],
            [0.5, 0.5, 1.0],
            [1.0, 1.0, 0.5],
            [1.0, 0.5, 1.0],
            [0.5, 1.0, 1.0],
        ]
        .iter()
        {
            rgb_manipulator.set_rgb((*array).into());
            for delta in [
                -180.0, -120.0, -60.0, -30.0, -10.0, -5.0, 5.0, 10.0, 30.0, 60.0, 120.0, 180.0,
            ]
            .iter()
            {
                //let cur_chroma = rgb_manipulator.chroma;
                let cur_sum = rgb_manipulator.sum;
                let cur_angle = rgb_manipulator.hue_data.unwrap().hue_angle();
                assert!(rgb_manipulator.rotate((*delta).into()));
                //assert_approx_eq!(cur_chroma, rgb_manipulator.chroma, 0.000000000000001);
                assert_approx_eq!(cur_sum, rgb_manipulator.sum);
                let expected_angle = cur_angle + (*delta).into();
                assert_approx_eq!(
                    expected_angle,
                    rgb_manipulator.hue_data.unwrap().hue_angle(),
                    0.000000000000001
                );
            }
        }
    }
}
