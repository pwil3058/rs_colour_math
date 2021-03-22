// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::{
    cmp::Ordering,
    convert::TryInto,
    convert::{From, TryFrom},
    ops::Index,
    ops::{Add, Mul},
    str::FromStr,
};

use lazy_static::lazy_static;
use regex::Regex;

use crate::{
    attributes::Warmth,
    debug::ApproxEq,
    fdrn::UFDRNumber,
    hue::{CMYHue, Hue, HueIfce, RGBHue, Sextant},
    Angle, Chroma, ColourBasics, Float, HueConstants, LightLevel, ManipulatedColour, Prop,
    RGBConstants, Value, HCV,
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Default)]
pub struct RGB<T: LightLevel>(pub(crate) [T; 3]);

impl<T: LightLevel> Eq for RGB<T> where T: Eq {}

impl<T: LightLevel> HueConstants for RGB<T> {
    const RED: Self = Self([T::ONE, T::ZERO, T::ZERO]);
    const GREEN: Self = Self([T::ZERO, T::ONE, T::ZERO]);
    const BLUE: Self = Self([T::ZERO, T::ZERO, T::ONE]);

    const CYAN: Self = Self([T::ZERO, T::ONE, T::ONE]);
    const MAGENTA: Self = Self([T::ONE, T::ZERO, T::ONE]);
    const YELLOW: Self = Self([T::ONE, T::ONE, T::ZERO]);

    const BLUE_CYAN: Self = Self([T::ZERO, T::HALF, T::ONE]);
    const BLUE_MAGENTA: Self = Self([T::HALF, T::ZERO, T::ONE]);
    const RED_MAGENTA: Self = Self([T::ONE, T::ZERO, T::HALF]);
    const RED_YELLOW: Self = Self([T::ONE, T::HALF, T::ZERO]);
    const GREEN_YELLOW: Self = Self([T::HALF, T::ONE, T::ZERO]);
    const GREEN_CYAN: Self = Self([T::ZERO, T::ONE, T::HALF]);
}

impl<T: LightLevel> RGBConstants for RGB<T> {
    const WHITE: Self = Self([T::ONE, T::ONE, T::ONE]);
    const BLACK: Self = Self([T::ZERO, T::ZERO, T::ZERO]);
}

impl<T: LightLevel + Copy + From<Prop>> RGB<T> {
    pub fn new_grey(value: Value) -> Self {
        let value: Prop = value.into();
        Self::from([value, value, value])
    }

    pub fn new_warmth_rgb(warmth: Warmth) -> Self {
        let prop: Prop = warmth.into();
        Self::from([prop, Prop::ONE - prop, Prop::ONE - prop])
    }
}

impl<T: LightLevel + Into<Prop>> RGB<T> {
    pub fn sum(&self) -> UFDRNumber {
        let [red, green, blue] = <[Prop; 3]>::from(*self);
        red + green + blue
    }

    pub fn max_chroma_rgb(&self) -> RGB<T> {
        if let Ok(hue) = Hue::try_from(self) {
            hue.max_chroma_rgb::<T>()
        } else {
            RGB::<T>::new_grey(self.value())
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter()
    }
}

impl<T: LightLevel + Into<Prop>> ColourBasics for RGB<T> {
    fn hue(&self) -> Option<Hue> {
        match self.try_into() {
            Ok(rgb) => Some(rgb),
            Err(_) => None,
        }
    }

    fn is_grey(&self) -> bool {
        self.0[0] == self.0[1] && self.0[1] == self.0[2]
    }

    fn chroma(&self) -> Chroma {
        if let Ok(hue) = Hue::try_from(self) {
            let [red, green, blue] = <[Prop; 3]>::from(*self);
            let sum = self.sum();
            match hue {
                Hue::Primary(RGBHue::Red) => Chroma::from((red - blue, hue, sum)),
                Hue::Primary(RGBHue::Green) => Chroma::from((green - red, hue, sum)),
                Hue::Primary(RGBHue::Blue) => Chroma::from((blue - green, hue, sum)),
                Hue::Secondary(CMYHue::Cyan) => Chroma::from((blue - red, hue, sum)),
                Hue::Secondary(CMYHue::Magenta) => Chroma::from((red - green, hue, sum)),
                Hue::Secondary(CMYHue::Yellow) => Chroma::from((green - blue, hue, sum)),
                Hue::Sextant(sextant_hue) => match sextant_hue.sextant() {
                    Sextant::RedYellow => Chroma::from((red - blue, hue, sum)),
                    Sextant::RedMagenta => Chroma::from((red - green, hue, sum)),
                    Sextant::GreenYellow => Chroma::from((green - blue, hue, sum)),
                    Sextant::GreenCyan => Chroma::from((green - red, hue, sum)),
                    Sextant::BlueCyan => Chroma::from((blue - red, hue, sum)),
                    Sextant::BlueMagenta => Chroma::from((blue - green, hue, sum)),
                },
            }
        } else {
            Chroma::ZERO
        }
    }
    fn value(&self) -> Value {
        (self.sum() / 3).into()
    }

    fn hcv(&self) -> HCV {
        self.into()
    }

    fn rgb<L: LightLevel>(&self) -> RGB<L> {
        <[Prop; 3]>::from(*self).into()
    }
}

impl<L: LightLevel> ManipulatedColour for RGB<L> {
    fn lightened(&self, prop: Prop) -> Self {
        let compl = Prop::ONE - prop;
        let mut array = <[Prop; 3]>::from(self);
        for item in &mut array {
            *item = (*item * compl + prop).into();
        }
        RGB::<L>::from(array)
    }

    fn darkened(&self, prop: Prop) -> Self {
        let compl = Prop::ONE - prop;
        let mut array = <[Prop; 3]>::from(self);
        for item in &mut array {
            *item = *item * compl;
        }
        RGB::<L>::from(array)
    }

    fn saturated(&self, prop: Prop) -> Self {
        let hcv = HCV::from(self).saturated(prop);
        RGB::<L>::from(hcv)
    }

    fn greyed(&self, prop: Prop) -> Self {
        let hcv = HCV::from(self).greyed(prop);
        RGB::<L>::from(hcv)
    }

    fn rotated(&self, angle: Angle) -> Self {
        let hcv = HCV::from(self).rotated(angle);
        RGB::<L>::from(hcv)
    }
}

impl<T: LightLevel> Index<usize> for RGB<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        debug_assert!(index < 3);
        self.0.index(index)
    }
}

// Comparisons
impl<T: LightLevel> PartialOrd for RGB<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.0 == other.0 {
            Some(Ordering::Equal)
        } else if let Ok(hue) = Hue::try_from(self) {
            if let Ok(other_hue) = Hue::try_from(other) {
                // This orders via hue from CYAN to CYAN via GREEN, RED, BLUE in that order
                match hue.cmp(&other_hue) {
                    Ordering::Equal => match self.chroma().cmp(&other.chroma()) {
                        Ordering::Equal => Some(self.sum().cmp(&self.sum())),
                        order => Some(order),
                    },
                    order => Some(order),
                }
            } else {
                Some(Ordering::Greater)
            }
        } else if Hue::try_from(other).is_ok() {
            Some(Ordering::Less)
        } else {
            // No need to look a chroma as it will be zero for both
            self.sum().partial_cmp(&other.sum())
        }
    }
}

impl<T: LightLevel> Ord for RGB<T>
where
    T: PartialOrd + Eq,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other)
            .expect("restricted range of values means this is OK")
    }
}

impl<T: LightLevel + ApproxEq> RGB<T> {
    pub fn approx_eq(&self, other: &Self, max_diff: Option<T>) -> bool {
        for i in 0..3 {
            if !self.0[i].approx_eq(&other.0[i], max_diff) {
                return false;
            }
        }
        true
    }
}

impl<L: LightLevel> From<[L; 3]> for RGB<L> {
    fn from(array: [L; 3]) -> Self {
        debug_assert!(array.iter().all(|a| *a >= L::ZERO && *a <= L::ONE));
        Self(array)
    }
}

impl<L: LightLevel> From<RGB<L>> for [L; 3] {
    fn from(rgb: RGB<L>) -> Self {
        rgb.0
    }
}

impl<L: LightLevel> From<&[L; 3]> for RGB<L> {
    fn from(array: &[L; 3]) -> Self {
        Self::from(*array)
    }
}

impl<L: LightLevel> From<&RGB<L>> for [L; 3] {
    fn from(rgb: &RGB<L>) -> Self {
        Self::from(*rgb)
    }
}

impl<T: LightLevel + From<Prop>> From<[Prop; 3]> for RGB<T> {
    fn from(array: [Prop; 3]) -> Self {
        let red: T = array[0].into();
        let green: T = array[1].into();
        let blue: T = array[2].into();
        Self([red, green, blue])
    }
}

impl<T: LightLevel + Into<Prop>> From<RGB<T>> for [Prop; 3] {
    fn from(rgb: RGB<T>) -> Self {
        [rgb.0[0].into(), rgb.0[1].into(), rgb.0[2].into()]
    }
}

impl<T: LightLevel + From<Prop>> From<&[Prop; 3]> for RGB<T> {
    fn from(array: &[Prop; 3]) -> Self {
        Self::from(*array)
    }
}

impl<T: LightLevel + From<Prop>> From<&RGB<T>> for [Prop; 3] {
    fn from(rgb: &RGB<T>) -> Self {
        Self::from(*rgb)
    }
}

// Arithmetic
impl<F: Float + LightLevel + From<Prop>> Mul<Prop> for RGB<F> {
    type Output = Self;

    fn mul(self, scalar: Prop) -> Self {
        let [red, green, blue] = <[Prop; 3]>::from(self);
        let array: [Prop; 3] = [red * scalar, green * scalar, blue * scalar];
        Self::from(array)
    }
}

impl<L: LightLevel + From<Prop>> Add<RGB<L>> for RGB<L> {
    // TODO: get rid of RGB addition
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let [red, green, blue] = <[Prop; 3]>::from(self);
        let [rhs_red, rhs_green, rhs_blue] = <[Prop; 3]>::from(rhs);
        let array: [Prop; 3] = [
            (red + rhs_red).into(),
            (green + rhs_green).into(),
            (blue + rhs_blue).into(),
        ];
        Self::from(array)
    }
}

#[derive(Debug, PartialEq, Eq)]
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

impl FromStr for RGB<u16> {
    type Err = RGBError;

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
            Err(RGBError::MalformedText(string.to_string()))
        }
    }
}
