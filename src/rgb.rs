// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    convert::{From, TryFrom},
    ops::Index,
    str::FromStr,
};

use regex::Regex;

pub use crate::{chroma, hue::*, ColourComponent, ColourInterface, I_BLUE, I_GREEN, I_RED};

use float_plus::*;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct RGB<F: ColourComponent>([F; 3]);

impl<F: ColourComponent> RGB<F> {
    pub const RED: Self = Self([F::ONE, F::ZERO, F::ZERO]);
    pub const GREEN: Self = Self([F::ZERO, F::ONE, F::ZERO]);
    pub const BLUE: Self = Self([F::ZERO, F::ZERO, F::ONE]);

    pub const CYAN: Self = Self([F::ZERO, F::ONE, F::ONE]);
    pub const MAGENTA: Self = Self([F::ONE, F::ZERO, F::ONE]);
    pub const YELLOW: Self = Self([F::ONE, F::ONE, F::ZERO]);

    pub const WHITE: Self = Self([F::ONE, F::ONE, F::ONE]);
    pub const BLACK: Self = Self([F::ZERO, F::ZERO, F::ZERO]);

    pub const PRIMARIES: [Self; 3] = [Self::RED, Self::GREEN, Self::BLUE];
    pub const SECONDARIES: [Self; 3] = [Self::CYAN, Self::MAGENTA, Self::YELLOW];
    pub const GREYS: [Self; 2] = [Self::BLACK, Self::WHITE];

    pub fn raw(self) -> [F; 3] {
        self.0
    }

    pub(crate) fn sum(self) -> F {
        self.0[I_RED] + self.0[I_GREEN] + self.0[I_BLUE]
    }

    pub(crate) fn x(self) -> F {
        self.0[I_RED] + (self.0[I_GREEN] + self.0[I_BLUE]) * F::COS_120
    }

    pub(crate) fn y(self) -> F {
        (self.0[I_GREEN] - self.0[I_BLUE]) * F::SIN_120
    }

    pub(crate) fn xy(self) -> (F, F) {
        (self.x(), self.y())
    }

    pub(crate) fn hypot(self) -> F {
        self.x().hypot(self.y())
    }

    pub(crate) fn indices_value_order(self) -> [usize; 3] {
        if self[I_RED] >= self[I_GREEN] {
            if self[I_RED] >= self[I_BLUE] {
                if self[I_GREEN] >= self[I_BLUE] {
                    [I_RED, I_GREEN, I_BLUE]
                } else {
                    [I_RED, I_BLUE, I_GREEN]
                }
            } else {
                [I_BLUE, I_RED, I_GREEN]
            }
        } else if self[I_GREEN] >= self[I_BLUE] {
            if self[I_RED] >= self[I_BLUE] {
                [I_GREEN, I_RED, I_BLUE]
            } else {
                [I_GREEN, I_BLUE, I_RED]
            }
        } else {
            [I_BLUE, I_GREEN, I_RED]
        }
    }

    pub(crate) fn indices_value_order_u8(self) -> [u8; 3] {
        if self[I_RED] >= self[I_GREEN] {
            if self[I_RED] >= self[I_BLUE] {
                if self[I_GREEN] >= self[I_BLUE] {
                    [I_RED as u8, I_GREEN as u8, I_BLUE as u8]
                } else {
                    [I_RED as u8, I_BLUE as u8, I_GREEN as u8]
                }
            } else {
                [I_BLUE as u8, I_RED as u8, I_GREEN as u8]
            }
        } else if self[I_GREEN] >= self[I_BLUE] {
            if self[I_RED] >= self[I_BLUE] {
                [I_GREEN as u8, I_RED as u8, I_BLUE as u8]
            } else {
                [I_GREEN as u8, I_BLUE as u8, I_RED as u8]
            }
        } else {
            [I_BLUE as u8, I_GREEN as u8, I_RED as u8]
        }
    }
}

impl<F: ColourComponent + std::fmt::Debug + std::iter::Sum> FloatApproxEq<F> for RGB<F> {
    fn abs_diff(&self, other: &Self) -> F {
        let sum: F = self
            .0
            .iter()
            .zip(other.0.iter())
            .map(|(a, b)| (*a - *b).powi(2))
            .sum();
        sum.sqrt() / F::THREE
    }

    fn rel_diff_scale_factor(&self, other: &Self) -> F {
        self.value().max(other.value())
    }
}

impl<F: ColourComponent> Index<usize> for RGB<F> {
    type Output = F;

    fn index(&self, index: usize) -> &F {
        &self.0[index]
    }
}

impl<F: ColourComponent> From<[F; 3]> for RGB<F> {
    fn from(array: [F; 3]) -> Self {
        debug_assert!(array.iter().all(|x| (*x).is_proportion()), "{:?}", array);
        Self(array)
    }
}

impl<F: ColourComponent> ColourInterface<F> for RGB<F> {
    fn rgb(&self) -> [F; 3] {
        self.0
    }

    fn rgba(&self, alpha: F) -> [F; 4] {
        debug_assert!(alpha.is_proportion());
        [self.0[I_RED], self.0[I_GREEN], self.0[I_BLUE], alpha]
    }

    fn hue(&self) -> Option<Hue<F>> {
        use std::convert::TryInto;
        if let Ok(hue) = (*self).try_into() {
            Some(hue)
        } else {
            None
        }
    }

    fn is_grey(&self) -> bool {
        self.hypot() == F::ZERO
    }

    fn chroma(&self) -> F {
        let xy = self.xy();
        let hypot = xy.0.hypot(xy.1);
        if hypot == F::ZERO {
            F::ZERO
        } else {
            let second = chroma::calc_other_from_xy(xy);
            (hypot * chroma::calc_chroma_correction(second)).min(F::ONE)
        }
    }

    fn max_chroma_rgb(&self) -> RGB<F> {
        let xy = self.xy();
        if xy.0 == F::ZERO && xy.1 == F::ZERO {
            Self::WHITE
        } else {
            let io = self.indices_value_order();
            let mut array: [F; 3] = [F::ZERO, F::ZERO, F::ZERO];
            array[io[0]] = F::ONE;
            array[io[1]] = chroma::calc_other_from_xy_alt(xy);
            array.into()
        }
    }

    fn greyness(&self) -> F {
        let xy = self.xy();
        let hypot = xy.0.hypot(xy.1);
        if hypot == F::ZERO {
            F::ONE
        } else {
            let second = chroma::calc_other_from_xy(xy);
            (F::ONE - hypot * chroma::calc_chroma_correction(second)).max(F::ZERO)
        }
    }

    fn value(&self) -> F {
        ((self.0[I_RED] + self.0[I_GREEN] + self.0[I_BLUE]) / F::THREE).min(F::ONE)
    }

    fn monotone_rgb(&self) -> RGB<F> {
        let value = self.value();
        [value, value, value].into()
    }

    fn warmth(&self) -> F {
        ((self.x() + F::ONE).max(F::ZERO) / F::TWO).min(F::ONE)
    }

    fn warmth_rgb(&self) -> RGB<F> {
        let x = self.x();
        let half = F::HALF;
        if x < F::ZERO {
            let temp = x.abs() + (F::ONE + x) * half;
            [F::ZERO, temp, temp].into()
        } else if x > F::ZERO {
            [x + (F::ONE - x) * half, F::ZERO, F::ZERO].into()
        } else {
            [half, half, half].into()
        }
    }

    fn best_foreground_rgb(&self) -> Self {
        if self[I_RED] * F::from(0.299).unwrap()
            + self[I_GREEN] * F::from(0.587).unwrap()
            + self[I_BLUE] * F::from(0.114).unwrap()
            > F::HALF
        {
            Self::BLACK
        } else {
            Self::WHITE
        }
    }
}

#[derive(Debug)]
pub enum RGBError {
    MalformedText(String),
}

impl std::fmt::Display for RGBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RGBError::MalformedText(string) => write!(f, "Malformed text: {}", string),
        }
    }
}

impl std::error::Error for RGBError {}

impl From<std::num::ParseIntError> for RGBError {
    fn from(error: std::num::ParseIntError) -> Self {
        RGBError::MalformedText(format!("{}", error))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct RGB16([u16; 3]);

lazy_static! {
    pub static ref RGB16_RE: Regex = Regex::new(
        r#"RGB(16)?\((red=)?0x(?P<red>[a-fA-F0-9]+), (green=)?0x(?P<green>[a-fA-F0-9]+), (blue=)?0x(?P<blue>[a-fA-F0-9]+)\)"#
    ).unwrap();
    pub static ref RGB16_BASE_10_RE: Regex = Regex::new(
        r#"RGB(16)?\((red=)?(?P<red>\d+), (green=)?(?P<green>\d+), (blue=)?(?P<blue>\d+)\)"#
    ).unwrap();
}

impl FromStr for RGB16 {
    type Err = RGBError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let Some(captures) = RGB16_RE.captures(string) {
            let red = u16::from_str_radix(captures.name("red").unwrap().as_str(), 16)?;
            let green = u16::from_str_radix(captures.name("green").unwrap().as_str(), 16)?;
            let blue = u16::from_str_radix(captures.name("blue").unwrap().as_str(), 16)?;
            Ok(RGB16([red, green, blue]))
        } else if let Some(captures) = RGB16_BASE_10_RE.captures(string) {
            let red = u16::from_str_radix(captures.name("red").unwrap().as_str(), 10)?;
            let green = u16::from_str_radix(captures.name("green").unwrap().as_str(), 10)?;
            let blue = u16::from_str_radix(captures.name("blue").unwrap().as_str(), 10)?;
            Ok(RGB16([red, green, blue]))
        } else {
            Err(RGBError::MalformedText(string.to_string()))
        }
    }
}

impl<F: ColourComponent> From<RGB<F>> for RGB16 {
    fn from(rgb: RGB<F>) -> Self {
        let scale: F = F::from_u16(0xFFFF).unwrap();
        let red: u16 = (rgb.0[0] * scale).round().to_u16().unwrap();
        let green: u16 = (rgb.0[1] * scale).round().to_u16().unwrap();
        let blue: u16 = (rgb.0[2] * scale).round().to_u16().unwrap();
        Self([red, green, blue])
    }
}

impl<F: ColourComponent> From<RGB16> for RGB<F> {
    fn from(rgb: RGB16) -> Self {
        let scale: F = F::from_u16(0xFFFF).unwrap();
        let red: F = F::from_u16(rgb.0[0]).unwrap() / scale;
        let green: F = F::from_u16(rgb.0[1]).unwrap() / scale;
        let blue: F = F::from_u16(rgb.0[2]).unwrap() / scale;
        Self([red, green, blue])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb16_to_and_from_rgb() {
        assert_eq!(RGB16([0xffff, 0xffff, 0x0]), RGB::<f64>::YELLOW.into());
        assert_eq!(RGB::<f32>::CYAN, RGB16([0, 0xffff, 0xffff]).into());
    }

    #[test]
    fn rgb16_from_str() {
        assert_eq!(
            RGB16::from_str("RGB16(red=0xF800, green=0xFA00, blue=0xF600)").unwrap(),
            RGB16([0xF800, 0xFA00, 0xF600])
        );
        assert_eq!(
            RGB16::from_str("RGB16(0xF800, 0xFA00, 0xF600)").unwrap(),
            RGB16([0xF800, 0xFA00, 0xF600])
        );
        assert_eq!(
            RGB16::from_str("RGB16(red=78, green=2345, blue=5678)").unwrap(),
            RGB16([78, 2345, 5678])
        );
        assert_eq!(
            RGB16::from_str("RGB16(128, 45670, 600)").unwrap(),
            RGB16([128, 45670, 600])
        );
    }

    #[test]
    fn indices_order() {
        assert_eq!(
            RGB::<f64>::WHITE.indices_value_order(),
            [I_RED, I_GREEN, I_BLUE]
        );
        assert_eq!(
            RGB::<f64>::BLACK.indices_value_order(),
            [I_RED, I_GREEN, I_BLUE]
        );
        assert_eq!(
            RGB::<f64>::RED.indices_value_order(),
            [I_RED, I_GREEN, I_BLUE]
        );
        assert_eq!(
            RGB::<f64>::GREEN.indices_value_order(),
            [I_GREEN, I_RED, I_BLUE]
        );
        assert_eq!(
            RGB::<f64>::BLUE.indices_value_order(),
            [I_BLUE, I_RED, I_GREEN]
        );
        assert_eq!(
            RGB::<f64>::CYAN.indices_value_order(),
            [I_GREEN, I_BLUE, I_RED]
        );
        assert_eq!(
            RGB::<f64>::MAGENTA.indices_value_order(),
            [I_RED, I_BLUE, I_GREEN]
        );
        assert_eq!(
            RGB::<f64>::YELLOW.indices_value_order(),
            [I_RED, I_GREEN, I_BLUE]
        );
    }
}
