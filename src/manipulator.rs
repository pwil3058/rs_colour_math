// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::chroma::HueData;
use crate::{chroma, rgb::RGB, ColourComponent, ColourInterface};

pub struct RGBManipulator<F: ColourComponent> {
    rgb: RGB<F>,
    hue_data: Option<(F, [usize; 3])>,
    sum: F,
    chroma: F,
}

impl<F: ColourComponent> RGBManipulator<F> {
    pub fn new() -> Self {
        Self {
            rgb: [F::ZERO, F::ZERO, F::ZERO].into(),
            hue_data: None,
            sum: F::ZERO,
            chroma: F::ZERO,
        }
    }

    pub fn set_rgb(&mut self, rgb: RGB<F>) {
        self.rgb = rgb;
        self.sum = rgb.sum();
        let xy = rgb.xy();
        if xy.0 == F::ZERO && xy.1 == F::ZERO {
            self.chroma = F::ZERO;
            self.hue_data = None;
        } else {
            let second = chroma::calc_other_from_xy_alt(xy);
            let io = rgb.indices_value_order();
            self.chroma = (xy.0.hypot(xy.1) * chroma::calc_chroma_correction(second)).min(F::ONE);
            self.hue_data = Some((second, io));
        }
    }

    pub fn decr_chroma(&mut self, delta: F) -> bool {
        debug_assert!(delta.is_proportion());
        if self.chroma == F::ZERO {
            false
        } else {
            let cur_chroma = self.chroma;
            let new_chroma = (cur_chroma - delta).max(F::ZERO);
            let (second, io) = self.hue_data.expect("chroma is non zero");
            let hue_data = HueData {
                second: second,
                io: [io[0] as u8, io[1] as u8, io[2] as u8],
            };
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
            let (second, io) = if let Some((second, io)) = self.hue_data {
                (second, io)
            } else {
                // Set the hue data to an arbitrary value
                self.hue_data = Some((F::ZERO, [0, 1, 2]));
                self.hue_data.expect("we just set it to some")
            };
            let hue_data = HueData {
                second: second,
                io: [io[0] as u8, io[1] as u8, io[2] as u8],
            };
            if let Some(rgb) = hue_data.rgb_for_sum_and_chroma(self.sum, new_chroma) {
                self.rgb = rgb;
            } else {
                let hue_data = HueData {
                    second: second,
                    io: [io[0] as u8, io[1] as u8, io[2] as u8],
                };
                self.rgb = hue_data.max_chroma_rgb_for_sum(self.sum);
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
            if let Some((second, io)) = self.hue_data {
                let hue_data = HueData {
                    second: second,
                    io: [io[0] as u8, io[1] as u8, io[2] as u8],
                };
                if let Some(rgb) = hue_data.rgb_for_sum_and_chroma(new_sum, self.chroma) {
                    self.rgb = rgb
                } else {
                    let hue_data = HueData {
                        second: second,
                        io: [io[0] as u8, io[1] as u8, io[2] as u8],
                    };
                    self.rgb = hue_data.min_sum_rgb_for_chroma(self.chroma);
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
            if let Some((second, io)) = self.hue_data {
                let hue_data = HueData {
                    second: second,
                    io: [io[0] as u8, io[1] as u8, io[2] as u8],
                };
                if let Some(rgb) = hue_data.rgb_for_sum_and_chroma(new_sum, self.chroma) {
                    self.rgb = rgb
                } else {
                    let hue_data = HueData {
                        second: second,
                        io: [io[0] as u8, io[1] as u8, io[2] as u8],
                    };
                    self.rgb = hue_data.max_sum_rgb_for_chroma(self.chroma);
                };
            } else {
                let new_value = new_sum / F::THREE;
                self.rgb = [new_value, new_value, new_value].into();
            }
            self.sum = self.rgb.sum();
            cur_sum != self.sum
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ColourInterface;
    use float_plus::*;

    #[test]
    fn decr_chroma() {
        let mut manipulator = super::RGBManipulator::<f64>::new();
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
        let mut manipulator = super::RGBManipulator::<f64>::new();
        assert!(!manipulator.incr_chroma(0.1));
        manipulator.set_rgb([0.75, 0.5, 0.75].into());
        let saved_hue_data = manipulator.hue_data;
        let incr = 0.1;
        let mut expected = (manipulator.chroma + incr).min(crate::chroma::max_chroma_for_sum(
            saved_hue_data.unwrap().0,
            manipulator.sum,
        ));
        while manipulator.incr_chroma(incr) {
            assert_approx_eq!(manipulator.chroma, expected, 0.00000000001);
            expected = (manipulator.chroma + incr).min(crate::chroma::max_chroma_for_sum(
                saved_hue_data.unwrap().0,
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
        let mut manipulator = super::RGBManipulator::<f64>::new();
        manipulator.set_rgb(crate::rgb::RGB::CYAN);
        while manipulator.decr_chroma(0.01) {}
        assert!(manipulator.rgb.is_grey());
        while manipulator.incr_chroma(0.01) {}
        assert_eq!(manipulator.rgb, crate::rgb::RGB::CYAN);
    }

    #[test]
    fn incr_decr_sum() {
        let mut manipulator = super::RGBManipulator::<f64>::new();
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
}
