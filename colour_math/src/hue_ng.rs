// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    cmp::Ordering,
    convert::{From, Into, TryFrom},
    ops::{Add, Index, Mul},
};

pub use crate::{
    chroma, hcv::*, rgb::RGB, urgb::URGB, ColourComponent, ColourInterface, HueConstants,
    RGBConstants, CCI,
};

use crate::chroma::HueData;
use crate::hue_ng::Sextant::{BlueMagenta, GreenYellow, RedMagenta, RedYellow};
use crate::HueIfce;
use normalised_angles::Degrees;
use num_traits_plus::float_plus::*;
use regex::Error;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord)]
pub enum RGBHue {
    Red = 5,
    Green = 9,
    Blue = 1,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord)]
pub enum CMYHue {
    Cyan = 11,
    Magenta = 3,
    Yellow = 7,
}

impl CMYHue {
    pub fn indices(&self) -> (CCI, CCI) {
        match self {
            CMYHue::Magenta => (CCI::Red, CCI::Blue),
            CMYHue::Yellow => (CCI::Red, CCI::Green),
            CMYHue::Cyan => (CCI::Green, CCI::Blue),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Sextant {
    RedMagenta = 4,
    RedYellow = 6,
    GreenYellow = 8,
    GreenCyan = 10,
    BlueCyan = 0,
    BlueMagenta = 2,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Hue<F: ColourComponent> {
    Primary(RGBHue),
    Secondary(CMYHue),
    Other(Sextant, F),
}

impl<F: ColourComponent> Eq for Hue<F> {}

impl<F: ColourComponent> HueConstants for Hue<F> {
    const RED: Self = Self::Primary(RGBHue::Red);
    const GREEN: Self = Self::Primary(RGBHue::Green);
    const BLUE: Self = Self::Primary(RGBHue::Blue);

    const CYAN: Self = Self::Secondary(CMYHue::Cyan);
    const MAGENTA: Self = Self::Secondary(CMYHue::Magenta);
    const YELLOW: Self = Self::Secondary(CMYHue::Yellow);
}

impl<F: ColourComponent> TryFrom<&RGB<F>> for Hue<F> {
    type Error = &'static str;

    fn try_from(rgb: &RGB<F>) -> Result<Self, Self::Error> {
        match rgb[CCI::Red].partial_cmp(&rgb[CCI::Green]).unwrap() {
            Ordering::Greater => match rgb[CCI::Green].partial_cmp(&rgb[CCI::Blue]).unwrap() {
                Ordering::Greater => Ok(Hue::Other(
                    Sextant::RedYellow,
                    (rgb[CCI::Green] - rgb[CCI::Blue]) / rgb[CCI::Red],
                )),
                Ordering::Less => match rgb[CCI::Red].partial_cmp(&rgb[CCI::Blue]).unwrap() {
                    Ordering::Greater => Ok(Hue::Other(
                        Sextant::RedMagenta,
                        (rgb[CCI::Blue] - rgb[CCI::Green]) / rgb[CCI::Red],
                    )),
                    Ordering::Less => Ok(Hue::Other(
                        Sextant::BlueMagenta,
                        (rgb[CCI::Red] - rgb[CCI::Green]) / rgb[CCI::Blue],
                    )),
                    Ordering::Equal => Ok(Hue::Secondary(CMYHue::Magenta)),
                },
                Ordering::Equal => Ok(Hue::Primary(RGBHue::Red)),
            },
            Ordering::Less => match rgb[CCI::Red].partial_cmp(&rgb[CCI::Blue]).unwrap() {
                Ordering::Greater => Ok(Hue::Other(
                    Sextant::GreenYellow,
                    (rgb[CCI::Red] - rgb[CCI::Blue]) / rgb[CCI::Green],
                )),
                Ordering::Less => match rgb[CCI::Green].partial_cmp(&rgb[CCI::Blue]).unwrap() {
                    Ordering::Greater => Ok(Hue::Other(
                        Sextant::GreenCyan,
                        (rgb[CCI::Blue] - rgb[CCI::Red]) / rgb[CCI::Green],
                    )),
                    Ordering::Less => Ok(Hue::Other(
                        Sextant::BlueCyan,
                        (rgb[CCI::Green] - rgb[CCI::Red]) / rgb[CCI::Blue],
                    )),
                    Ordering::Equal => Ok(Hue::Secondary(CMYHue::Cyan)),
                },
                Ordering::Equal => Ok(Hue::Primary(RGBHue::Green)),
            },
            Ordering::Equal => match rgb[CCI::Red].partial_cmp(&rgb[CCI::Blue]).unwrap() {
                Ordering::Greater => Ok(Hue::Secondary(CMYHue::Yellow)),
                Ordering::Less => Ok(Hue::Primary(RGBHue::Blue)),
                Ordering::Equal => Err("RGB is grey and hs no hue"),
            },
        }
    }
}

impl<F: ColourComponent> Hue<F> {
    pub fn ord_index(&self) -> u8 {
        0
    }
}

impl<F: ColourComponent> PartialOrd for Hue<F> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.ord_index().partial_cmp(&other.ord_index())
    }
}

impl<F: ColourComponent> Ord for Hue<F> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[cfg(test)]
mod hue_ng_tests {
    use super::*;

    #[test]
    fn hue_from_rgb() {
        for rgb in &[RGB::<f64>::BLACK, RGB::WHITE, RGB::from([0.5, 0.5, 0.5])] {
            assert!(Hue::<f64>::try_from(rgb).is_err());
        }
        for (rgb, hue) in RGB::<f64>::PRIMARIES.iter().zip(Hue::PRIMARIES.iter()) {
            assert_eq!(Hue::<f64>::try_from(rgb), Ok(*hue));
        }
        for (rgb, hue) in RGB::<f64>::SECONDARIES.iter().zip(Hue::SECONDARIES.iter()) {
            assert_eq!(Hue::<f64>::try_from(rgb), Ok(*hue));
        }
        for (rgb, hue) in &[
            (
                RGB::<f64>::from([1.0, 0.5, 0.0]),
                Hue::Other(Sextant::RedYellow, 0.5),
            ),
            (
                RGB::<f64>::from([0.0, 0.25, 0.5]),
                Hue::Other(Sextant::BlueCyan, 0.5),
            ),
        ] {
            assert_eq!(Hue::<f64>::try_from(rgb), Ok(*hue));
        }
    }
}
