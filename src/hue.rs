// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    cmp::{Ordering, PartialEq, PartialOrd},
    convert::From,
    hash::{Hash, Hasher},
    ops::{Add, Sub},
};

use normalised_angles::{Angle, AngleConst};
use num::traits::{Float, NumAssign, NumOps};

use crate::rgb::{is_proportion, ZeroOneEtc, RGB};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct HueAngle<F>
where
    F: Float + NumAssign + NumOps + AngleConst + Copy + ZeroOneEtc,
{
    angle: Angle<F>,
    max_chroma_rgb: RGB<F>,
    chroma_correction: F,
}

impl<F> HueAngle<F>
where
    F: Float + NumAssign + NumOps + AngleConst + Copy + ZeroOneEtc,
{
    pub const RED_ANGLE: Angle<F> = Angle::<F>::DEG_0;
    pub const GREEN_ANGLE: Angle<F> = Angle::<F>::DEG_120;
    pub const BLUE_ANGLE: Angle<F> = Angle::<F>::NEG_DEG_120;

    pub const CYAN_ANGLE: Angle<F> = Angle::<F>::DEG_180;
    pub const YELLOW_ANGLE: Angle<F> = Angle::<F>::DEG_60;
    pub const MAGENTA_ANGLE: Angle<F> = Angle::<F>::NEG_DEG_60;

    fn calc_other(abs_angle: Angle<F>) -> F {
        if [Angle::<F>::DEG_0, Angle::<F>::DEG_120].contains(&abs_angle) {
            F::from(0.0).unwrap()
        } else if [Angle::<F>::DEG_60, Angle::<F>::DEG_180].contains(&abs_angle) {
            F::from(1.0).unwrap()
        } else {
            fn f<F: Float + NumAssign + NumOps + AngleConst>(angle: Angle<F>) -> F {
                // Careful of float not fully representing reals
                (angle.sin() / (Angle::<F>::DEG_120 - angle).sin()).min(F::from(1.0).unwrap())
            };
            if abs_angle <= Angle::<F>::DEG_60 {
                f(abs_angle)
            } else if abs_angle <= Angle::<F>::DEG_120 {
                f(Angle::<F>::DEG_120 - abs_angle)
            } else {
                f(abs_angle - Angle::<F>::DEG_120)
            }
        }
    }
}

impl<F> From<Angle<F>> for HueAngle<F>
where
    F: Float + NumAssign + NumOps + AngleConst + Copy + ZeroOneEtc,
{
    fn from(angle: Angle<F>) -> Self {
        if angle.is_nan() {
            Self {
                angle,
                max_chroma_rgb: RGB::<F>::WHITE,
                chroma_correction: F::ONE,
            }
        } else {
            let other = Self::calc_other(angle.abs());
            let max_chroma_rgb: RGB<F> = if angle >= Self::RED_ANGLE {
                if angle <= Self::YELLOW_ANGLE {
                    [F::ONE, other, F::ZERO].into()
                } else if angle <= Self::GREEN_ANGLE {
                    [other, F::ONE, F::ZERO].into()
                } else {
                    [F::ZERO, F::ONE, other].into()
                }
            } else {
                if angle >= Self::MAGENTA_ANGLE {
                    [F::ONE, F::ZERO, other].into()
                } else if angle >= Self::BLUE_ANGLE {
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

impl<F> From<RGB<F>> for HueAngle<F>
where
    F: Float + NumAssign + NumOps + AngleConst + Copy + ZeroOneEtc,
{
    fn from(rgb: RGB<F>) -> Self {
        let (x, y) = rgb.xy();
        let angle: Angle<F> = Angle::atan2(x, y);
        if angle.is_nan() {
            // NB: float limitations make using ::from(angle) unwise for real angles
            return HueAngle::from(angle);
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

impl<F> Hash for HueAngle<F>
where
    F: Float + NumAssign + NumOps + AngleConst + Copy + ZeroOneEtc,
    Angle<F>: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.angle.hash(state)
    }
}

impl<F> PartialEq for HueAngle<F>
where
    F: Float + NumAssign + NumOps + AngleConst + Copy + ZeroOneEtc,
{
    fn eq(&self, other: &HueAngle<F>) -> bool {
        self.angle.eq(&other.angle)
    }
}

impl<F> PartialOrd for HueAngle<F>
where
    F: Float + NumAssign + NumOps + AngleConst + Copy + ZeroOneEtc,
{
    fn partial_cmp(&self, other: &HueAngle<F>) -> Option<Ordering> {
        self.angle.partial_cmp(&other.angle)
    }
}

impl<F> Add<Angle<F>> for HueAngle<F>
where
    F: Float + NumAssign + NumOps + AngleConst + Copy + ZeroOneEtc,
{
    type Output = Self;

    fn add(self, angle: Angle<F>) -> Self {
        (self.angle + angle).into()
    }
}

impl<F> Sub<Angle<F>> for HueAngle<F>
where
    F: Float + NumAssign + NumOps + AngleConst + Copy + ZeroOneEtc,
{
    type Output = Self;

    fn sub(self, angle: Angle<F>) -> Self {
        (self.angle - angle).into()
    }
}

impl<F> Sub<HueAngle<F>> for HueAngle<F>
where
    F: Float + NumAssign + NumOps + AngleConst + Copy + ZeroOneEtc,
{
    type Output = Angle<F>;

    fn sub(self, other: HueAngle<F>) -> Angle<F> {
        self.angle - other.angle
    }
}
impl<F> HueAngle<F>
where
    F: Float + NumAssign + NumOps + AngleConst + Copy + ZeroOneEtc,
{
    /// Returns `true` if this `HueAngle` is grey i.e. completely devoid of colour/chroma/hue
    pub fn is_grey(&self) -> bool {
        self.angle.is_nan()
    }

    pub fn angle(&self) -> Angle<F> {
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
            let darkest: [F; 3] = [
                self.max_chroma_rgb[0] * chroma,
                self.max_chroma_rgb[1] * chroma,
                self.max_chroma_rgb[2] * chroma,
            ];
            let delta = F::ONE - chroma;
            let lightest: [F; 3] = [darkest[0] + delta, darkest[1] + delta, darkest[2] + delta];
            Some((darkest.into(), lightest.into()))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rgb::{I_BLUE, I_GREEN, I_RED};
    use float_cmp::*;

    const TEST_ANGLES: [Angle<f64>; 13] = [
        Angle::NEG_DEG_180,
        Angle::NEG_DEG_150,
        Angle::NEG_DEG_120,
        Angle::NEG_DEG_90,
        Angle::NEG_DEG_60,
        Angle::NEG_DEG_30,
        Angle::DEG_0,
        Angle::DEG_30,
        Angle::DEG_60,
        Angle::DEG_90,
        Angle::DEG_120,
        Angle::DEG_150,
        Angle::DEG_180,
    ];

    const TEST_RATIOS: [f64; 10] = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];

    fn calculate_chroma(rgb: &RGB<f64>) -> f64 {
        let (x, y) = rgb.xy();
        x.hypot(y) * HueAngle::from(*rgb).chroma_correction
    }

    #[test]
    fn from_angle() {
        assert_eq!(
            HueAngle::<f64>::from(Angle::NEG_DEG_150)
                .max_chroma_rgb
                .indices_value_order(),
            [I_BLUE, I_GREEN, I_RED]
        );
        assert_eq!(
            HueAngle::<f64>::from(Angle::NEG_DEG_90)
                .max_chroma_rgb
                .indices_value_order(),
            [I_BLUE, I_RED, I_GREEN]
        );
        assert_eq!(
            HueAngle::<f64>::from(Angle::NEG_DEG_30)
                .max_chroma_rgb
                .indices_value_order(),
            [I_RED, I_BLUE, I_GREEN]
        );
        assert_eq!(
            HueAngle::<f64>::from(Angle::DEG_30)
                .max_chroma_rgb
                .indices_value_order(),
            [I_RED, I_GREEN, I_BLUE]
        );
        assert_eq!(
            HueAngle::<f64>::from(Angle::DEG_90)
                .max_chroma_rgb
                .indices_value_order(),
            [I_GREEN, I_RED, I_BLUE]
        );
        assert_eq!(
            HueAngle::<f64>::from(Angle::DEG_150)
                .max_chroma_rgb
                .indices_value_order(),
            [I_GREEN, I_BLUE, I_RED]
        );
        assert_eq!(
            HueAngle::<f64>::from(HueAngle::<f64>::RED_ANGLE).max_chroma_rgb,
            RGB::<f64>::RED
        );
        assert_eq!(
            HueAngle::<f64>::from(HueAngle::<f64>::GREEN_ANGLE).max_chroma_rgb,
            RGB::<f64>::GREEN
        );
        assert_eq!(
            HueAngle::<f64>::from(HueAngle::<f64>::BLUE_ANGLE).max_chroma_rgb,
            RGB::<f64>::BLUE
        );
        assert_eq!(
            HueAngle::<f64>::from(HueAngle::<f64>::CYAN_ANGLE).max_chroma_rgb,
            RGB::<f64>::CYAN
        );
        assert_eq!(
            HueAngle::<f64>::from(HueAngle::<f64>::MAGENTA_ANGLE).max_chroma_rgb,
            RGB::<f64>::MAGENTA
        );
        assert_eq!(
            HueAngle::<f64>::from(HueAngle::<f64>::YELLOW_ANGLE).max_chroma_rgb,
            RGB::<f64>::YELLOW
        );
    }

    #[test]
    fn from_rgb() {
        assert!(HueAngle::<f64>::from(RGB::<f64>::RED)
            .angle
            .approx_eq(HueAngle::<f64>::RED_ANGLE));
        assert!(HueAngle::<f64>::from(RGB::<f64>::GREEN)
            .angle
            .approx_eq(HueAngle::<f64>::GREEN_ANGLE));
        assert!(HueAngle::<f64>::from(RGB::<f64>::BLUE)
            .angle
            .approx_eq(HueAngle::<f64>::BLUE_ANGLE));
        assert!(HueAngle::<f64>::from(RGB::<f64>::CYAN)
            .angle
            .approx_eq(HueAngle::<f64>::CYAN_ANGLE));
        assert!(HueAngle::<f64>::from(RGB::<f64>::MAGENTA)
            .angle
            .approx_eq(HueAngle::<f64>::MAGENTA_ANGLE));
        assert!(HueAngle::<f64>::from(RGB::<f64>::YELLOW)
            .angle
            .approx_eq(HueAngle::<f64>::YELLOW_ANGLE));
        assert!(HueAngle::<f64>::from(RGB::<f64>::BLACK).angle.is_nan());
        assert!(HueAngle::<f64>::from(RGB::<f64>::WHITE).angle.is_nan());
    }

    #[test]
    fn rotation() {
        assert!(
            (HueAngle::<f64>::from(HueAngle::<f64>::YELLOW_ANGLE) + Angle::from_degrees(60.0))
                .angle
                .approx_eq(HueAngle::<f64>::GREEN_ANGLE)
        );
        assert!(
            (HueAngle::<f64>::from(HueAngle::<f64>::MAGENTA_ANGLE) - Angle::from_degrees(60.0))
                .angle
                .approx_eq(HueAngle::<f64>::BLUE_ANGLE)
        )
    }

    #[test]
    fn difference() {
        assert!((HueAngle::<f64>::from(HueAngle::<f64>::YELLOW_ANGLE)
            - HueAngle::from(HueAngle::<f64>::GREEN_ANGLE))
        .approx_eq(Angle::from_degrees(-60.0)));
        assert!((HueAngle::<f64>::from(HueAngle::<f64>::YELLOW_ANGLE)
            - HueAngle::from(HueAngle::<f64>::MAGENTA_ANGLE))
        .approx_eq(Angle::from_degrees(120.0)));
    }

    #[test]
    fn rgb_range_with_chroma() {
        for angle in TEST_ANGLES.iter() {
            let hue_angle: HueAngle<f64> = (*angle).into();
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
                assert!(angle.approx_eq(HueAngle::from(shade_rgb).angle));
                assert!(angle.approx_eq(HueAngle::from(tint_rgb).angle));
            }
        }
        let hue_angle = HueAngle::<f64>::from(Angle::<f64>::from(std::f64::NAN));
        assert_eq!(
            hue_angle.rgb_range_with_chroma(0.0),
            Some((RGB::BLACK, RGB::WHITE))
        );
        for chroma in TEST_RATIOS.iter() {
            assert!(hue_angle.rgb_range_with_chroma(*chroma).is_none())
        }
    }
}
