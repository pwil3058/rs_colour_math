// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    convert::From,
    ops::{Add, Index, Mul},
    str::FromStr,
};

use regex::Regex;

pub use crate::{chroma, hue::*, ColourComponent, ColourInterface, I_BLUE, I_GREEN, I_RED};

use float_plus::*;
use normalised_angles::Degrees;

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
        //self.0[I_RED] + self.0[I_GREEN] + self.0[I_BLUE]
        self.0.iter().map(|x| *x).sum()
    }

    pub(crate) fn x(self) -> F {
        self[I_RED] + (self[I_GREEN] + self[I_BLUE]) * F::COS_120
    }

    pub(crate) fn y(self) -> F {
        (self[I_GREEN] - self[I_BLUE]) * F::SIN_120
    }

    pub(crate) fn xy(self) -> (F, F) {
        (self.x(), self.y())
    }

    pub(crate) fn hypot(self) -> F {
        self.x().hypot(self.y())
    }

    pub(crate) fn indices_value_order(self) -> [u8; 3] {
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

    fn ff(&self, indices: (u8, u8), ks: (F, F)) -> F {
        self[indices.0] * ks.0 + self[indices.1] * ks.1
    }

    //Return a copy of the rgb with each component rotated by the specified
    //angle. This results in an rgb the same value but the hue angle rotated
    //by the specified amount.
    //NB the chroma will change when there are less than 3 non zero
    //components and in the case of 2 non zero components this change may
    //be undesirable.  If it is undesirable it can be avoided by using a
    //higher level wrapper function to adjust/restore the chroma value.
    //In some cases maintaining bof chroma and value will not be
    //possible due to the complex relationship between value and chroma.
    pub fn components_rotated(&self, delta_hue_angle: Degrees<F>) -> RGB<F> {
        fn calc_ks<F: ColourComponent>(delta_hue_angle: Degrees<F>) -> (F, F) {
            let a = delta_hue_angle.sin();
            let b = (Degrees::DEG_120 - delta_hue_angle).sin();
            let c = a + b;
            (b / c, a / c)
        }
        if delta_hue_angle > Degrees::DEG_0 {
            if delta_hue_angle > Degrees::DEG_120 {
                let ks = calc_ks(delta_hue_angle - Degrees::DEG_120);
                return RGB([
                    self.ff((2, 1), ks),
                    self.ff((0, 2), ks),
                    self.ff((1, 0), ks),
                ]);
            } else {
                let ks = calc_ks(delta_hue_angle);
                return RGB([
                    self.ff((0, 2), ks),
                    self.ff((1, 0), ks),
                    self.ff((2, 1), ks),
                ]);
            }
        } else if delta_hue_angle < Degrees::DEG_0 {
            if delta_hue_angle < -Degrees::DEG_120 {
                let ks = calc_ks(delta_hue_angle.abs() - Degrees::DEG_120);
                return RGB([
                    self.ff((1, 2), ks),
                    self.ff((2, 0), ks),
                    self.ff((0, 1), ks),
                ]);
            } else {
                let ks = calc_ks(delta_hue_angle.abs());
                return RGB([
                    self.ff((0, 1), ks),
                    self.ff((1, 2), ks),
                    self.ff((2, 0), ks),
                ]);
            }
        }
        *self
    }

    pub fn pango_string(&self) -> String {
        RGB8::from(*self).pango_string()
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

impl<F: ColourComponent> Index<u8> for RGB<F> {
    type Output = F;

    fn index(&self, index: u8) -> &F {
        &self.0[index as usize]
    }
}

impl<F: ColourComponent> Add for RGB<F> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let array: [F; 3] = [
            self.0[0] + other.0[0],
            self.0[1] + other.0[1],
            self.0[2] + other.0[2],
        ];
        array.into()
    }
}

impl<F: ColourComponent> Mul<F> for RGB<F> {
    type Output = Self;

    fn mul(self, scalar: F) -> Self {
        let array: [F; 3] = [self.0[0] * scalar, self.0[1] * scalar, self.0[2] * scalar];
        array.into()
    }
}

impl<F: ColourComponent> From<[F; 3]> for RGB<F> {
    fn from(array: [F; 3]) -> Self {
        debug_assert!(array.iter().all(|x| (*x).is_proportion()), "{:?}", array);
        Self(array)
    }
}

impl<F: ColourComponent> From<&[u8]> for RGB<F> {
    fn from(array: &[u8]) -> Self {
        debug_assert_eq!(array.len(), 3);
        let divisor = F::from(255.0).unwrap();
        Self([
            F::from_u8(array[0]).unwrap() / divisor,
            F::from_u8(array[1]).unwrap() / divisor,
            F::from_u8(array[2]).unwrap() / divisor,
        ])
    }
}

impl<F: ColourComponent> ColourInterface<F> for RGB<F> {
    fn rgb(&self) -> RGB<F> {
        *self
    }

    fn rgba(&self, alpha: F) -> [F; 4] {
        debug_assert!(alpha.is_proportion());
        [self.0[0], self.0[1], self.0[2], alpha]
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
            array[io[0] as usize] = F::ONE;
            array[io[1] as usize] = chroma::calc_other_from_xy_alt(xy);
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
        (self.sum() / F::THREE).min(F::ONE)
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
        if x < F::ZERO {
            let temp = x.abs() + (F::ONE + x) * F::HALF;
            [F::ZERO, temp, temp].into()
        } else if x > F::ZERO {
            [x + (F::ONE - x) * F::HALF, F::ZERO, F::ZERO].into()
        } else {
            [F::HALF, F::HALF, F::HALF].into()
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

impl Index<usize> for RGB16 {
    type Output = u16;

    fn index(&self, index: usize) -> &u16 {
        &self.0[index]
    }
}

impl RGB16 {
    pub const RED: Self = Self([0xFFFF, 0x0000, 0x0000]);
    pub const GREEN: Self = Self([0x0000, 0xFFFF, 0x0000]);
    pub const BLUE: Self = Self([0x0000, 0x0000, 0xFFFF]);

    pub const CYAN: Self = Self([0x0000, 0xFFFF, 0xFFFF]);
    pub const MAGENTA: Self = Self([0xFFFF, 0x0000, 0xFFFF]);
    pub const YELLOW: Self = Self([0xFFFF, 0xFFFF, 0x0000]);

    pub const WHITE: Self = Self([0xFFFF, 0xFFFF, 0xFFFF]);
    pub const BLACK: Self = Self([0x0000, 0x0000, 0x0000]);

    pub const PRIMARIES: [Self; 3] = [Self::RED, Self::GREEN, Self::BLUE];
    pub const SECONDARIES: [Self; 3] = [Self::CYAN, Self::MAGENTA, Self::YELLOW];
    pub const GREYS: [Self; 2] = [Self::BLACK, Self::WHITE];
}

lazy_static! {
    pub static ref RGB16_RE: Regex = Regex::new(
        r#"RGB(16)?\((red=)?0x(?P<red>[a-fA-F0-9]+), (green=)?0x(?P<green>[a-fA-F0-9]+), (blue=)?0x(?P<blue>[a-fA-F0-9]+)\)"#
    ).unwrap();
    pub static ref RGB16_BASE_10_RE: Regex = Regex::new(
        r#"RGB(16)?\((red=)?(?P<red>\d+), (green=)?(?P<green>\d+), (blue=)?(?P<blue>\d+)\)"#
    ).unwrap();
    pub static ref RGB_PANGO_RE: Regex = Regex::new(
        r#"#(?P<red>[a-fA-F0-9][a-fA-F0-9])(?P<green>[a-fA-F0-9][a-fA-F0-9])(?P<blue>[a-fA-F0-9][a-fA-F0-9])"#
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

impl From<[u16; 3]> for RGB16 {
    fn from(array: [u16; 3]) -> Self {
        Self(array)
    }
}

impl From<RGB16> for [u16; 3] {
    fn from(rgb: RGB16) -> Self {
        rgb.0
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

impl std::fmt::Display for RGB16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RGB16(red=0x{:04X}, green=0x{:04X}, blue=0x{:04X})",
            self.0[0], self.0[1], self.0[2]
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct RGB8([u8; 3]);

impl Index<usize> for RGB8 {
    type Output = u8;

    fn index(&self, index: usize) -> &u8 {
        &self.0[index]
    }
}

impl RGB8 {
    pub const RED: Self = Self([0xFF, 0x00, 0x00]);
    pub const GREEN: Self = Self([0x00, 0xFF, 0x00]);
    pub const BLUE: Self = Self([0x00, 0x00, 0xFF]);

    pub const CYAN: Self = Self([0x00, 0xFF, 0xFF]);
    pub const MAGENTA: Self = Self([0xFF, 0x00, 0xFF]);
    pub const YELLOW: Self = Self([0xFF, 0xFF, 0x00]);

    pub const WHITE: Self = Self([0xFF, 0xFF, 0xFF]);
    pub const BLACK: Self = Self([0x00, 0x00, 0x00]);

    pub const PRIMARIES: [Self; 3] = [Self::RED, Self::GREEN, Self::BLUE];
    pub const SECONDARIES: [Self; 3] = [Self::CYAN, Self::MAGENTA, Self::YELLOW];
    pub const GREYS: [Self; 2] = [Self::BLACK, Self::WHITE];

    pub fn pango_string(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.0[0], self.0[1], self.0[2])
    }
}

impl FromStr for RGB8 {
    type Err = RGBError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let Some(captures) = RGB_PANGO_RE.captures(string) {
            let red = u8::from_str_radix(captures.name("red").unwrap().as_str(), 16)?;
            let green = u8::from_str_radix(captures.name("green").unwrap().as_str(), 16)?;
            let blue = u8::from_str_radix(captures.name("blue").unwrap().as_str(), 16)?;
            Ok(RGB8([red, green, blue]))
        } else {
            Err(RGBError::MalformedText(string.to_string()))
        }
    }
}

impl From<[u8; 3]> for RGB8 {
    fn from(array: [u8; 3]) -> Self {
        Self(array)
    }
}

impl From<RGB8> for [u8; 3] {
    fn from(rgb: RGB8) -> Self {
        rgb.0
    }
}

impl<F: ColourComponent> From<RGB<F>> for RGB8 {
    fn from(rgb: RGB<F>) -> Self {
        let scale: F = F::from_u8(0xFF).unwrap();
        let red: u8 = (rgb.0[0] * scale).round().to_u8().unwrap();
        let green: u8 = (rgb.0[1] * scale).round().to_u8().unwrap();
        let blue: u8 = (rgb.0[2] * scale).round().to_u8().unwrap();
        Self([red, green, blue])
    }
}

impl<F: ColourComponent> From<RGB8> for RGB<F> {
    fn from(rgb: RGB8) -> Self {
        let scale: F = F::from_u8(0xFF).unwrap();
        let red: F = F::from_u8(rgb.0[0]).unwrap() / scale;
        let green: F = F::from_u8(rgb.0[1]).unwrap() / scale;
        let blue: F = F::from_u8(rgb.0[2]).unwrap() / scale;
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
    fn rgb8_to_and_from_rgb() {
        assert_eq!(RGB8([0xff, 0xff, 0x0]), RGB::<f64>::YELLOW.into());
        assert_eq!(RGB::<f32>::CYAN, RGB8([0, 0xff, 0xff]).into());
    }

    #[test]
    fn rgb8_from_str() {
        assert_eq!(
            RGB8::from_str("#F8A0F6)").unwrap(),
            RGB8([0xF8, 0xA0, 0xF6])
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
