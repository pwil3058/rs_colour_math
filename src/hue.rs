// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use normalised_angles::{Angle, AngleConst};
use num::traits::{Float, NumAssign, NumOps};

use crate::rgb::{ZeroOneEtc, RGB};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct HueAngle<F: Float + NumAssign + NumOps + AngleConst + Copy + ZeroOneEtc> {
    angle: Angle<F>,
    max_chroma_rgb: RGB<F>,
    chroma_correction: F,
}

impl<F: Float + NumAssign + NumOps + AngleConst + Copy + ZeroOneEtc> HueAngle<F> {
    pub fn calc_other(abs_angle: Angle<F>) -> F {
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
