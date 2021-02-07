// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
#[macro_use]
extern crate serde_derive;

pub mod hue;
pub mod proportion;
pub mod rgb;

use normalised_angles::Degrees;

use crate::proportion::Float;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CCI {
    Red,
    Green,
    Blue,
}

pub trait HueConstants: Sized + Copy {
    const RED: Self;
    const GREEN: Self;
    const BLUE: Self;

    const CYAN: Self;
    const MAGENTA: Self;
    const YELLOW: Self;

    const PRIMARIES: [Self; 3] = [Self::RED, Self::GREEN, Self::BLUE];
    const SECONDARIES: [Self; 3] = [Self::CYAN, Self::MAGENTA, Self::YELLOW];
}

pub trait RGBConstants: HueConstants + Copy {
    const WHITE: Self;
    const BLACK: Self;

    const GREYS: [Self; 2] = [Self::BLACK, Self::WHITE];
}

impl<F: Float> HueConstants for Degrees<F> {
    const RED: Self = Self::DEG_0;
    const GREEN: Self = Self::DEG_120;
    const BLUE: Self = Self::NEG_DEG_120;

    const CYAN: Self = Self::DEG_180;
    const MAGENTA: Self = Self::NEG_DEG_60;
    const YELLOW: Self = Self::DEG_60;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
