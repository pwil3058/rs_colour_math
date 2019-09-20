// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::convert::From;

use normalised_angles::{Angle, AngleConst};
use num::traits::{Float, NumAssign, NumOps};

use crate::rgb::{ZeroOneEtc, RGB};

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
                } else if angle >= Self::CYAN_ANGLE {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_angle() {
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
}
