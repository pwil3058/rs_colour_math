// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

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
            self.rgb = chroma::rgb_for_sum_and_chroma(second, self.sum, new_chroma, &io)
                .expect("smaller chroma always possible");
            self.chroma = self.rgb.chroma();
            self.sum = self.rgb.sum();
            cur_chroma != self.chroma
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ColourInterface;
    use float_cmp::*;

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
            assert!(
                approx_eq!(f64, manipulator.chroma, expected, epsilon = 0.00000000001),
                "{} == {}",
                manipulator.chroma,
                expected,
            );
            expected = (manipulator.chroma - decr).max(0.0);
            assert_eq!(manipulator.sum, 2.0);
            assert_eq!(manipulator.hue_data, saved_hue_data);
        }
        assert!(manipulator.rgb.is_grey());
        assert_eq!(manipulator.chroma, 0.0);
    }
}
