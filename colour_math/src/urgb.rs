// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{fmt::UpperHex, ops::Index, str::FromStr};

use regex::Regex;

use crate::{
    rgb::{ColourComponent, RGB},
    HueConstants, RGBConstants,
};

pub trait UnsignedComponent:
    Copy
    + Default
    + Ord
    + UpperHex
    + num_traits_plus::num_traits::Unsigned
    + num_traits_plus::num_traits::NumCast
    + num_traits_plus::num_traits::ToPrimitive
    + num_traits_plus::NumberConstants
{
    fn from_fcc<F: ColourComponent>(cc: F) -> Self {
        debug_assert!(cc >= F::ZERO && cc <= F::ONE);
        let value = F::from::<Self>(Self::MAX).unwrap() * cc;
        Self::from::<F>(value.round()).unwrap()
    }

    fn to_fcc<F: ColourComponent>(self) -> F {
        F::from::<Self>(self).unwrap() / F::from::<Self>(Self::MAX).unwrap()
    }
}

impl UnsignedComponent for u8 {}

impl UnsignedComponent for u16 {}

#[derive(
    Serialize, Deserialize, Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct URGB<U: UnsignedComponent>([U; 3]);

pub type RGB16 = crate::urgb::URGB<u16>;

pub type RGB8 = crate::urgb::URGB<u8>;

impl<U> URGB<U>
where
    U: UnsignedComponent,
{
    pub fn iter(&self) -> impl Iterator<Item = &U> {
        self.0.iter()
    }

    pub fn pango_string(&self) -> String {
        let urgb: URGB<u8> = self.into();
        format!("#{:02X}{:02X}{:02X}", urgb.0[0], urgb.0[1], urgb.0[2])
    }
}

impl<U: UnsignedComponent> HueConstants for URGB<U> {
    const RED: Self = Self([U::MAX, U::ZERO, U::ZERO]);
    const GREEN: Self = Self([U::ZERO, U::MAX, U::ZERO]);
    const BLUE: Self = Self([U::ZERO, U::ZERO, U::MAX]);

    const CYAN: Self = Self([U::ZERO, U::MAX, U::MAX]);
    const MAGENTA: Self = Self([U::MAX, U::ZERO, U::MAX]);
    const YELLOW: Self = Self([U::MAX, U::MAX, U::ZERO]);
}

impl<U: UnsignedComponent> RGBConstants for URGB<U> {
    const WHITE: Self = Self([U::ZERO, U::ZERO, U::ZERO]);
    const BLACK: Self = Self([U::MAX, U::MAX, U::MAX]);
}

impl<U: UnsignedComponent> From<&[U]> for URGB<U> {
    fn from(array: &[U]) -> Self {
        debug_assert!(array.len() == 3);
        Self([array[0], array[1], array[2]])
    }
}

impl<U: UnsignedComponent> From<&[U; 3]> for URGB<U> {
    fn from(array: &[U; 3]) -> Self {
        Self([array[0], array[1], array[2]])
    }
}

impl<U: UnsignedComponent> From<[U; 3]> for URGB<U> {
    fn from(array: [U; 3]) -> Self {
        Self(array)
    }
}

impl<U: UnsignedComponent> From<&(U, U, U)> for URGB<U> {
    fn from(tuple: &(U, U, U)) -> Self {
        Self([tuple.0, tuple.1, tuple.2])
    }
}

impl<U: UnsignedComponent> From<(U, U, U)> for URGB<U> {
    fn from(tuple: (U, U, U)) -> Self {
        Self([tuple.0, tuple.1, tuple.2])
    }
}

impl<U, F> From<&RGB<F>> for URGB<U>
where
    F: ColourComponent,
    U: UnsignedComponent,
{
    fn from(rgb: &RGB<F>) -> Self {
        let v: Vec<U> = rgb.iter().map(|f| U::from_fcc(*f)).collect();
        URGB::<U>::from(&v[..])
    }
}

impl<U, F> From<RGB<F>> for URGB<U>
where
    F: ColourComponent,
    U: UnsignedComponent,
{
    fn from(rgb: RGB<F>) -> Self {
        (&rgb).into()
    }
}

impl<F, U> From<&URGB<U>> for RGB<F>
where
    F: ColourComponent,
    U: UnsignedComponent,
{
    fn from(urgb: &URGB<U>) -> Self {
        let v: Vec<F> = urgb.iter().map(|u| u.to_fcc()).collect();
        RGB::<F>::from([v[0], v[1], v[2]])
    }
}

impl<F, U> From<URGB<U>> for RGB<F>
where
    F: ColourComponent,
    U: UnsignedComponent,
{
    fn from(urgb: URGB<U>) -> Self {
        (&urgb).into()
    }
}

impl<U, V> From<&URGB<V>> for URGB<U>
where
    U: UnsignedComponent,
    V: UnsignedComponent,
{
    fn from(urgb: &URGB<V>) -> Self {
        if U::BYTES == V::BYTES {
            Self([
                U::from::<V>(urgb.0[0]).unwrap(),
                U::from::<V>(urgb.0[1]).unwrap(),
                U::from::<V>(urgb.0[2]).unwrap(),
            ])
        } else {
            let rgb: RGB<f64> = urgb.into();
            rgb.into()
        }
    }
}

impl<U: UnsignedComponent> From<&URGB<U>> for (U, U, U) {
    fn from(urgb: &URGB<U>) -> (U, U, U) {
        (urgb[0], urgb[1], urgb[2])
    }
}

impl<U: UnsignedComponent> From<&URGB<U>> for [U; 3] {
    fn from(urgb: &URGB<U>) -> [U; 3] {
        urgb.0
    }
}

impl<U: UnsignedComponent> Index<u8> for URGB<U> {
    type Output = U;

    fn index(&self, index: u8) -> &U {
        &self.0[index as usize]
    }
}

lazy_static! {
    pub static ref RGB16_RE: Regex = Regex::new(
        r#"RGB(16)?\((red=)?0x(?P<red>[a-fA-F0-9]{4}), (green=)?0x(?P<green>[a-fA-F0-9]{4}), (blue=)?0x(?P<blue>[a-fA-F0-9]{4})\)"#
    ).unwrap();
    pub static ref RGB16_BASE_10_RE: Regex = Regex::new(
        r#"RGB(16)?\((red=)?(?P<red>\d{1,5}), (green=)?(?P<green>\d{1,5}), (blue=)?(?P<blue>\d{1,5})\)"#
    ).unwrap();
    pub static ref RGB8_RE: Regex = Regex::new(
        r#"RGB(8)?\((red=)?0x(?P<red>[a-fA-F0-9]{2}), (green=)?0x(?P<green>[a-fA-F0-9]{2}), (blue=)?0x(?P<blue>[a-fA-F0-9]{2})\)"#
    ).unwrap();
    pub static ref RGB8_BASE_10_RE: Regex = Regex::new(
        r#"RGB(8)?\((red=)?(?P<red>\d{1,3}), (green=)?(?P<green>\d{1,3}), (blue=)?(?P<blue>\d{1,3})\)"#
    ).unwrap();
    pub static ref RGB_PANGO_RE: Regex = Regex::new(
        r#"#(?P<red>[a-fA-F0-9][a-fA-F0-9])(?P<green>[a-fA-F0-9][a-fA-F0-9])(?P<blue>[a-fA-F0-9][a-fA-F0-9])"#
    ).unwrap();
}

#[derive(Debug)]
pub enum URGBError {
    MalformedText(String),
}

impl std::fmt::Display for URGBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            URGBError::MalformedText(string) => write!(f, "Malformed text: {}", string),
        }
    }
}

impl std::error::Error for URGBError {}

impl From<std::num::ParseIntError> for URGBError {
    fn from(error: std::num::ParseIntError) -> Self {
        URGBError::MalformedText(format!("{}", error))
    }
}

impl FromStr for URGB<u8> {
    type Err = URGBError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let Some(captures) = RGB8_RE.captures(string) {
            let red = u8::from_str_radix(captures.name("red").unwrap().as_str(), 16)?;
            let green = u8::from_str_radix(captures.name("green").unwrap().as_str(), 16)?;
            let blue = u8::from_str_radix(captures.name("blue").unwrap().as_str(), 16)?;
            Ok([red, green, blue].into())
        } else if let Some(captures) = RGB8_BASE_10_RE.captures(string) {
            let red = u8::from_str_radix(captures.name("red").unwrap().as_str(), 10)?;
            let green = u8::from_str_radix(captures.name("green").unwrap().as_str(), 10)?;
            let blue = u8::from_str_radix(captures.name("blue").unwrap().as_str(), 10)?;
            Ok([red, green, blue].into())
        } else if let Some(captures) = RGB_PANGO_RE.captures(string) {
            let red = u8::from_str_radix(captures.name("red").unwrap().as_str(), 16)?;
            let green = u8::from_str_radix(captures.name("green").unwrap().as_str(), 16)?;
            let blue = u8::from_str_radix(captures.name("blue").unwrap().as_str(), 16)?;
            Ok([red, green, blue].into())
        } else {
            Err(URGBError::MalformedText(string.to_string()))
        }
    }
}

impl std::fmt::Display for URGB<u16> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "URGB<u16>(red=0x{:04X}, green=0x{:04X}, blue=0x{:04X})",
            self[0], self[1], self[2]
        )
    }
}

impl FromStr for URGB<u16> {
    type Err = URGBError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let Some(captures) = RGB16_RE.captures(string) {
            let red = u16::from_str_radix(captures.name("red").unwrap().as_str(), 16)?;
            let green = u16::from_str_radix(captures.name("green").unwrap().as_str(), 16)?;
            let blue = u16::from_str_radix(captures.name("blue").unwrap().as_str(), 16)?;
            Ok([red, green, blue].into())
        } else if let Some(captures) = RGB16_BASE_10_RE.captures(string) {
            let red = u16::from_str_radix(captures.name("red").unwrap().as_str(), 10)?;
            let green = u16::from_str_radix(captures.name("green").unwrap().as_str(), 10)?;
            let blue = u16::from_str_radix(captures.name("blue").unwrap().as_str(), 10)?;
            Ok([red, green, blue].into())
        } else {
            Err(URGBError::MalformedText(string.to_string()))
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
        assert_eq!(URGB::<u8>::RED, URGB::from(&RGB::<f64>::RED));
        assert_eq!(URGB::<u8>::CYAN, URGB::from(&RGB::<f64>::CYAN));
        assert_eq!(URGB::<u8>::YELLOW, URGB::from(RGB::<f64>::YELLOW));
    }

    #[test]
    fn from_urgb_to_urgb() {
        assert_eq!(URGB::<u8>::RED, URGB::<u8>::from(&URGB::<u16>::RED));
        assert_eq!(URGB::<u8>::RED, URGB::<u8>::from(&URGB::<u8>::RED));
        assert_eq!(URGB::<u16>::RED, URGB::<u16>::from(&URGB::<u8>::RED));
    }

    #[test]
    fn rgb16_from_str() {
        assert_eq!(
            URGB::<u16>::from_str("RGB16(red=0xF800, green=0xFA00, blue=0xF600)").unwrap(),
            URGB::<u16>::from([0xF800, 0xFA00, 0xF600])
        );
        assert_eq!(
            URGB::<u16>::from_str("RGB16(0xF800, 0xFA00, 0xF600)").unwrap(),
            URGB::<u16>::from([0xF800, 0xFA00, 0xF600])
        );
        assert_eq!(
            URGB::<u16>::from_str("RGB16(red=78, green=2345, blue=5678)").unwrap(),
            URGB::<u16>::from([78, 2345, 5678])
        );
        assert_eq!(
            URGB::<u16>::from_str("RGB16(128, 45670, 600)").unwrap(),
            URGB::<u16>::from([128, 45670, 600])
        );
    }

    #[test]
    fn rgb8_from_pango_str() {
        assert_eq!(
            URGB::<u8>::from_str("#F8A0F6)").unwrap(),
            URGB::<u8>::from([0xF8, 0xA0, 0xF6])
        );
    }
}
