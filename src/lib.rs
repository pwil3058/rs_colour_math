#[macro_use]
extern crate serde_derive;

use num::traits::{Float, NumAssign, NumOps};

use normalised_angles::AngleConst;

pub mod hue;
pub mod rgb;

pub trait ColourComponent: Float + PartialOrd + Copy + NumAssign + NumOps + AngleConst {
    const ZERO: Self;
    const ONE: Self;
    const TWO: Self;
    const THREE: Self;
    const SIN_120: Self;
    const COS_120: Self;

    const RED_ANGLE: Self;
    const GREEN_ANGLE: Self;
    const BLUE_ANGLE: Self;

    const CYAN_ANGLE: Self;
    const YELLOW_ANGLE: Self;
    const MAGENTA_ANGLE: Self;

    fn is_proportion(self) -> bool {
        self <= Self::ONE && self >= Self::ZERO
    }
}

impl ColourComponent for f32 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const TWO: Self = 2.0;
    const THREE: Self = 3.0;
    const SIN_120: Self = 0.86602_54037_844387;
    const COS_120: Self = -0.5;

    const RED_ANGLE: Self = 0.0;
    const GREEN_ANGLE: Self = 120.0;
    const BLUE_ANGLE: Self = -120.0;

    const CYAN_ANGLE: Self = 180.0;
    const YELLOW_ANGLE: Self = 60.0;
    const MAGENTA_ANGLE: Self = -60.0;
}

impl ColourComponent for f64 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const TWO: Self = 2.0;
    const THREE: Self = 3.0;
    const SIN_120: Self = 0.86602_54037_844387;
    const COS_120: Self = -0.5;

    const RED_ANGLE: Self = 0.0;
    const GREEN_ANGLE: Self = 120.0;
    const BLUE_ANGLE: Self = -120.0;

    const CYAN_ANGLE: Self = 180.0;
    const YELLOW_ANGLE: Self = 60.0;
    const MAGENTA_ANGLE: Self = -60.0;
}

pub const I_RED: usize = 0;
pub const I_GREEN: usize = 1;
pub const I_BLUE: usize = 2;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
