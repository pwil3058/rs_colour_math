// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    cmp::{Ordering, PartialEq, PartialOrd},
    convert::From,
    hash::{Hash, Hasher},
    ops::{Add, Sub},
};

use float_plus::*;
use normalised_angles::*;

use crate::rgb::RGB;
use crate::{chroma, ColourComponent, ColourInterface};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Hue<F: ColourComponent> {
    angle: Degrees<F>,
    max_chroma_rgb: RGB<F>,
    chroma_correction: F,
}

impl<F: ColourComponent> From<Degrees<F>> for Hue<F> {
    fn from(angle: Degrees<F>) -> Self {
        if angle.is_nan() {
            Self {
                angle,
                max_chroma_rgb: RGB::<F>::WHITE,
                chroma_correction: F::ONE,
            }
        } else {
            let other = chroma::calc_other_from_angle(angle.abs());
            let max_chroma_rgb: RGB<F> = if angle.degrees() >= F::RED_ANGLE {
                if angle.degrees() <= F::YELLOW_ANGLE {
                    [F::ONE, other, F::ZERO].into()
                } else if angle.degrees() <= F::GREEN_ANGLE {
                    [other, F::ONE, F::ZERO].into()
                } else {
                    [F::ZERO, F::ONE, other].into()
                }
            } else {
                if angle.degrees() >= F::MAGENTA_ANGLE {
                    [F::ONE, F::ZERO, other].into()
                } else if angle.degrees() >= F::BLUE_ANGLE {
                    [other, F::ZERO, F::ONE].into()
                } else {
                    [F::ZERO, other, F::ONE].into()
                }
            };
            let chroma_correction = chroma::calc_chroma_correction(other);
            Self {
                angle,
                max_chroma_rgb,
                chroma_correction,
            }
        }
    }
}

impl<F: ColourComponent> From<RGB<F>> for Hue<F> {
    fn from(rgb: RGB<F>) -> Self {
        let (x, y) = rgb.xy();
        let angle: Degrees<F> = Degrees::atan2(x, y);
        if angle.is_nan() {
            // NB: float limitations make using ::from(angle) unwise for non NAN angles
            return Hue::from(angle);
        }
        let io = rgb.indices_value_order();
        let mut parts: [F; 3] = [F::ZERO, F::ZERO, F::ZERO];
        parts[io[0]] = F::ONE;
        if rgb[io[0]] == rgb[io[1]] {
            // Secondary colour
            parts[io[1]] = F::ONE;
        } else if rgb[io[1]] != rgb[io[2]] {
            // Not Primary or Secondary
            parts[io[1]] = chroma::calc_other_from_angle(angle.abs());
        }
        let max_chroma_rgb: RGB<F> = parts.into();
        let chroma_correction = chroma::calc_chroma_correction(max_chroma_rgb[io[1]]);
        Self {
            angle,
            max_chroma_rgb,
            chroma_correction,
        }
    }
}

impl<F: ColourComponent> Hash for Hue<F>
where
    Degrees<F>: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.angle.hash(state)
    }
}

impl<F: ColourComponent> PartialEq for Hue<F> {
    fn eq(&self, other: &Hue<F>) -> bool {
        self.angle.eq(&other.angle)
    }
}

impl<F: ColourComponent> PartialOrd for Hue<F> {
    fn partial_cmp(&self, other: &Hue<F>) -> Option<Ordering> {
        self.angle.partial_cmp(&other.angle)
    }
}

impl<F: ColourComponent> Add<Degrees<F>> for Hue<F> {
    type Output = Self;

    fn add(self, angle: Degrees<F>) -> Self {
        (self.angle + angle).into()
    }
}

impl<F: ColourComponent> Sub<Degrees<F>> for Hue<F> {
    type Output = Self;

    fn sub(self, angle: Degrees<F>) -> Self {
        (self.angle - angle).into()
    }
}

impl<F: ColourComponent> Sub<Hue<F>> for Hue<F> {
    type Output = Degrees<F>;

    fn sub(self, other: Hue<F>) -> Degrees<F> {
        self.angle - other.angle
    }
}
impl<F: ColourComponent> Hue<F> {
    /// Returns `true` if this `Hue` is grey i.e. completely devoid of colour/chroma/hue
    pub fn is_grey(&self) -> bool {
        self.angle.is_nan()
    }

    pub fn angle(&self) -> Degrees<F> {
        self.angle
    }

    /// Returns an `RGB<F>` representing the colour with this hue, the maximum achievable chroma
    /// and the highest achievable value.
    pub fn max_chroma_rgb(&self) -> RGB<F> {
        self.max_chroma_rgb
    }

    pub fn chroma_correction(&self) -> F {
        self.chroma_correction
    }

    /// Returns the maximum chroma that can be achieved for this view and the given `value`.
    /// 'value` must be in the range 0.0 to 1.0 inclusive
    pub fn max_chroma_for_value(&self, value: F) -> F {
        debug_assert!(value.is_proportion());
        if self.is_grey() {
            F::ZERO
        } else {
            // NB using sum() rather than value() for numeric accuracy
            let mcv = self.max_chroma_rgb.sum();
            let val = value * F::THREE;
            // NB these will be safe because mcv will be between 1.0 and 2.0
            if mcv > val {
                val / mcv
            } else if mcv < val {
                (F::THREE - val) / (F::THREE - mcv)
            } else {
                F::ONE
            }
        }
    }

    /// Returns the range of `RGB` that can be created with this hue and the given `chroma`
    /// returns `None` if no such range exists. `chroma` must be in range 0.0 to 1.0 inclusive.
    pub fn rgb_range_with_chroma(&self, chroma: F) -> Option<(RGB<F>, RGB<F>)> {
        debug_assert!(chroma.is_proportion());
        if chroma == F::ZERO {
            Some((RGB::BLACK, RGB::WHITE))
        } else if self.is_grey() {
            None
        } else if chroma == F::ONE {
            Some((self.max_chroma_rgb, self.max_chroma_rgb))
        } else {
            let shade: [F; 3] = [
                self.max_chroma_rgb[0] * chroma,
                self.max_chroma_rgb[1] * chroma,
                self.max_chroma_rgb[2] * chroma,
            ];
            let delta = F::ONE - chroma;
            let tint: [F; 3] = [shade[0] + delta, shade[1] + delta, shade[2] + delta];
            Some((shade.into(), tint.into()))
        }
    }

    /// Returns the range of `values` for which it is possible to construct an `RGB` with this hue
    /// and the specified `chroma`.
    pub fn value_range_with_chroma(&self, chroma: F) -> Option<(F, F)> {
        debug_assert!(chroma.is_proportion());
        if chroma == F::ZERO {
            Some((F::ZERO, F::ONE))
        } else if self.is_grey() {
            None
        } else if chroma == F::ONE {
            let val = self.max_chroma_rgb.value();
            Some((val, val))
        } else {
            // NB using sum() rather than value() for numeric accuracy
            let shade = self.max_chroma_rgb.sum() * chroma / F::THREE;
            let tint =
                (F::THREE + self.max_chroma_rgb.sum() * chroma - chroma * F::THREE) / F::THREE;
            Some((shade, tint))
        }
    }

    /// Returns a `RGB` with the specified `chroma` and `value` if feasible and `None` otherwise.
    pub fn rgb_with_chroma_and_value(&self, chroma: F, value: F) -> Option<RGB<F>> {
        debug_assert!(chroma.is_proportion(), "{:?}", chroma);
        debug_assert!(value.is_proportion(), "{:?}", value);
        if let Some((min_value, max_value)) = self.value_range_with_chroma(chroma) {
            if value < min_value || value > max_value {
                None
            } else {
                let delta = value - min_value;
                let rgb: RGB<F> = [
                    (self.max_chroma_rgb[0] * chroma + delta).min(F::ONE),
                    (self.max_chroma_rgb[1] * chroma + delta).min(F::ONE),
                    (self.max_chroma_rgb[2] * chroma + delta).min(F::ONE),
                ]
                .into();
                // NB: because floats only approximate reals trying to
                // set chroma too small (but non zero) results in a drift
                // in the hue angle of the resulting RGB. When this
                // happens we go straight to a zero chroma RGB
                let (x, y) = rgb.xy();
                let angle: Degrees<F> = Degrees::atan2(x, y);
                let limit = F::from(0.00000000000001).unwrap();
                if angle.approx_eq(&self.angle, Some(limit), Some(limit)) {
                    Some(rgb)
                } else {
                    Some(RGB::from([value, value, value]))
                }
            }
        } else {
            None
        }
    }

    /// Returns a `RGB` with the maximum feasible chroma for this hue and the given `value`
    pub fn max_chroma_rgb_with_value(&self, value: F) -> RGB<F> {
        debug_assert!(value.is_proportion());
        let mcv = self.max_chroma_rgb.value();
        if mcv > value {
            if value == F::ZERO {
                RGB::BLACK
            } else {
                [
                    self.max_chroma_rgb[0] * value / mcv,
                    self.max_chroma_rgb[1] * value / mcv,
                    self.max_chroma_rgb[2] * value / mcv,
                ]
                .into()
            }
        } else if mcv < value {
            if value == F::ONE {
                RGB::WHITE
            } else {
                let mut array = [F::ONE, F::ONE, F::ONE];
                let io = self.max_chroma_rgb.indices_value_order();
                // it's simpler to work out weakest component first
                let other = self.max_chroma_rgb[io[1]];
                let shortfall = (value - mcv) * F::THREE;
                array[io[2]] = shortfall / (F::TWO - other);
                array[io[1]] = other + shortfall - array[io[2]];
                array.into()
            }
        } else {
            self.max_chroma_rgb
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{I_BLUE, I_GREEN, I_RED};

    const TEST_ANGLES: [f64; 13] = [
        -180.0, -150.0, -120.0, -90.0, -60.0, -30.0, 0.0, 30.0, 60.0, 90.0, 120.0, 150.0, 180.0,
    ];

    const NON_ZERO_TEST_RATIOS: [f64; 10] = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];

    const TEST_RATIOS: [f64; 11] = [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];

    #[test]
    fn from_angle() {
        assert_eq!(
            Hue::<f64>::from(Degrees::from(-150.0))
                .max_chroma_rgb
                .indices_value_order(),
            [I_BLUE, I_GREEN, I_RED]
        );
        assert_eq!(
            Hue::<f64>::from(Degrees::from(-90.0))
                .max_chroma_rgb
                .indices_value_order(),
            [I_BLUE, I_RED, I_GREEN]
        );
        assert_eq!(
            Hue::<f64>::from(Degrees::from(-30.0))
                .max_chroma_rgb
                .indices_value_order(),
            [I_RED, I_BLUE, I_GREEN]
        );
        assert_eq!(
            Hue::<f64>::from(Degrees::from(30.0))
                .max_chroma_rgb
                .indices_value_order(),
            [I_RED, I_GREEN, I_BLUE]
        );
        assert_eq!(
            Hue::<f64>::from(Degrees::from(90.0))
                .max_chroma_rgb
                .indices_value_order(),
            [I_GREEN, I_RED, I_BLUE]
        );
        assert_eq!(
            Hue::<f64>::from(Degrees::from(150.0))
                .max_chroma_rgb
                .indices_value_order(),
            [I_GREEN, I_BLUE, I_RED]
        );
        assert_eq!(
            Hue::<f64>::from(Degrees::<f64>::from(f64::RED_ANGLE)).max_chroma_rgb,
            RGB::<f64>::RED
        );
        assert_eq!(
            Hue::<f64>::from(Degrees::<f64>::from(f64::GREEN_ANGLE)).max_chroma_rgb,
            RGB::<f64>::GREEN
        );
        assert_eq!(
            Hue::<f64>::from(Degrees::<f64>::from(f64::BLUE_ANGLE)).max_chroma_rgb,
            RGB::<f64>::BLUE
        );
        assert_eq!(
            Hue::<f64>::from(Degrees::<f64>::from(f64::CYAN_ANGLE)).max_chroma_rgb,
            RGB::<f64>::CYAN
        );
        assert_eq!(
            Hue::<f64>::from(Degrees::<f64>::from(f64::MAGENTA_ANGLE)).max_chroma_rgb,
            RGB::<f64>::MAGENTA
        );
        assert_eq!(
            Hue::<f64>::from(Degrees::<f64>::from(f64::YELLOW_ANGLE)).max_chroma_rgb,
            RGB::<f64>::YELLOW
        );
    }

    #[test]
    fn chroma_correction_from_angle() {
        assert_eq!(
            Hue::<f64>::from(Degrees::from(f64::RED_ANGLE)).chroma_correction,
            1.0
        );
        assert_eq!(
            Hue::<f64>::from(Degrees::from(f64::GREEN_ANGLE)).chroma_correction,
            1.0
        );
        assert_eq!(
            Hue::<f64>::from(Degrees::from(f64::BLUE_ANGLE)).chroma_correction,
            1.0
        );
        assert_eq!(
            Hue::<f64>::from(Degrees::from(f64::CYAN_ANGLE)).chroma_correction,
            1.0
        );
        assert_eq!(
            Hue::<f64>::from(Degrees::from(f64::MAGENTA_ANGLE)).chroma_correction,
            1.0
        );
        assert_eq!(
            Hue::<f64>::from(Degrees::from(f64::YELLOW_ANGLE)).chroma_correction,
            1.0
        );
        assert_eq!(
            Hue::<f64>::from(Degrees::from(30.0)).chroma_correction,
            2.0 / 3.0_f64.sqrt()
        );
        assert_eq!(
            Hue::<f64>::from(Degrees::from(90.0)).chroma_correction,
            2.0 / 3.0_f64.sqrt()
        );
        assert_eq!(
            Hue::<f64>::from(Degrees::from(150.0)).chroma_correction,
            2.0 / 3.0_f64.sqrt()
        );
        assert_eq!(
            Hue::<f64>::from(Degrees::from(-150.0)).chroma_correction,
            2.0 / 3.0_f64.sqrt()
        );
        assert_eq!(
            Hue::<f64>::from(Degrees::from(-90.0)).chroma_correction,
            2.0 / 3.0_f64.sqrt()
        );
        assert_eq!(
            Hue::<f64>::from(Degrees::from(-30.0)).chroma_correction,
            2.0 / 3.0_f64.sqrt()
        );
    }

    #[test]
    fn from_rgb() {
        assert_approx_eq!(
            Hue::<f64>::from(RGB::<f64>::RED).angle,
            Degrees::<f64>::from(f64::RED_ANGLE)
        );
        assert_approx_eq!(
            Hue::<f64>::from(RGB::<f64>::GREEN).angle,
            Degrees::<f64>::from(f64::GREEN_ANGLE)
        );
        assert_approx_eq!(
            Hue::<f64>::from(RGB::<f64>::BLUE).angle,
            Degrees::<f64>::from(f64::BLUE_ANGLE)
        );
        assert_approx_eq!(
            Hue::<f64>::from(RGB::<f64>::CYAN).angle,
            Degrees::<f64>::from(f64::CYAN_ANGLE)
        );
        assert_approx_eq!(
            Hue::<f64>::from(RGB::<f64>::MAGENTA).angle,
            Degrees::<f64>::from(f64::MAGENTA_ANGLE)
        );
        assert_approx_eq!(
            Hue::<f64>::from(RGB::<f64>::YELLOW).angle,
            Degrees::<f64>::from(f64::YELLOW_ANGLE)
        );
        assert!(Hue::<f64>::from(RGB::<f64>::BLACK).angle.is_nan());
        assert!(Hue::<f64>::from(RGB::<f64>::WHITE).angle.is_nan());
    }

    #[test]
    fn chroma_correction_from_rgb() {
        assert_eq!(Hue::<f64>::from(RGB::RED).chroma_correction, 1.0);
        assert_eq!(Hue::<f64>::from(RGB::GREEN).chroma_correction, 1.0);
        assert_eq!(Hue::<f64>::from(RGB::BLUE).chroma_correction, 1.0);
        assert_eq!(Hue::<f64>::from(RGB::CYAN).chroma_correction, 1.0);
        assert_eq!(Hue::<f64>::from(RGB::MAGENTA).chroma_correction, 1.0);
        assert_eq!(Hue::<f64>::from(RGB::YELLOW).chroma_correction, 1.0);

        assert_approx_eq!(
            Hue::<f64>::from(RGB::from([0.5, 0.25, 0.0])).chroma_correction,
            2.0 / 3.0_f64.sqrt(),
        );
        assert_approx_eq!(
            Hue::<f64>::from(RGB::from([0.25, 0.5, 0.0])).chroma_correction,
            2.0 / 3.0_f64.sqrt(),
        );
        assert_approx_eq!(
            Hue::<f64>::from(RGB::from([0.5, 0.0, 0.25])).chroma_correction,
            2.0 / 3.0_f64.sqrt(),
        );
        assert_approx_eq!(
            Hue::<f64>::from(RGB::from([0.25, 0.0, 0.5])).chroma_correction,
            2.0 / 3.0_f64.sqrt(),
        );
        assert_approx_eq!(
            Hue::<f64>::from(RGB::from([0.0, 0.5, 0.25])).chroma_correction,
            2.0 / 3.0_f64.sqrt(),
        );
        assert_approx_eq!(
            Hue::<f64>::from(RGB::from([0.0, 0.25, 0.5])).chroma_correction,
            2.0 / 3.0_f64.sqrt(),
        );
    }

    #[test]
    fn rotation() {
        assert_approx_eq!(
            (Hue::<f64>::from(Degrees::<f64>::from(f64::YELLOW_ANGLE)) + Degrees::from(60.0)).angle,
            Degrees::<f64>::from(f64::GREEN_ANGLE)
        );
        assert_approx_eq!(
            (Hue::<f64>::from(Degrees::<f64>::from(f64::MAGENTA_ANGLE)) - Degrees::from(60.0))
                .angle,
            Degrees::<f64>::from(f64::BLUE_ANGLE)
        )
    }

    #[test]
    fn difference() {
        assert_approx_eq!(
            (Hue::<f64>::from(Degrees::<f64>::from(f64::YELLOW_ANGLE))
                - Hue::from(Degrees::<f64>::from(f64::GREEN_ANGLE))),
            Degrees::from(-60.0)
        );
        assert_approx_eq!(
            (Hue::<f64>::from(Degrees::<f64>::from(f64::YELLOW_ANGLE))
                - Hue::from(Degrees::<f64>::from(f64::MAGENTA_ANGLE))),
            Degrees::from(120.0)
        );
    }

    #[test]
    fn chroma_and_value_ranges() {
        for angle in TEST_ANGLES.iter().map(|x| Degrees::from(*x)) {
            let hue = Hue::from(angle);
            for chroma in NON_ZERO_TEST_RATIOS.iter() {
                if let Some((shade_value, tint_value)) = hue.value_range_with_chroma(*chroma) {
                    // TODO: try and make these exact reciprocals
                    let max_chroma = hue.max_chroma_for_value(shade_value);
                    assert_approx_eq!(*chroma, max_chroma);
                    let max_chroma = hue.max_chroma_for_value(tint_value);
                    assert_approx_eq!(*chroma, max_chroma, 0.00000000001);
                }
            }
        }
    }

    #[test]
    fn max_chroma_for_value() {
        for angle in TEST_ANGLES.iter().map(|x| Degrees::from(*x)) {
            let hue = Hue::from(angle);
            for value in NON_ZERO_TEST_RATIOS.iter().filter(|x| **x < 1.0) {
                let max_chroma = hue.max_chroma_for_value(*value);
                if let Some(rgb) = hue.rgb_with_chroma_and_value(max_chroma, *value) {
                    assert_approx_eq!(rgb.chroma(), max_chroma, 0.000000000000001);
                    assert_approx_eq!(rgb.value(), *value);
                    assert_approx_eq!(angle, Hue::from(rgb).angle, 0.000000000000001);
                } else {
                    if let Some((shade_value, tint_value)) = hue.value_range_with_chroma(max_chroma)
                    {
                        // TODO: Try and enable panic! version
                        assert!(*value < shade_value || *value > tint_value);
                    //panic!(
                    //    "hue: {:?} value: {} max chroma: {} range: ({}..{} {:?}",
                    //    angle, value, max_chroma, shade_value, tint_value, hue.max_chroma_rgb
                    //);
                    } else {
                        panic!(
                            "hue: {:?} value: {} max chroma: {}",
                            angle, value, max_chroma
                        );
                    }
                }
            }
            for value in [0.0, 1.0].iter() {
                let max_chroma = hue.max_chroma_for_value(*value);
                let rgb = hue.rgb_with_chroma_and_value(max_chroma, *value).unwrap();
                assert_approx_eq!(rgb.chroma(), max_chroma,);
                assert_approx_eq!(rgb.value(), *value);
                assert!(Hue::from(rgb).is_grey());
            }
        }
        let hue = Hue::from(Degrees::from(std::f64::NAN));
        for value in NON_ZERO_TEST_RATIOS.iter() {
            let max_chroma = hue.max_chroma_for_value(*value);
            assert_eq!(max_chroma, 0.0);
            let rgb = hue.rgb_with_chroma_and_value(max_chroma, *value).unwrap();
            assert_approx_eq!(rgb.chroma(), max_chroma,);
            assert_approx_eq!(rgb.value(), *value);
            assert!(Hue::from(rgb).is_grey());
        }
    }

    #[test]
    fn rgb_range_with_chroma() {
        for angle in TEST_ANGLES.iter().map(|x| Degrees::from(*x)) {
            let hue: Hue<f64> = angle.into();
            assert_eq!(
                hue.rgb_range_with_chroma(0.0).unwrap(),
                (RGB::BLACK, RGB::WHITE)
            );
            for chroma in NON_ZERO_TEST_RATIOS.iter() {
                let (shade_rgb, tint_rgb) = hue.rgb_range_with_chroma(*chroma).unwrap();
                assert!(
                    shade_rgb.value() <= tint_rgb.value(),
                    "{} == {} :: {} : {:?}",
                    shade_rgb.value(),
                    tint_rgb.value(),
                    angle.degrees(),
                    shade_rgb
                );
                assert_approx_eq!(shade_rgb.chroma(), *chroma, 0.000000000000001);
                assert_approx_eq!(tint_rgb.chroma(), *chroma, 0.000000000000001);
                assert_approx_eq!(angle, Hue::from(shade_rgb).angle, 0.000000000000001);
                assert_approx_eq!(angle, Hue::from(tint_rgb).angle, 0.00000000000001);
            }
        }
        let hue = Hue::<f64>::from(Degrees::<f64>::from(std::f64::NAN));
        assert_eq!(
            hue.rgb_range_with_chroma(0.0),
            Some((RGB::BLACK, RGB::WHITE))
        );
        for chroma in NON_ZERO_TEST_RATIOS.iter() {
            assert!(hue.rgb_range_with_chroma(*chroma).is_none())
        }
    }

    #[test]
    fn rgb_with_chroma_and_value() {
        let mut count_a = 0;
        let mut count_b = 0;
        for angle in TEST_ANGLES.iter().map(|x| Degrees::from(*x)) {
            let hue = Hue::<f64>::from(angle);
            for chroma in NON_ZERO_TEST_RATIOS.iter() {
                for value in NON_ZERO_TEST_RATIOS.iter() {
                    match hue.rgb_with_chroma_and_value(*chroma, *value) {
                        Some(rgb) => {
                            assert_approx_eq!(rgb.chroma(), *chroma, 0.000000000000001);
                            assert_approx_eq!(rgb.value(), *value);
                            assert_approx_eq!(hue.angle, Hue::from(rgb).angle, 0.00000000000001);
                        }
                        None => {
                            if let Some((min_value, max_value)) =
                                hue.value_range_with_chroma(*chroma)
                            {
                                assert!(*value < min_value || *value > max_value);
                            } else {
                                panic!("File: {:?} Line: {:?} shouldn't get here", file!(), line!())
                            }
                        }
                    }
                }
            }
            // Check for handling of hue drift for small chroma values
            for value in TEST_RATIOS.iter() {
                for chroma in [0.000000001, 0.0000000001, 0.00000000001, 0.000000000001].iter() {
                    match hue.rgb_with_chroma_and_value(*chroma, *value) {
                        Some(rgb) => {
                            let rgb_hue = Hue::from(rgb);
                            if rgb_hue.is_grey() {
                                assert_approx_eq!(rgb.value(), *value);
                                count_a += 1;
                            } else {
                                //assert!(approx_eq!(f64, rgb.chroma(), *chroma, epsilon = 0.000000000000001));
                                assert_approx_eq!(rgb.value(), *value);
                                assert_approx_eq!(hue.angle, rgb_hue.angle, 0.000000000000001);
                                count_b += 1;
                            }
                        }
                        None => {
                            if let Some((min_value, max_value)) =
                                hue.value_range_with_chroma(*chroma)
                            {
                                assert!(*value < min_value || *value > max_value);
                            } else {
                                panic!("File: {:?} Line: {:?} shouldn't get here", file!(), line!())
                            }
                        }
                    }
                }
            }
            for value in TEST_RATIOS.iter() {
                match hue.rgb_with_chroma_and_value(0.0, *value) {
                    Some(rgb) => {
                        assert_approx_eq!(rgb.chroma(), 0.0);
                        assert_approx_eq!(rgb.value(), *value);
                        assert!(Hue::from(rgb).is_grey());
                    }
                    None => (assert!(false)),
                }
            }
        }
        assert!(count_a > 0);
        assert!(count_b > 0);
        let hue = Hue::from(Degrees::from(std::f64::NAN));
        for chroma in NON_ZERO_TEST_RATIOS.iter() {
            for value in TEST_RATIOS.iter() {
                assert_eq!(hue.rgb_with_chroma_and_value(*chroma, *value), None);
            }
        }
        for value in TEST_RATIOS.iter() {
            assert_eq!(
                hue.rgb_with_chroma_and_value(0.0, *value),
                Some([*value, *value, *value].into())
            );
        }
    }

    #[test]
    fn rgb_with_chroma_and_value_extremities() {
        for angle in TEST_ANGLES.iter().map(|x| Degrees::from(*x)) {
            let hue = Hue::<f64>::from(angle);
            for chroma in NON_ZERO_TEST_RATIOS.iter() {
                let (min_value, max_value) = hue.value_range_with_chroma(*chroma).unwrap();
                let shade_rgb = hue.rgb_with_chroma_and_value(*chroma, min_value).unwrap();
                let shade_hue = Hue::from(shade_rgb);
                assert_approx_eq!(shade_rgb.chroma(), *chroma, 0.000000000000001);
                assert_approx_eq!(shade_rgb.value(), min_value);
                assert_approx_eq!(angle, shade_hue.angle, 0.000000000000001);
                let tint_rgb = hue.rgb_with_chroma_and_value(*chroma, max_value).unwrap();
                let tint_value = tint_rgb.value();
                assert_approx_eq!(tint_value, max_value);
                assert_approx_eq!(tint_rgb.chroma(), *chroma, 0.000000000000001);
                let tint_hue = Hue::from(tint_rgb);
                assert_approx_eq!(angle, tint_hue.angle, 0.00000000000001);
            }
        }
    }

    #[test]
    fn rgb_with_max_chroma_for_value() {
        for angle in TEST_ANGLES.iter().map(|x| Degrees::from(*x)) {
            let hue = Hue::from(angle);
            for value in NON_ZERO_TEST_RATIOS.iter().map(|x| *x - 0.01) {
                let rgb = hue.max_chroma_rgb_with_value(value);
                assert_approx_eq!(rgb.value(), value);
                assert_approx_eq!(hue.angle, Hue::from(rgb).angle, 0.00000000000001);
                let max_chroma = hue.max_chroma_for_value(value);
                let rgb_chroma = rgb.chroma();
                assert_approx_eq!(rgb_chroma, max_chroma, 0.000000000000001);
            }
            for value in [0.0, 1.0].iter() {
                let rgb = hue.max_chroma_rgb_with_value(*value);
                assert_approx_eq!(rgb.value(), *value);
                assert!(Hue::from(rgb).is_grey());
            }
        }
        let hue = Hue::from(Degrees::from(std::f64::NAN));
        for value in TEST_RATIOS.iter() {
            assert_eq!(
                hue.max_chroma_rgb_with_value(*value),
                [*value, *value, *value].into()
            );
        }
    }
}
