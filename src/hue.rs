// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    cmp::{Ordering, PartialEq, PartialOrd},
    convert::From,
    hash::{Hash, Hasher},
    ops::{Add, Sub},
};

use normalised_angles::{AngleConst, Degrees};
use num::traits::{Float, NumAssign, NumOps};

use crate::rgb::{is_proportion, ZeroOneEtc, RGB};

pub trait HueAngles: Float + NumAssign + NumOps + AngleConst + Copy + ZeroOneEtc {
    const RED_ANGLE: Self;
    const GREEN_ANGLE: Self;
    const BLUE_ANGLE: Self;

    const CYAN_ANGLE: Self;
    const YELLOW_ANGLE: Self;
    const MAGENTA_ANGLE: Self;
}

impl HueAngles for f32 {
    const RED_ANGLE: Self = 0.0;
    const GREEN_ANGLE: Self = 120.0;
    const BLUE_ANGLE: Self = -120.0;

    const CYAN_ANGLE: Self = 180.0;
    const YELLOW_ANGLE: Self = 60.0;
    const MAGENTA_ANGLE: Self = -60.0;
}

impl HueAngles for f64 {
    const RED_ANGLE: Self = 0.0;
    const GREEN_ANGLE: Self = 120.0;
    const BLUE_ANGLE: Self = -120.0;

    const CYAN_ANGLE: Self = 180.0;
    const YELLOW_ANGLE: Self = 60.0;
    const MAGENTA_ANGLE: Self = -60.0;
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Hue<F: HueAngles> {
    angle: Degrees<F>,
    max_chroma_rgb: RGB<F>,
    chroma_correction: F,
}

impl<F: HueAngles> Hue<F> {
    fn calc_other(abs_angle: Degrees<F>) -> F {
        if [F::RED_ANGLE, F::GREEN_ANGLE].contains(&abs_angle.degrees()) {
            F::ZERO
        } else if [F::YELLOW_ANGLE, F::CYAN_ANGLE].contains(&abs_angle.degrees()) {
            F::ONE
        } else {
            fn f<F: Float + NumAssign + NumOps + AngleConst + ZeroOneEtc + HueAngles>(
                angle: Degrees<F>,
            ) -> F {
                // Careful of float not fully representing reals
                (angle.sin() / (Degrees::from(F::GREEN_ANGLE) - angle).sin()).min(F::ONE)
            };
            if abs_angle.degrees() <= F::YELLOW_ANGLE {
                f(abs_angle)
            } else if abs_angle.degrees() <= F::GREEN_ANGLE {
                f(Degrees::from(F::GREEN_ANGLE) - abs_angle)
            } else {
                f(abs_angle - Degrees::from(F::GREEN_ANGLE))
            }
        }
    }
}

impl<F: HueAngles> From<Degrees<F>> for Hue<F> {
    fn from(angle: Degrees<F>) -> Self {
        if angle.is_nan() {
            Self {
                angle,
                max_chroma_rgb: RGB::<F>::WHITE,
                chroma_correction: F::ONE,
            }
        } else {
            let other = Self::calc_other(angle.abs());
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
            // Careful of fact floats only approximate real numbers
            let chroma_correction = (F::ONE + other * other - other).sqrt().min(F::ONE).recip();
            Self {
                angle,
                max_chroma_rgb,
                chroma_correction,
            }
        }
    }
}

impl<F: HueAngles> From<RGB<F>> for Hue<F> {
    fn from(rgb: RGB<F>) -> Self {
        let (x, y) = rgb.xy();
        let angle: Degrees<F> = Degrees::atan2(x, y);
        if angle.is_nan() {
            // NB: float limitations make using ::from(angle) unwise for real angles
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
            parts[io[1]] = Self::calc_other(angle.abs());
        }
        let max_chroma_rgb: RGB<F> = parts.into();
        let (x, y) = max_chroma_rgb.xy();
        // Be paranoid about fact floats only approximate reals
        let chroma_correction = x.hypot(y).min(F::ONE).recip();
        Self {
            angle,
            max_chroma_rgb,
            chroma_correction,
        }
    }
}

impl<F: HueAngles> Hash for Hue<F>
where
    Degrees<F>: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.angle.hash(state)
    }
}

impl<F: HueAngles> PartialEq for Hue<F> {
    fn eq(&self, other: &Hue<F>) -> bool {
        self.angle.eq(&other.angle)
    }
}

impl<F: HueAngles> PartialOrd for Hue<F> {
    fn partial_cmp(&self, other: &Hue<F>) -> Option<Ordering> {
        self.angle.partial_cmp(&other.angle)
    }
}

impl<F: HueAngles> Add<Degrees<F>> for Hue<F> {
    type Output = Self;

    fn add(self, angle: Degrees<F>) -> Self {
        (self.angle + angle).into()
    }
}

impl<F: HueAngles> Sub<Degrees<F>> for Hue<F> {
    type Output = Self;

    fn sub(self, angle: Degrees<F>) -> Self {
        (self.angle - angle).into()
    }
}

impl<F: HueAngles> Sub<Hue<F>> for Hue<F> {
    type Output = Degrees<F>;

    fn sub(self, other: Hue<F>) -> Degrees<F> {
        self.angle - other.angle
    }
}
impl<F: HueAngles> Hue<F> {
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
        debug_assert!(is_proportion(value));
        if self.is_grey() {
            F::ZERO
        } else {
            let mcv = self.max_chroma_rgb.value();
            // NB these will be safe because mcv will be between 1.0 / 3.0 and 2.0 / 3.0
            if mcv > value {
                value / mcv
            } else {
                (F::ONE - value) / (F::ONE - mcv)
            }
        }
    }

    /// Returns the range of `RGB` that can be created with this hue and the given `chroma`
    /// returns `None` if no such range exists. `chroma` must be in range 0.0 to 1.0 inclusive.
    pub fn rgb_range_with_chroma(&self, chroma: F) -> Option<(RGB<F>, RGB<F>)> {
        debug_assert!(is_proportion(chroma));
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
        assert!(is_proportion(chroma));
        if chroma == F::ZERO {
            Some((F::ZERO, F::ONE))
        } else if self.is_grey() {
            None
        } else if chroma == F::ONE {
            let val = self.max_chroma_rgb.value();
            Some((val, val))
        } else {
            let shade = self.max_chroma_rgb.value() * chroma;
            let tint = shade + (F::ONE - chroma);
            Some((shade, tint))
        }
    }

    /// Returns a `RGB` with the specified `chroma` and `value` if feasible and `None` otherwise.
    pub fn rgb_with_chroma_and_value(&self, chroma: F, value: F) -> Option<RGB<F>> {
        assert!(is_proportion(chroma));
        assert!(is_proportion(value));
        if let Some((min_value, max_value)) = self.value_range_with_chroma(chroma) {
            if value < min_value || value > max_value {
                None
            } else {
                let delta = value - min_value;
                let rgb: RGB<F> = [
                    self.max_chroma_rgb[0] * chroma + delta,
                    self.max_chroma_rgb[1] * chroma + delta,
                    self.max_chroma_rgb[2] * chroma + delta,
                ]
                .into();
                // NB: because floats only approximate reals trying to
                // set chroma too small (but non zero) results in a drift
                // in the hue angle of the resulting RGB. When this
                // happens we go straight to a zero chroma RGB
                let (x, y) = rgb.xy();
                let angle: Degrees<F> = Degrees::atan2(x, y);
                if angle.approx_eq(self.angle) {
                    Some(rgb)
                } else {
                    Some(RGB::from([value, value, value]))
                }
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rgb::{I_BLUE, I_GREEN, I_RED};
    use float_cmp::*;

    const TEST_ANGLES: [f64; 13] = [
        -180.0, -150.0, -120.0, -90.0, -60.0, -30.0, 0.0, 30.0, 60.0, 90.0, 120.0, 150.0, 180.0,
    ];

    const TEST_RATIOS: [f64; 10] = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];

    fn calculate_chroma(rgb: &RGB<f64>) -> f64 {
        let (x, y) = rgb.xy();
        x.hypot(y) * Hue::from(*rgb).chroma_correction
    }

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
    fn from_rgb() {
        assert!(Hue::<f64>::from(RGB::<f64>::RED)
            .angle
            .approx_eq(Degrees::<f64>::from(f64::RED_ANGLE)));
        assert!(Hue::<f64>::from(RGB::<f64>::GREEN)
            .angle
            .approx_eq(Degrees::<f64>::from(f64::GREEN_ANGLE)));
        assert!(Hue::<f64>::from(RGB::<f64>::BLUE)
            .angle
            .approx_eq(Degrees::<f64>::from(f64::BLUE_ANGLE)));
        assert!(Hue::<f64>::from(RGB::<f64>::CYAN)
            .angle
            .approx_eq(Degrees::<f64>::from(f64::CYAN_ANGLE)));
        assert!(Hue::<f64>::from(RGB::<f64>::MAGENTA)
            .angle
            .approx_eq(Degrees::<f64>::from(f64::MAGENTA_ANGLE)));
        assert!(Hue::<f64>::from(RGB::<f64>::YELLOW)
            .angle
            .approx_eq(Degrees::<f64>::from(f64::YELLOW_ANGLE)));
        assert!(Hue::<f64>::from(RGB::<f64>::BLACK).angle.is_nan());
        assert!(Hue::<f64>::from(RGB::<f64>::WHITE).angle.is_nan());
    }

    #[test]
    fn rotation() {
        assert!(
            (Hue::<f64>::from(Degrees::<f64>::from(f64::YELLOW_ANGLE)) + Degrees::from(60.0))
                .angle
                .approx_eq(Degrees::<f64>::from(f64::GREEN_ANGLE))
        );
        assert!(
            (Hue::<f64>::from(Degrees::<f64>::from(f64::MAGENTA_ANGLE)) - Degrees::from(60.0))
                .angle
                .approx_eq(Degrees::<f64>::from(f64::BLUE_ANGLE))
        )
    }

    #[test]
    fn difference() {
        assert!((Hue::<f64>::from(Degrees::<f64>::from(f64::YELLOW_ANGLE))
            - Hue::from(Degrees::<f64>::from(f64::GREEN_ANGLE)))
        .approx_eq(Degrees::from(-60.0)));
        assert!((Hue::<f64>::from(Degrees::<f64>::from(f64::YELLOW_ANGLE))
            - Hue::from(Degrees::<f64>::from(f64::MAGENTA_ANGLE)))
        .approx_eq(Degrees::from(120.0)));
    }

    #[test]
    fn rgb_range_with_chroma() {
        for angle in TEST_ANGLES.iter().map(|x| Degrees::from(*x)) {
            let hue_angle: Hue<f64> = angle.into();
            assert_eq!(
                hue_angle.rgb_range_with_chroma(0.0).unwrap(),
                (RGB::BLACK, RGB::WHITE)
            );
            for chroma in TEST_RATIOS.iter() {
                let (shade_rgb, tint_rgb) = hue_angle.rgb_range_with_chroma(*chroma).unwrap();
                assert!(shade_rgb.value() <= tint_rgb.value());
                let shade_chroma = calculate_chroma(&shade_rgb);
                assert!(approx_eq!(f64, shade_chroma, *chroma, ulps = 4));
                let tint_chroma = calculate_chroma(&tint_rgb);
                assert!(approx_eq!(f64, tint_chroma, *chroma, ulps = 4));
                assert!(angle.approx_eq(Hue::from(shade_rgb).angle));
                assert!(angle.approx_eq(Hue::from(tint_rgb).angle));
            }
        }
        let hue_angle = Hue::<f64>::from(Degrees::<f64>::from(std::f64::NAN));
        assert_eq!(
            hue_angle.rgb_range_with_chroma(0.0),
            Some((RGB::BLACK, RGB::WHITE))
        );
        for chroma in TEST_RATIOS.iter() {
            assert!(hue_angle.rgb_range_with_chroma(*chroma).is_none())
        }
    }
}
