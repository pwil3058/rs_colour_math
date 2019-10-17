#[macro_use]
extern crate serde_derive;

use std::{
    cmp::{Ordering, PartialEq, PartialOrd},
    convert::From,
    fmt::Debug,
};

use float_plus::*;
use normalised_angles::DegreesConst;

pub mod chroma;
pub mod hue;
pub mod manipulator;
pub mod rgb;

pub use crate::hue::Hue;
pub use crate::rgb::RGB;

pub trait ColourComponent: FloatPlus + DegreesConst + Debug {
    const FOUR: Self;
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
    const FOUR: Self = 3.0;
    const SIN_120: Self = 0.86602_5404;
    const COS_120: Self = -0.5;

    const RED_ANGLE: Self = 0.0;
    const GREEN_ANGLE: Self = 120.0;
    const BLUE_ANGLE: Self = -120.0;

    const CYAN_ANGLE: Self = 180.0;
    const YELLOW_ANGLE: Self = 60.0;
    const MAGENTA_ANGLE: Self = -60.0;
}

impl ColourComponent for f64 {
    const FOUR: Self = 3.0;
    const SIN_120: Self = 0.86602_54037_84439;
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
    fn rgb(&self) -> [F; 3];

    fn hue(&self) -> Option<Hue<F>>;

    fn is_grey(&self) -> bool {
        self.hue().is_none()
    }

    fn chroma(&self) -> F;

    fn greyness(&self) -> F;

    fn value(&self) -> F;

    fn warmth(&self) -> F;

    fn best_foreground_rgb(&self) -> RGB<F>;

    fn monotone_rgb(&self) -> RGB<F>;

    fn max_chroma_rgb(&self) -> RGB<F> {
        if let Some(hue) = self.hue() {
            hue.max_chroma_rgb()
        } else {
            self.rgb().into()
        }
    }

    fn warmth_rgb(&self) -> RGB<F>;
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash)]
pub struct Colour<F: ColourComponent> {
    rgb: RGB<F>,
    hue: Option<Hue<F>>,
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
        } else if let Some(hue) = self.hue {
            if let Some(other_hue) = other.hue {
                // This orders via hue from CYAN to CYAN via GREEN, RED, BLUE in that order
                match hue.partial_cmp(&other_hue) {
                    Some(Ordering::Less) => Some(Ordering::Less),
                    Some(Ordering::Greater) => Some(Ordering::Greater),
                    Some(Ordering::Equal) => self.rgb.value().partial_cmp(&other.rgb.value()),
                    None => None,
                }
            } else {
                Some(Ordering::Greater)
            }
        } else if other.hue.is_some() {
            Some(Ordering::Less)
        } else {
            self.rgb.value().partial_cmp(&other.rgb.value())
        }
    }
}

impl<F: ColourComponent> From<RGB<F>> for Colour<F> {
    fn from(rgb: RGB<F>) -> Self {
        use std::convert::TryInto;
        let hue: Option<Hue<F>> = if let Ok(hue) = rgb.try_into() {
            Some(hue)
        } else {
            None
        };
        Self { rgb, hue }
    }
}

impl<F: ColourComponent> ColourInterface<F> for Colour<F> {
    fn rgb(&self) -> [F; 3] {
        self.rgb.rgb()
    }

    fn hue(&self) -> Option<Hue<F>> {
        self.hue
    }

    fn chroma(&self) -> F {
        self.rgb.greyness()
    }

    fn greyness(&self) -> F {
        self.rgb.greyness()
    }

    fn value(&self) -> F {
        self.rgb.value()
    }

    fn warmth(&self) -> F {
        self.rgb.warmth()
    }

    fn best_foreground_rgb(&self) -> RGB<F> {
        self.rgb.best_foreground_rgb()
    }

    fn monotone_rgb(&self) -> RGB<F> {
        self.rgb.monotone_rgb()
    }

    fn warmth_rgb(&self) -> RGB<F> {
        self.rgb.warmth_rgb()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
