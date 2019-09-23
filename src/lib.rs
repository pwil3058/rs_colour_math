#[macro_use]
extern crate serde_derive;

use std::{
    cmp::{Ordering, PartialEq, PartialOrd},
    convert::From,
};

use num::traits::{Float, NumAssign, NumOps};

use normalised_angles::AngleConst;

pub mod hue;
pub mod rgb;

pub use crate::hue::Hue;
pub use crate::rgb::RGB;

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

pub trait ColourInterface<F: ColourComponent> {
    fn rgb(&self) -> RGB<F>;

    fn hue(&self) -> Hue<F>;

    fn is_grey(&self) -> bool {
        self.hue().is_grey()
    }

    fn chroma(&self) -> F {
        // Be paranoid about fact floats only approximate real numbers
        (self.rgb().hypot() * self.hue().chroma_correction()).min(F::ONE)
    }

    fn greyness(&self) -> F {
        // Be paranoid about fact floats only approximate real numbers
        (F::ONE - self.rgb().hypot() * self.hue().chroma_correction()).max(F::ZERO)
    }

    fn value(&self) -> F {
        self.rgb().value()
    }

    fn warmth(&self) -> F {
        (self.rgb().x() + F::ONE) / F::TWO
    }

    fn best_foreground_rgb(&self) -> RGB<F> {
        self.rgb().best_foreground_rgb()
    }

    fn monotone_rgb(&self) -> RGB<F> {
        let value = self.rgb().value();
        [value, value, value].into()
    }

    fn max_chroma_rgb(&self) -> RGB<F> {
        self.hue().max_chroma_rgb()
    }

    fn warmth_rgb(&self) -> RGB<F> {
        let x = self.rgb().x();
        let half = F::from(0.5).unwrap();
        if x < F::ZERO {
            let temp = x.abs() + (F::ONE + x) * half;
            [F::ZERO, temp, temp].into()
        } else if x > F::ZERO {
            [x + (F::ONE - x) * half, F::ZERO, F::ZERO].into()
        } else {
            [half, half, half].into()
        }
    }
}

impl<F: ColourComponent> ColourInterface<F> for RGB<F> {
    fn rgb(&self) -> RGB<F> {
        *self
    }

    fn hue(&self) -> Hue<F> {
        (*self).into()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash)]
pub struct Colour<F: ColourComponent> {
    rgb: RGB<F>,
    hue: Hue<F>,
}

impl<F: ColourComponent> PartialEq for Colour<F> {
    fn eq(&self, other: &Self) -> bool {
        self.rgb == other.rgb
    }
}

impl<F: ColourComponent> Eq for Colour<F> {}

impl<F: ColourComponent> PartialOrd for Colour<F> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.rgb == other.rgb {
            Some(Ordering::Equal)
        } else if self.hue.is_grey() {
            if other.hue.is_grey() {
                self.rgb.value().partial_cmp(&other.rgb.value())
            } else {
                Some(Ordering::Less)
            }
        } else if other.hue.is_grey() {
            Some(Ordering::Greater)
        } else {
            // This orders via hue from CYAN to CYAN via GREEN, RED, BLUE in that order
            self.hue
                .angle()
                .radians()
                .partial_cmp(&other.hue.angle().radians())
        }
    }
}

impl<F: ColourComponent> From<RGB<F>> for Colour<F> {
    fn from(rgb: RGB<F>) -> Self {
        Self {
            rgb,
            hue: rgb.into(),
        }
    }
}

impl<F: ColourComponent> ColourInterface<F> for Colour<F> {
    fn rgb(&self) -> RGB<F> {
        self.rgb
    }

    fn hue(&self) -> Hue<F> {
        self.hue
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
