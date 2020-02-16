// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    cmp::{Ordering, PartialEq, PartialOrd},
    convert::{From, TryFrom},
    hash::{Hash, Hasher},
    ops::{Add, Sub},
};

use normalised_angles::*;

use crate::rgb::RGB;
use crate::{chroma, ColourComponent, ColourInterface, HueConstants, RGBConstants};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct HueMod<F: ColourComponent> {
    angle: Option<Degrees<F>>,
    max_chroma_rgb: RGB<F>,
    chroma_correction: F,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Hue<F: ColourComponent> {
    angle: Degrees<F>,
    max_chroma_rgb: RGB<F>,
    chroma_correction: F,
}

impl<F: ColourComponent> From<Degrees<F>> for Hue<F> {
    fn from(angle: Degrees<F>) -> Self {
        let other = chroma::calc_other_from_angle(angle.abs());
        let max_chroma_rgb: RGB<F> = if angle >= Degrees::RED {
            if angle <= Degrees::YELLOW {
                [F::ONE, other, F::ZERO].into()
            } else if angle <= Degrees::GREEN {
                [other, F::ONE, F::ZERO].into()
            } else {
                [F::ZERO, F::ONE, other].into()
            }
        } else if angle >= Degrees::MAGENTA {
            [F::ONE, F::ZERO, other].into()
        } else if angle >= Degrees::BLUE {
            [other, F::ZERO, F::ONE].into()
        } else {
            [F::ZERO, other, F::ONE].into()
        };
        let chroma_correction = chroma::calc_chroma_correction(other);
        Self {
            angle,
            max_chroma_rgb,
            chroma_correction,
        }
    }
}

impl<F: ColourComponent> TryFrom<RGB<F>> for Hue<F> {
    type Error = &'static str;

    fn try_from(rgb: RGB<F>) -> Result<Self, Self::Error> {
        use std::convert::TryInto;
        let angle: Degrees<F> = rgb.xy().try_into()?;
        let io = rgb.indices_value_order();
        let mut parts: [F; 3] = [F::ZERO, F::ZERO, F::ZERO];
        parts[io[0] as usize] = F::ONE;
        if rgb[io[0]] == rgb[io[1]] {
            // Secondary colour
            parts[io[1] as usize] = F::ONE;
        } else if rgb[io[1]] != rgb[io[2]] {
            // Not Primary or Secondary
            parts[io[1] as usize] = chroma::calc_other_from_angle(angle.abs());
        }
        let max_chroma_rgb: RGB<F> = parts.into();
        let chroma_correction = chroma::calc_chroma_correction(max_chroma_rgb[io[1]]);
        Ok(Self {
            angle,
            max_chroma_rgb,
            chroma_correction,
        })
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

    /// Returns the range of `RGB` that can be created with this hue and the given `chroma`
    /// returns `None` if no such range exists. `chroma` must be in range 0.0 to 1.0 inclusive.
    pub fn rgb_range_with_chroma(&self, chroma: F) -> (RGB<F>, RGB<F>) {
        debug_assert!(chroma.is_proportion());
        if chroma == F::ZERO {
            (RGB::BLACK, RGB::WHITE)
        } else if chroma == F::ONE {
            (self.max_chroma_rgb, self.max_chroma_rgb)
        } else {
            let shade: [F; 3] = [
                self.max_chroma_rgb[0] * chroma,
                self.max_chroma_rgb[1] * chroma,
                self.max_chroma_rgb[2] * chroma,
            ];
            let delta = F::ONE - chroma;
            let tint: [F; 3] = [shade[0] + delta, shade[1] + delta, shade[2] + delta];
            (shade.into(), tint.into())
        }
    }

    /// Returns the range of `values` for which it is possible to construct an `RGB` with this hue
    /// and the specified `chroma`.
    pub fn value_range_with_chroma(&self, chroma: F) -> (F, F) {
        debug_assert!(chroma.is_proportion());
        if chroma == F::ZERO {
            (F::ZERO, F::ONE)
        } else if chroma == F::ONE {
            let val = self.max_chroma_rgb.value();
            (val, val)
        } else {
            // NB using sum() rather than value() for numeric accuracy
            let shade = self.max_chroma_rgb.sum() * chroma / F::THREE;
            let tint =
                (F::THREE + self.max_chroma_rgb.sum() * chroma - chroma * F::THREE) / F::THREE;
            (shade, tint)
        }
    }

    /// Returns a `RGB` with the specified `chroma` and `value` if feasible and `None` otherwise.
    pub fn rgb_with_chroma_and_value(&self, chroma: F, value: F) -> Option<RGB<F>> {
        debug_assert!(chroma.is_proportion(), "{:?}", chroma);
        debug_assert!(value.is_proportion(), "{:?}", value);
        let (min_value, max_value) = self.value_range_with_chroma(chroma);
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
            if let Some(angle) = Degrees::atan2(y, x) {
                if angle.approx_eq(&self.angle, Some(F::from(0.000_000_000_000_01).unwrap())) {
                    Some(rgb)
                } else {
                    Some(RGB::from([value, value, value]))
                }
            } else {
                Some(RGB::from([value, value, value]))
            }
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
                array[io[2] as usize] = shortfall / (F::TWO - other);
                array[io[1] as usize] = other + shortfall - array[io[2] as usize];
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
    use crate::{HueConstants, RGBConstants, I_BLUE, I_GREEN, I_RED};
    use num_traits_plus::assert_approx_eq;

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
            Hue::<f64>::from(Degrees::RED).max_chroma_rgb,
            RGB::<f64>::RED
        );
        assert_eq!(
            Hue::<f64>::from(Degrees::GREEN).max_chroma_rgb,
            RGB::<f64>::GREEN
        );
        assert_eq!(
            Hue::<f64>::from(Degrees::BLUE).max_chroma_rgb,
            RGB::<f64>::BLUE
        );
        assert_eq!(
            Hue::<f64>::from(Degrees::CYAN).max_chroma_rgb,
            RGB::<f64>::CYAN
        );
        assert_eq!(
            Hue::<f64>::from(Degrees::MAGENTA).max_chroma_rgb,
            RGB::<f64>::MAGENTA
        );
        assert_eq!(
            Hue::<f64>::from(Degrees::YELLOW).max_chroma_rgb,
            RGB::<f64>::YELLOW
        );
    }

    #[test]
    fn chroma_correction_from_angle() {
        assert_eq!(Hue::<f64>::from(Degrees::RED).chroma_correction, 1.0);
        assert_eq!(Hue::<f64>::from(Degrees::GREEN).chroma_correction, 1.0);
        assert_eq!(Hue::<f64>::from(Degrees::BLUE).chroma_correction, 1.0);
        assert_eq!(Hue::<f64>::from(Degrees::CYAN).chroma_correction, 1.0);
        assert_eq!(Hue::<f64>::from(Degrees::MAGENTA).chroma_correction, 1.0);
        assert_eq!(Hue::<f64>::from(Degrees::YELLOW).chroma_correction, 1.0);
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
            Hue::<f64>::try_from(RGB::<f64>::RED).unwrap().angle,
            Degrees::RED
        );
        assert_approx_eq!(
            Hue::<f64>::try_from(RGB::<f64>::GREEN).unwrap().angle,
            Degrees::GREEN
        );
        assert_approx_eq!(
            Hue::<f64>::try_from(RGB::<f64>::BLUE).unwrap().angle,
            Degrees::BLUE
        );
        assert_approx_eq!(
            Hue::<f64>::try_from(RGB::<f64>::CYAN).unwrap().angle,
            Degrees::CYAN
        );
        assert_approx_eq!(
            Hue::<f64>::try_from(RGB::<f64>::MAGENTA).unwrap().angle,
            Degrees::MAGENTA
        );
        assert_approx_eq!(
            Hue::<f64>::try_from(RGB::<f64>::YELLOW).unwrap().angle,
            Degrees::YELLOW
        );
        assert!(Hue::<f64>::try_from(RGB::<f64>::BLACK).is_err());
        assert!(Hue::<f64>::try_from(RGB::<f64>::WHITE).is_err());
    }

    #[test]
    fn chroma_correction_from_rgb() {
        assert_eq!(
            Hue::<f64>::try_from(RGB::RED).unwrap().chroma_correction,
            1.0
        );
        assert_eq!(
            Hue::<f64>::try_from(RGB::GREEN).unwrap().chroma_correction,
            1.0
        );
        assert_eq!(
            Hue::<f64>::try_from(RGB::BLUE).unwrap().chroma_correction,
            1.0
        );
        assert_eq!(
            Hue::<f64>::try_from(RGB::CYAN).unwrap().chroma_correction,
            1.0
        );
        assert_eq!(
            Hue::<f64>::try_from(RGB::MAGENTA)
                .unwrap()
                .chroma_correction,
            1.0
        );
        assert_eq!(
            Hue::<f64>::try_from(RGB::YELLOW).unwrap().chroma_correction,
            1.0
        );

        assert_approx_eq!(
            Hue::<f64>::try_from(RGB::from([0.5, 0.25, 0.0]))
                .unwrap()
                .chroma_correction,
            2.0 / 3.0_f64.sqrt(),
        );
        assert_approx_eq!(
            Hue::<f64>::try_from(RGB::from([0.25, 0.5, 0.0]))
                .unwrap()
                .chroma_correction,
            2.0 / 3.0_f64.sqrt(),
        );
        assert_approx_eq!(
            Hue::<f64>::try_from(RGB::from([0.5, 0.0, 0.25]))
                .unwrap()
                .chroma_correction,
            2.0 / 3.0_f64.sqrt(),
        );
        assert_approx_eq!(
            Hue::<f64>::try_from(RGB::from([0.25, 0.0, 0.5]))
                .unwrap()
                .chroma_correction,
            2.0 / 3.0_f64.sqrt(),
        );
        assert_approx_eq!(
            Hue::<f64>::try_from(RGB::from([0.0, 0.5, 0.25]))
                .unwrap()
                .chroma_correction,
            2.0 / 3.0_f64.sqrt(),
        );
        assert_approx_eq!(
            Hue::<f64>::try_from(RGB::from([0.0, 0.25, 0.5]))
                .unwrap()
                .chroma_correction,
            2.0 / 3.0_f64.sqrt(),
        );
    }

    #[test]
    fn rotation() {
        assert_approx_eq!(
            (Hue::<f64>::from(Degrees::YELLOW) + Degrees::from(60.0)).angle,
            Degrees::GREEN
        );
        assert_approx_eq!(
            (Hue::<f64>::from(Degrees::MAGENTA) - Degrees::from(60.0)).angle,
            Degrees::BLUE
        )
    }

    #[test]
    fn difference() {
        assert_approx_eq!(
            (Hue::<f64>::from(Degrees::YELLOW) - Hue::from(Degrees::GREEN)),
            Degrees::from(-60.0)
        );
        assert_approx_eq!(
            (Hue::<f64>::from(Degrees::YELLOW) - Hue::from(Degrees::MAGENTA)),
            Degrees::from(120.0)
        );
    }

    #[test]
    fn chroma_and_value_ranges() {
        for angle in TEST_ANGLES.iter().map(|x| Degrees::from(*x)) {
            let hue = Hue::from(angle);
            for chroma in NON_ZERO_TEST_RATIOS.iter() {
                let (shade_value, tint_value) = hue.value_range_with_chroma(*chroma);
                // TODO: try and make these exact reciprocals
                let max_chroma = hue.max_chroma_for_value(shade_value);
                assert_approx_eq!(*chroma, max_chroma);
                let max_chroma = hue.max_chroma_for_value(tint_value);
                assert_approx_eq!(*chroma, max_chroma, 0.00000000001);
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
                    assert_approx_eq!(angle, Hue::try_from(rgb).unwrap().angle, 0.000000000000001);
                } else {
                    let (shade_value, tint_value) = hue.value_range_with_chroma(max_chroma);
                    // TODO: Try and enable panic! version
                    assert!(*value < shade_value || *value > tint_value);
                }
            }
            for value in [0.0, 1.0].iter() {
                let max_chroma = hue.max_chroma_for_value(*value);
                let rgb = hue.rgb_with_chroma_and_value(max_chroma, *value).unwrap();
                assert_approx_eq!(rgb.chroma(), max_chroma,);
                assert_approx_eq!(rgb.value(), *value);
                assert!(Hue::try_from(rgb).is_err());
            }
        }
    }

    #[test]
    fn rgb_range_with_chroma() {
        for angle in TEST_ANGLES.iter().map(|x| Degrees::from(*x)) {
            let hue: Hue<f64> = angle.into();
            assert_eq!(hue.rgb_range_with_chroma(0.0), (RGB::BLACK, RGB::WHITE));
            for chroma in NON_ZERO_TEST_RATIOS.iter() {
                let (shade_rgb, tint_rgb) = hue.rgb_range_with_chroma(*chroma);
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
                assert_approx_eq!(
                    angle,
                    Hue::try_from(shade_rgb).unwrap().angle,
                    0.000000000000001
                );
                assert_approx_eq!(
                    angle,
                    Hue::try_from(tint_rgb).unwrap().angle,
                    0.00000000000001
                );
            }
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
                            assert_approx_eq!(
                                hue.angle,
                                Hue::try_from(rgb).unwrap().angle,
                                0.00000000000001
                            );
                        }
                        None => {
                            let (min_value, max_value) = hue.value_range_with_chroma(*chroma);
                            assert!(*value < min_value || *value > max_value);
                        }
                    }
                }
            }
            // Check for handling of hue drift for small chroma values
            for value in TEST_RATIOS.iter() {
                for chroma in [0.000000001, 0.0000000001, 0.00000000001, 0.000000000001].iter() {
                    match hue.rgb_with_chroma_and_value(*chroma, *value) {
                        Some(rgb) => {
                            if let Ok(rgb_hue) = Hue::try_from(rgb) {
                                assert_approx_eq!(rgb.chroma(), *chroma, 0.000000000000001);
                                assert_approx_eq!(rgb.value(), *value);
                                assert_approx_eq!(hue.angle, rgb_hue.angle, 0.000000000000001);
                                count_b += 1;
                            } else {
                                assert_approx_eq!(rgb.value(), *value);
                                count_a += 1;
                            }
                        }
                        None => {
                            let (min_value, max_value) = hue.value_range_with_chroma(*chroma);
                            assert!(*value < min_value || *value > max_value);
                        }
                    }
                }
            }
            for value in TEST_RATIOS.iter() {
                match hue.rgb_with_chroma_and_value(0.0, *value) {
                    Some(rgb) => {
                        assert_approx_eq!(rgb.chroma(), 0.0);
                        assert_approx_eq!(rgb.value(), *value);
                        assert!(Hue::try_from(rgb).is_err());
                    }
                    None => (assert!(false)),
                }
            }
        }
        assert!(count_a > 0);
        assert!(count_b > 0);
    }

    #[test]
    fn rgb_with_chroma_and_value_extremities() {
        for angle in TEST_ANGLES.iter().map(|x| Degrees::from(*x)) {
            let hue = Hue::<f64>::from(angle);
            for chroma in NON_ZERO_TEST_RATIOS.iter() {
                let (min_value, max_value) = hue.value_range_with_chroma(*chroma);
                let shade_rgb = hue.rgb_with_chroma_and_value(*chroma, min_value).unwrap();
                let shade_hue = Hue::try_from(shade_rgb).unwrap();
                assert_approx_eq!(shade_rgb.chroma(), *chroma, 0.000000000000001);
                assert_approx_eq!(shade_rgb.value(), min_value);
                assert_approx_eq!(angle, shade_hue.angle, 0.000000000000001);
                let tint_rgb = hue.rgb_with_chroma_and_value(*chroma, max_value).unwrap();
                let tint_value = tint_rgb.value();
                assert_approx_eq!(tint_value, max_value);
                assert_approx_eq!(tint_rgb.chroma(), *chroma, 0.000000000000001);
                let tint_hue = Hue::try_from(tint_rgb).unwrap();
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
                assert_approx_eq!(
                    hue.angle,
                    Hue::try_from(rgb).unwrap().angle,
                    0.00000000000001
                );
                let max_chroma = hue.max_chroma_for_value(value);
                let rgb_chroma = rgb.chroma();
                assert_approx_eq!(rgb_chroma, max_chroma, 0.000000000000001);
            }
            for value in [0.0, 1.0].iter() {
                let rgb = hue.max_chroma_rgb_with_value(*value);
                assert_approx_eq!(rgb.value(), *value);
                assert!(Hue::try_from(rgb).is_err());
            }
        }
    }
}
