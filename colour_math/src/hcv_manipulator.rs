// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::{chroma::*, hcv::*, rgb::*, ColourComponent, Degrees, HueIfce};

#[derive(Clone, Copy)]
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
    saved_hue_data: HueData<F>,
}

impl<F: ColourComponent + ChromaTolerance> ColourManipulator<F> {
    pub fn rgb(&self) -> RGB<F> {
        (&self.hcv).into()
    }

    pub fn set_hcv(&mut self, hcv: &HCV<F>) {
        self.hcv = *hcv;
        if let Some(hue_data) = self.hcv.hue_data() {
            self.saved_hue_data = hue_data;
        } else {
            self.saved_hue_data = HueData::default();
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
                self.saved_hue_data = hue_data;
                self.hcv.hue_data = None;
            }
            self.hcv.chroma = new_chroma;
            cur_chroma != self.hcv.chroma
        }
    }

    fn adjust_sum(&mut self, hue_data: &HueData<F>) {
        let (min_sum, max_sum) = hue_data.sum_range_for_chroma(self.hcv.chroma);
        if self.hcv.sum < min_sum {
            self.hcv.sum = min_sum;
        } else if self.hcv.sum > max_sum {
            self.hcv.sum = max_sum;
        }
    }

    pub fn incr_chroma(&mut self, delta: F) -> bool {
        debug_assert!(delta.is_proportion());
        if self.hcv.chroma == F::ONE {
            false
        } else {
            let cur_chroma = self.hcv.chroma;
            if let Some(hue_data) = self.hcv.hue_data {
                if self.clamped {
                    let max_chroma = hue_data.max_chroma_for_sum(self.hcv.sum);
                    debug_assert!(
                        max_chroma > cur_chroma || max_chroma.approx_eq(&cur_chroma, None)
                    );
                    self.hcv.chroma = (cur_chroma + delta).min(max_chroma);
                } else {
                    self.hcv.chroma = (cur_chroma + delta).min(F::ONE);
                    self.adjust_sum(&hue_data);
                };
            } else {
                let hue_data = self.saved_hue_data;
                if self.clamped {
                    let max_chroma = hue_data.max_chroma_for_sum(self.hcv.sum);
                    debug_assert!(max_chroma >= cur_chroma);
                    self.hcv.chroma = (cur_chroma + delta).min(max_chroma);
                } else {
                    self.hcv.chroma = (cur_chroma + delta).min(F::ONE);
                    self.adjust_sum(&hue_data);
                };
                if self.hcv.chroma > F::ZERO {
                    self.hcv.hue_data = Some(hue_data);
                }
            }
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
                self.saved_hue_data = new_hue_data;
                true
            }
        } else {
            false
        }
    }
}

#[derive(Default)]
pub struct ColourManipulatorBuilder<F>
where
    F: ColourComponent + ChromaTolerance,
{
    init_hcv: Option<HCV<F>>,
    clamped: bool,
    rotation_policy: RotationPolicy,
}

impl<F> ColourManipulatorBuilder<F>
where
    F: ColourComponent + ChromaTolerance,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn init_rgb(&mut self, rgb: &RGB<F>) -> &mut Self {
        self.init_hcv = Some(rgb.into());
        self
    }

    pub fn init_hcv(&mut self, hcv: &HCV<F>) -> &mut Self {
        self.init_hcv = Some(*hcv);
        self
    }

    pub fn clamped(&mut self, clamped: bool) -> &mut Self {
        self.clamped = clamped;
        self
    }

    pub fn rotation_policy(&mut self, rotation_policy: RotationPolicy) -> &mut Self {
        self.rotation_policy = rotation_policy;
        self
    }

    pub fn build(&self) -> ColourManipulator<F> {
        let mut manipulator = ColourManipulator::<F>::default();
        manipulator.clamped = self.clamped;
        manipulator.rotation_policy = self.rotation_policy;
        if let Some(init_hcv) = self.init_hcv {
            manipulator.set_hcv(&init_hcv);
        };
        manipulator
    }
}

#[cfg(test)]
mod hcv_manipulator_tests {
    use super::*;
    use num_traits_plus::{assert_approx_eq, float_plus::*};

    #[test]
    fn set_colour() {
        let mut manipulator = ColourManipulatorBuilder::<f64>::new().clamped(true).build();
        for rgb in &RGB::<f64>::SECONDARIES {
            manipulator.set_rgb(rgb);
            assert_eq!(manipulator.rgb(), *rgb);
        }
    }

    #[test]
    fn decr_chroma() {
        // clamping should make no difference to chroma decrementing
        for clamped in &[true, false] {
            let mut manipulator = ColourManipulatorBuilder::<f64>::new()
                .clamped(*clamped)
                .build();
            assert_eq!(manipulator.hcv, HCV::BLACK);
            assert!(!manipulator.decr_chroma(0.1));
            manipulator.set_rgb(&crate::rgb::RGB::YELLOW);
            assert_eq!(manipulator.hcv.chroma, 1.0);
            let saved_hue_data = manipulator.hcv.hue_data;
            let decr = 0.1;
            let mut expected = (manipulator.hcv.chroma - decr).max(0.0);
            while manipulator.decr_chroma(decr) {
                assert_approx_eq!(manipulator.hcv.chroma, expected, 0.00000000001);
                expected = (manipulator.hcv.chroma - decr).max(0.0);
                assert_eq!(manipulator.hcv.sum, 2.0);
                if manipulator.hcv.chroma > 0.0 {
                    assert_eq!(manipulator.hcv.hue_data, saved_hue_data);
                }
            }
            assert!(manipulator.hcv.is_grey());
            assert_eq!(manipulator.hcv.chroma, 0.0);
            assert_eq!(manipulator.hcv.sum, 2.0);
            assert_eq!(manipulator.hcv.hue_data, None);
            assert_eq!(manipulator.saved_hue_data, saved_hue_data.unwrap());
        }
    }

    #[test]
    fn incr_chroma_clamped() {
        let mut manipulator = super::ColourManipulatorBuilder::<f64>::new()
            .clamped(true)
            .build();
        assert_eq!(manipulator.hcv, HCV::BLACK);
        assert!(!manipulator.incr_chroma(0.1));
        // Test where clamping makes a difference and where it doesn't
        for array in &[[0.75_f64, 0.5, 0.0], [0.75, 0.5, 0.75]] {
            manipulator.set_rgb(&array.into());
            let start_sum = manipulator.hcv.sum;
            let saved_hue_data = manipulator.hcv.hue_data.unwrap();
            let incr = 0.1;
            let mut expected = (manipulator.hcv.chroma + incr)
                .min(saved_hue_data.max_chroma_for_sum(manipulator.hcv.sum));
            while manipulator.incr_chroma(incr) {
                assert_approx_eq!(manipulator.hcv.chroma, expected, 0.00000000001);
                expected = (manipulator.hcv.chroma + incr)
                    .min(saved_hue_data.max_chroma_for_sum(manipulator.hcv.sum));
                assert_eq!(manipulator.hcv.sum, start_sum);
                assert_eq!(manipulator.hcv.hue_data, Some(saved_hue_data));
            }
            assert!(!manipulator.hcv.is_grey());
            assert_eq!(
                manipulator.hcv.chroma,
                saved_hue_data.max_chroma_for_sum(start_sum)
            );
            assert_eq!(manipulator.hcv.sum, start_sum);
            assert_eq!(manipulator.hcv.hue_data, Some(saved_hue_data));
        }
    }

    #[test]
    fn incr_chroma_unclamped() {
        let mut manipulator = super::ColourManipulatorBuilder::<f64>::new()
            .clamped(false)
            .build();
        assert_eq!(manipulator.hcv, HCV::BLACK);
        assert!(manipulator.incr_chroma(0.1));
        // Test where clamping makes a difference and where it doesn't
        for array in &[[0.75_f64, 0.5, 0.0], [0.75, 0.5, 0.75]] {
            manipulator.set_rgb(&array.into());
            let start_sum = manipulator.hcv.sum;
            let saved_hue_data = manipulator.hcv.hue_data.unwrap();
            let incr = 0.1;
            let mut expected = (manipulator.hcv.chroma + incr).min(1.0);
            while manipulator.incr_chroma(incr) {
                assert_approx_eq!(manipulator.hcv.chroma, expected, 0.00000000001);
                expected = (manipulator.hcv.chroma + incr).min(1.0);
                let (min_sum, max_sum) =
                    saved_hue_data.sum_range_for_chroma(manipulator.hcv.chroma);
                assert!(
                    manipulator.hcv.sum == start_sum
                        || manipulator.hcv.sum == min_sum
                        || manipulator.hcv.sum == max_sum
                );
                assert_eq!(manipulator.hcv.hue_data, Some(saved_hue_data));
            }
            assert!(!manipulator.hcv.is_grey());
            assert_eq!(manipulator.hcv.chroma, 1.0);
            let (min_sum, max_sum) = saved_hue_data.sum_range_for_chroma(manipulator.hcv.chroma);
            assert!(
                manipulator.hcv.sum == start_sum
                    || manipulator.hcv.sum == min_sum
                    || manipulator.hcv.sum == max_sum
            );
            assert_eq!(manipulator.hcv.hue_data, Some(saved_hue_data));
        }
    }

    #[test]
    fn round_trip_chroma() {
        let mut manipulator = super::ColourManipulatorBuilder::<f64>::new()
            .clamped(true)
            .build();
        manipulator.set_rgb(&crate::rgb::RGB::CYAN);
        while manipulator.decr_chroma(0.01) {}
        assert!(manipulator.hcv.is_grey());
        while manipulator.incr_chroma(0.01) {}
        assert_eq!(manipulator.rgb(), crate::rgb::RGB::CYAN);
    }

    #[test]
    fn rotate_rgb_favouring_chroma() {
        let mut manipulator = ColourManipulatorBuilder::<f64>::new()
            .rotation_policy(RotationPolicy::FavourChroma)
            .build();
        for delta in [
            -180.0, -120.0, -60.0, -30.0, -10.0, -5.0, 5.0, 10.0, 30.0, 60.0, 120.0, 180.0,
        ]
        .iter()
        {
            assert!(!manipulator.rotate((*delta).into()));
        }
        // pure colours
        for rgb in crate::rgb::RGB::<f64>::PRIMARIES
            .iter()
            .chain(crate::rgb::RGB::SECONDARIES.iter())
        {
            manipulator.set_rgb(rgb);
            for delta in [
                -180.0, -120.0, -60.0, -30.0, -10.0, -5.0, 5.0, 10.0, 30.0, 60.0, 120.0, 180.0,
            ]
            .iter()
            {
                let cur_chroma = manipulator.hcv.chroma;
                let cur_sum = manipulator.hcv.sum;
                let cur_angle = manipulator.hcv.hue_data.unwrap().hue_angle();
                assert!(manipulator.rotate((*delta).into()));
                assert_approx_eq!(cur_chroma, manipulator.hcv.chroma);
                let (min_sum, max_sum) = manipulator
                    .hcv
                    .hue_data
                    .unwrap()
                    .sum_range_for_chroma(cur_chroma);
                assert!(
                    cur_sum.approx_eq(&manipulator.hcv.sum, None)
                        || min_sum.approx_eq(&manipulator.hcv.sum, None)
                        || max_sum.approx_eq(&manipulator.hcv.sum, None)
                );
                let expected_angle = cur_angle + (*delta).into();
                assert_approx_eq!(
                    expected_angle,
                    manipulator.hcv.hue_data.unwrap().hue_angle(),
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
            manipulator.set_rgb(&(*array).into());
            for delta in [
                -180.0, -120.0, -60.0, -30.0, -10.0, -5.0, 5.0, 10.0, 30.0, 60.0, 120.0, 180.0,
            ]
            .iter()
            {
                let cur_chroma = manipulator.hcv.chroma;
                let cur_sum = manipulator.hcv.sum;
                let cur_angle = manipulator.hcv.hue_data.unwrap().hue_angle();
                assert!(manipulator.rotate((*delta).into()));
                assert_approx_eq!(cur_chroma, manipulator.hcv.chroma, 0.000000000000001);
                let (min_sum, max_sum) = manipulator
                    .hcv
                    .hue_data
                    .unwrap()
                    .sum_range_for_chroma(cur_chroma);
                assert!(
                    cur_sum.approx_eq(&manipulator.hcv.sum, None)
                        || min_sum.approx_eq(&manipulator.hcv.sum, None)
                        || max_sum.approx_eq(&manipulator.hcv.sum, None)
                );
                let expected_angle = cur_angle + (*delta).into();
                assert_approx_eq!(
                    expected_angle,
                    manipulator.hcv.hue_data.unwrap().hue_angle(),
                    0.000000000000001
                );
            }
        }
    }

    #[test]
    fn rotate_rgb_favouring_value() {
        let mut manipulator = ColourManipulatorBuilder::<f64>::new()
            .rotation_policy(RotationPolicy::FavourValue)
            .build();
        for delta in [
            -180.0, -120.0, -60.0, -30.0, -10.0, -5.0, 5.0, 10.0, 30.0, 60.0, 120.0, 180.0,
        ]
        .iter()
        {
            assert!(!manipulator.rotate((*delta).into()));
        }
        // pure colours
        for rgb in crate::rgb::RGB::<f64>::PRIMARIES
            .iter()
            .chain(crate::rgb::RGB::SECONDARIES.iter())
        {
            manipulator.set_rgb(rgb);
            for delta in [
                -180.0, -120.0, -60.0, -30.0, -10.0, -5.0, 5.0, 10.0, 30.0, 60.0, 120.0, 180.0,
            ]
            .iter()
            {
                let cur_chroma = manipulator.hcv.chroma;
                let cur_sum = manipulator.hcv.sum;
                let cur_angle = manipulator.hcv.hue_data.unwrap().hue_angle();
                assert!(manipulator.rotate((*delta).into()));
                let max_chroma = manipulator
                    .hcv
                    .hue_data
                    .unwrap()
                    .max_chroma_for_sum(manipulator.hcv.sum);
                assert!(
                    cur_chroma.approx_eq(&manipulator.hcv.chroma, Some(0.000000000000001))
                        || max_chroma.approx_eq(&manipulator.hcv.chroma, Some(0.000000000000001))
                );
                assert_approx_eq!(cur_sum, manipulator.hcv.sum);
                let expected_angle = cur_angle + (*delta).into();
                assert_approx_eq!(
                    expected_angle,
                    manipulator.hcv.hue_data.unwrap().hue_angle(),
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
            manipulator.set_rgb(&(*array).into());
            for delta in [
                -180.0, -120.0, -60.0, -30.0, -10.0, -5.0, 5.0, 10.0, 30.0, 60.0, 120.0, 180.0,
            ]
            .iter()
            {
                let cur_chroma = manipulator.hcv.chroma;
                let cur_sum = manipulator.hcv.sum;
                let cur_angle = manipulator.hcv.hue_data.unwrap().hue_angle();
                assert!(manipulator.rotate((*delta).into()));
                let max_chroma = manipulator
                    .hcv
                    .hue_data
                    .unwrap()
                    .max_chroma_for_sum(manipulator.hcv.sum);
                assert!(
                    cur_chroma.approx_eq(&manipulator.hcv.chroma, Some(0.000000000000001))
                        || max_chroma.approx_eq(&manipulator.hcv.chroma, Some(0.000000000000001))
                );
                assert_approx_eq!(cur_sum, manipulator.hcv.sum);
                let expected_angle = cur_angle + (*delta).into();
                assert_approx_eq!(
                    expected_angle,
                    manipulator.hcv.hue_data.unwrap().hue_angle(),
                    0.000000000000001
                );
            }
        }
    }
}
