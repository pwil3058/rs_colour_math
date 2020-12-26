// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{ops::Index, str::FromStr};

use regex::Regex;

use crate::{
    rgb::ColourComponent, rgba::RGBA, urgb::UnsignedComponent, HueConstants, RGBConstants,
};

#[derive(
    Serialize, Deserialize, Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct URGBA<U: UnsignedComponent>([U; 4]);

pub type RGBA16 = crate::urgba::URGBA<u16>;

pub type RGBA8 = crate::urgba::URGBA<u8>;

impl<U> URGBA<U>
where
    U: UnsignedComponent,
{
    pub fn iter(&self) -> impl Iterator<Item = &U> {
        self.0.iter()
    }

    pub fn pango_string(&self) -> String {
        let urgba: URGBA<u8> = self.into();
        format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            urgba.0[0], urgba.0[1], urgba.0[2], urgba.0[3]
        )
    }
}

impl<U: UnsignedComponent> HueConstants for URGBA<U> {
    const RED: Self = Self([U::MAX, U::ZERO, U::ZERO, U::MAX]);
    const GREEN: Self = Self([U::ZERO, U::MAX, U::ZERO, U::MAX]);
    const BLUE: Self = Self([U::ZERO, U::ZERO, U::MAX, U::MAX]);

    const CYAN: Self = Self([U::ZERO, U::MAX, U::MAX, U::MAX]);
    const MAGENTA: Self = Self([U::MAX, U::ZERO, U::MAX, U::MAX]);
    const YELLOW: Self = Self([U::MAX, U::MAX, U::ZERO, U::MAX]);
}

impl<U: UnsignedComponent> RGBConstants for URGBA<U> {
    const WHITE: Self = Self([U::ZERO, U::ZERO, U::ZERO, U::MAX]);
    const BLACK: Self = Self([U::MAX, U::MAX, U::MAX, U::MAX]);
}

impl<U: UnsignedComponent> From<&[U]> for URGBA<U> {
    fn from(array: &[U]) -> Self {
        debug_assert!(array.len() == 4);
        Self([array[0], array[1], array[2], array[3]])
    }
}

impl<U: UnsignedComponent> From<&[U; 4]> for URGBA<U> {
    fn from(array: &[U; 4]) -> Self {
        Self([array[0], array[1], array[2], array[3]])
    }
}

impl<U: UnsignedComponent> From<[U; 4]> for URGBA<U> {
    fn from(array: [U; 4]) -> Self {
        Self(array)
    }
}

impl<U: UnsignedComponent> From<&(U, U, U, U)> for URGBA<U> {
    fn from(tuple: &(U, U, U, U)) -> Self {
        Self([tuple.0, tuple.1, tuple.2, tuple.3])
    }
}

impl<U: UnsignedComponent> From<(U, U, U, U)> for URGBA<U> {
    fn from(tuple: (U, U, U, U)) -> Self {
        Self([tuple.0, tuple.1, tuple.2, tuple.3])
    }
}

impl<U, F> From<&RGBA<F>> for URGBA<U>
where
    F: ColourComponent,
    U: UnsignedComponent,
{
    fn from(rgba: &RGBA<F>) -> Self {
        let v: Vec<U> = rgba.iter().map(|f| U::from_fcc(*f)).collect();
        URGBA::<U>::from(&v[..])
    }
}

impl<U, F> From<RGBA<F>> for URGBA<U>
where
    F: ColourComponent,
    U: UnsignedComponent,
{
    fn from(rgba: RGBA<F>) -> Self {
        (&rgba).into()
    }
}

impl<F, U> From<&URGBA<U>> for RGBA<F>
where
    F: ColourComponent,
    U: UnsignedComponent,
{
    fn from(urgba: &URGBA<U>) -> Self {
        let v: Vec<F> = urgba.iter().map(|u| u.to_fcc()).collect();
        RGBA::<F>::from([v[0], v[1], v[2], v[3]])
    }
}

impl<F, U> From<URGBA<U>> for RGBA<F>
where
    F: ColourComponent,
    U: UnsignedComponent,
{
    fn from(urgba: URGBA<U>) -> Self {
        (&urgba).into()
    }
}

impl<U, V> From<&URGBA<V>> for URGBA<U>
where
    U: UnsignedComponent,
    V: UnsignedComponent,
{
    fn from(urgba: &URGBA<V>) -> Self {
        if U::BYTES == V::BYTES {
            Self([
                U::from::<V>(urgba.0[0]).unwrap(),
                U::from::<V>(urgba.0[1]).unwrap(),
                U::from::<V>(urgba.0[2]).unwrap(),
                U::from::<V>(urgba.0[3]).unwrap(),
            ])
        } else {
            let rgba: RGBA<f64> = urgba.into();
            rgba.into()
        }
    }
}

impl<U: UnsignedComponent> From<&URGBA<U>> for (U, U, U, U) {
    fn from(urgba: &URGBA<U>) -> (U, U, U, U) {
        (urgba[0], urgba[1], urgba[2], urgba[3])
    }
}

impl<U: UnsignedComponent> From<&URGBA<U>> for [U; 4] {
    fn from(urgba: &URGBA<U>) -> [U; 4] {
        urgba.0
    }
}

impl<U: UnsignedComponent> Index<u8> for URGBA<U> {
    type Output = U;

    fn index(&self, index: u8) -> &U {
        &self.0[index as usize]
    }
}

lazy_static! {
    pub static ref RGBA16_RE: Regex = Regex::new(
        r#"RGBA(16)?\((red=)?0x(?P<red>[a-fA-F0-9]{4}), (green=)?0x(?P<green>[a-fA-F0-9]{4}), (blue=)?0x(?P<blue>[a-fA-F0-9]{4}), (alpha=)?0x(?P<alpha>[a-fA-F0-9]{4})\)"#
    ).unwrap();
    pub static ref RGBA16_BASE_10_RE: Regex = Regex::new(
        r#"RGBA(16)?\((red=)?(?P<red>\d{1,5}), (green=)?(?P<green>\d{1,5}), (blue=)?(?P<blue>\d{1,5}), (alpha=)?(?P<alpha>\d{1,5})\)"#
    ).unwrap();
    pub static ref RGBA8_RE: Regex = Regex::new(
        r#"RGBA(8)?\((red=)?0x(?P<red>[a-fA-F0-9]{2}), (green=)?0x(?P<green>[a-fA-F0-9]{2}), (blue=)?0x(?P<blue>[a-fA-F0-9]{2}), (alpha=)?0x(?P<alpha>[a-fA-F0-9]{2})\)"#
    ).unwrap();
    pub static ref RGBA8_BASE_10_RE: Regex = Regex::new(
        r#"RGBA(8)?\((red=)?(?P<red>\d{1,3}), (green=)?(?P<green>\d{1,3}), (blue=)?(?P<blue>\d{1,3}), (alpha=)?(?P<alpha>\d{1,3})\)"#
    ).unwrap();
    pub static ref RGBA_PANGO_RE: Regex = Regex::new(
        r#"#(?P<red>[a-fA-F0-9][a-fA-F0-9])(?P<green>[a-fA-F0-9][a-fA-F0-9])(?P<blue>[a-fA-F0-9][a-fA-F0-9])(?P<alpha>[a-fA-F0-9][a-fA-F0-9])"#
    ).unwrap();
}

#[derive(Debug, PartialEq, Eq)]
pub enum URGBAError {
    MalformedText(String),
}

impl std::fmt::Display for URGBAError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            URGBAError::MalformedText(string) => write!(f, "Malformed text: {}", string),
        }
    }
}

impl std::error::Error for URGBAError {}

impl From<std::num::ParseIntError> for URGBAError {
    fn from(error: std::num::ParseIntError) -> Self {
        URGBAError::MalformedText(format!("{}", error))
    }
}

impl std::fmt::Display for URGBA<u8> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RGBA8(red=0x{:02X}, green=0x{:02X}, blue=0x{:02X}, alpha=0x{:02X})",
            self[0], self[1], self[2], self[3]
        )
    }
}

impl FromStr for URGBA<u8> {
    type Err = URGBAError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let Some(captures) = RGBA8_RE.captures(string) {
            let red = u8::from_str_radix(captures.name("red").unwrap().as_str(), 16)?;
            let green = u8::from_str_radix(captures.name("green").unwrap().as_str(), 16)?;
            let blue = u8::from_str_radix(captures.name("blue").unwrap().as_str(), 16)?;
            let alpha = u8::from_str_radix(captures.name("alpha").unwrap().as_str(), 16)?;
            Ok([red, green, blue, alpha].into())
        } else if let Some(captures) = RGBA8_BASE_10_RE.captures(string) {
            let red = u8::from_str_radix(captures.name("red").unwrap().as_str(), 10)?;
            let green = u8::from_str_radix(captures.name("green").unwrap().as_str(), 10)?;
            let blue = u8::from_str_radix(captures.name("blue").unwrap().as_str(), 10)?;
            let alpha = u8::from_str_radix(captures.name("alpha").unwrap().as_str(), 10)?;
            Ok([red, green, blue, alpha].into())
        } else if let Some(captures) = RGBA_PANGO_RE.captures(string) {
            let red = u8::from_str_radix(captures.name("red").unwrap().as_str(), 16)?;
            let green = u8::from_str_radix(captures.name("green").unwrap().as_str(), 16)?;
            let blue = u8::from_str_radix(captures.name("blue").unwrap().as_str(), 16)?;
            let alpha = u8::from_str_radix(captures.name("alpha").unwrap().as_str(), 16)?;
            Ok([red, green, blue, alpha].into())
        } else {
            Err(URGBAError::MalformedText(string.to_string()))
        }
    }
}

impl std::fmt::Display for URGBA<u16> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RGBA16(red=0x{:04X}, green=0x{:04X}, blue=0x{:04X}, alpha=0x{:04X})",
            self[0], self[1], self[2], self[3]
        )
    }
}

impl FromStr for URGBA<u16> {
    type Err = URGBAError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let Some(captures) = RGBA16_RE.captures(string) {
            let red = u16::from_str_radix(captures.name("red").unwrap().as_str(), 16)?;
            let green = u16::from_str_radix(captures.name("green").unwrap().as_str(), 16)?;
            let blue = u16::from_str_radix(captures.name("blue").unwrap().as_str(), 16)?;
            let alpha = u16::from_str_radix(captures.name("alpha").unwrap().as_str(), 16)?;
            Ok([red, green, blue, alpha].into())
        } else if let Some(captures) = RGBA16_BASE_10_RE.captures(string) {
            let red = u16::from_str_radix(captures.name("red").unwrap().as_str(), 10)?;
            let green = u16::from_str_radix(captures.name("green").unwrap().as_str(), 10)?;
            let blue = u16::from_str_radix(captures.name("blue").unwrap().as_str(), 10)?;
            let alpha = u16::from_str_radix(captures.name("alpha").unwrap().as_str(), 10)?;
            Ok([red, green, blue, alpha].into())
        } else {
            Err(URGBAError::MalformedText(string.to_string()))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use num_traits_plus::{
        assert_approx_eq,
        float_plus::{FloatApproxEq, FloatPlus},
    };

    #[test]
    fn component_conversion() {
        assert_eq!(u8::from_fcc(f64::ONE), 0xFF);
        assert_eq!(u8::from_fcc(f64::ZERO), 0x00);
        assert_eq!(u8::from_fcc(0.392157_f64), 0x64);
        assert_eq!(0xFF_u8.to_fcc::<f64>(), 1.0);
        assert_eq!(0x00_u8.to_fcc::<f64>(), 0.0);
        assert_approx_eq!(0x64_u8.to_fcc::<f64>(), 0.39215686274509803);
    }

    #[test]
    fn from_rgb_to_urgb() {
        assert_eq!(URGBA::<u8>::RED, URGBA::from(&RGBA::<f64>::RED));
        assert_eq!(URGBA::<u8>::CYAN, URGBA::from(&RGBA::<f64>::CYAN));
        assert_eq!(URGBA::<u8>::YELLOW, URGBA::from(RGBA::<f64>::YELLOW));
    }

    #[test]
    fn from_urgb_to_urgb() {
        assert_eq!(URGBA::<u8>::RED, URGBA::<u8>::from(&URGBA::<u16>::RED));
        assert_eq!(URGBA::<u8>::RED, URGBA::<u8>::from(&URGBA::<u8>::RED));
        assert_eq!(URGBA::<u16>::RED, URGBA::<u16>::from(&URGBA::<u8>::RED));
    }

    #[test]
    fn rgb16_from_str() {
        assert_eq!(
            URGBA::<u16>::from_str("RGBA16(red=0xF800, green=0xFA00, blue=0xF600, alpha=0x0100)")
                .unwrap(),
            URGBA::<u16>::from([0xF800, 0xFA00, 0xF600, 0x0100])
        );
        assert_eq!(
            URGBA::<u16>::from_str("RGBA16(0xF800, 0xFA00, 0xF600, 0x0100)").unwrap(),
            URGBA::<u16>::from([0xF800, 0xFA00, 0xF600, 0x0100])
        );
        assert_eq!(
            URGBA::<u16>::from_str("RGBA16(red=78, green=2345, blue=5678, alpha=100)").unwrap(),
            URGBA::<u16>::from([78, 2345, 5678, 100])
        );
        assert_eq!(
            URGBA::<u16>::from_str("RGBA16(128, 45670, 600, 100)").unwrap(),
            URGBA::<u16>::from([128, 45670, 600, 100])
        );
    }

    #[test]
    fn rgb16_to_from_str() {
        for tuple in [(0xFFFF, 0x8000, 0x6000, 0x1000)].iter() {
            let urgba: URGBA<u16> = tuple.into();
            let string: String = urgba.to_string();
            assert_eq!(Ok(urgba), URGBA::<u16>::from_str(&string));
        }
    }

    #[test]
    fn rgb8_from_pango_str() {
        assert_eq!(
            URGBA::<u8>::from_str("#F8A0F6FF)").unwrap(),
            URGBA::<u8>::from([0xF8, 0xA0, 0xF6, 0xFF])
        );
    }

    #[test]
    fn rgb8_to_from_str() {
        for tuple in [(0xFF, 0x80, 0x60, 0xF0)].iter() {
            let urgba: URGBA<u8> = tuple.into();
            let string: String = urgba.to_string();
            assert_eq!(Ok(urgba), URGBA::<u8>::from_str(&string));
        }
    }
}
