// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
//use std::cmp::Ordering;

#[cfg(test)]
mod proportion_tests;

use std::{
    cmp::Ordering,
    fmt::{self, Debug, Formatter},
    ops::{Add, Div, Mul, Sub},
};

use crate::{
    fdrn::{FDRNumber, UFDRNumber},
    hue::HueIfce,
    Hue,
};

#[cfg(test)]
use num_traits_plus::float_plus::*;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Chroma {
    Shade(Prop),
    Tint(Prop),
    Neither(Prop),
}

impl Chroma {
    pub const ZERO: Self = Self::Neither(Prop::ZERO);
    pub const ONE: Self = Self::Neither(Prop::ONE);

    pub fn is_zero(self) -> bool {
        self.prop() == Prop::ZERO
    }

    pub fn is_valid(self) -> bool {
        match self {
            Chroma::Neither(_) => true,
            Chroma::Shade(c_prop) | Chroma::Tint(c_prop) => {
                c_prop > Prop::ZERO && c_prop < Prop::ONE
            }
        }
    }

    pub fn prop(&self) -> Prop {
        use Chroma::*;
        match self {
            Shade(proportion) | Tint(proportion) | Neither(proportion) => *proportion,
        }
    }

    pub fn abs_diff(&self, other: &Self) -> Prop {
        self.prop().abs_diff(&other.prop())
    }
}

impl Default for Chroma {
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<(Prop, Hue, UFDRNumber)> for Chroma {
    fn from((prop, hue, sum): (Prop, Hue, UFDRNumber)) -> Self {
        match prop {
            Prop::ZERO => Chroma::ZERO,
            Prop::ONE => Chroma::ONE,
            prop => match sum.cmp(&hue.sum_for_max_chroma()) {
                Ordering::Greater => Self::Tint(prop),
                Ordering::Less => Self::Shade(prop),
                Ordering::Equal => Self::Neither(prop),
            },
        }
    }
}

impl PartialOrd for Chroma {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        use Chroma::*;
        match self {
            Shade(proportion) => match rhs {
                Shade(other_proportion) => proportion.partial_cmp(&other_proportion),
                _ => Some(Ordering::Less),
            },
            Tint(proportion) => match rhs {
                Tint(other_proportion) => proportion.partial_cmp(&other_proportion),
                Shade(_) => Some(Ordering::Greater),
                Neither(_) => Some(Ordering::Less),
            },
            Neither(proportion) => match rhs {
                Neither(other_proportion) => proportion.partial_cmp(&other_proportion),
                _ => Some(Ordering::Greater),
            },
        }
    }
}

impl Ord for Chroma {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.partial_cmp(rhs).unwrap()
    }
}

#[cfg(test)]
impl Chroma {
    pub fn approx_eq(&self, other: &Self, acceptable_rounding_error: Option<u64>) -> bool {
        use Chroma::*;
        match self {
            Shade(proportion) => match other {
                Shade(other_proportion) | Neither(other_proportion) => {
                    proportion.approx_eq(other_proportion, acceptable_rounding_error)
                }
                Tint(_) => false,
            },
            Tint(proportion) => match other {
                Shade(_) => false,
                Tint(other_proportion) | Neither(other_proportion) => {
                    proportion.approx_eq(other_proportion, acceptable_rounding_error)
                }
            },
            Neither(proportion) => proportion.approx_eq(&other.prop(), acceptable_rounding_error),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Greyness {
    Shade(Prop),
    Tint(Prop),
    Neither(Prop),
}

impl Greyness {
    pub const ZERO: Self = Self::Neither(Prop::ZERO);
    pub const ONE: Self = Self::Neither(Prop::ONE);

    pub fn is_zero(&self) -> bool {
        self.prop() == Prop::ZERO
    }

    pub fn prop(&self) -> Prop {
        use Greyness::*;
        match self {
            Shade(proportion) | Tint(proportion) | Neither(proportion) => *proportion,
        }
    }

    pub fn abs_diff(&self, other: &Self) -> Prop {
        self.prop().abs_diff(&other.prop())
    }
}

impl Default for Greyness {
    fn default() -> Self {
        Self::ZERO
    }
}

impl PartialOrd for Greyness {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        use Greyness::*;
        match self {
            Shade(proportion) => match rhs {
                Shade(other_proportion) => proportion.partial_cmp(&other_proportion),
                _ => Some(Ordering::Less),
            },
            Tint(proportion) => match rhs {
                Tint(other_proportion) => proportion.partial_cmp(&other_proportion),
                Shade(_) => Some(Ordering::Greater),
                Neither(_) => Some(Ordering::Less),
            },
            Neither(proportion) => match rhs {
                Neither(other_proportion) => proportion.partial_cmp(&other_proportion),
                _ => Some(Ordering::Greater),
            },
        }
    }
}

impl Ord for Greyness {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.partial_cmp(rhs).unwrap()
    }
}

#[cfg(test)]
impl Greyness {
    pub fn approx_eq(&self, other: &Self, acceptable_rounding_error: Option<u64>) -> bool {
        use Greyness::*;
        match self {
            Shade(proportion) => match other {
                Shade(other_proportion) | Neither(other_proportion) => {
                    proportion.approx_eq(other_proportion, acceptable_rounding_error)
                }
                Tint(_) => false,
            },
            Tint(proportion) => match other {
                Shade(_) => false,
                Tint(other_proportion) | Neither(other_proportion) => {
                    proportion.approx_eq(other_proportion, acceptable_rounding_error)
                }
            },
            Neither(proportion) => proportion.approx_eq(&other.prop(), acceptable_rounding_error),
        }
    }
}

impl From<Chroma> for Greyness {
    fn from(chroma: Chroma) -> Self {
        match chroma {
            Chroma::Shade(prop) => Greyness::Shade(Prop::ONE - prop),
            Chroma::Tint(prop) => Greyness::Tint(Prop::ONE - prop),
            Chroma::Neither(prop) => Greyness::Neither(Prop::ONE - prop),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Prop(pub(crate) u64);

impl Prop {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(u64::MAX);

    pub(crate) const HALF: Self = Self(u64::MAX / 2);

    pub fn abs_diff(&self, other: &Self) -> Prop {
        match self.cmp(other) {
            Ordering::Greater => Prop(self.0 - other.0),
            Ordering::Less => Prop(other.0 - self.0),
            Ordering::Equal => Prop(0),
        }
    }
}

#[cfg(test)]
impl Prop {
    pub fn approx_eq(&self, other: &Self, acceptable_rounding_error: Option<u64>) -> bool {
        let abs_diff = self.abs_diff(other);
        let scaled_diff = if self >= other {
            if self.0 > 0 {
                ((u64::MAX / self.0) as u128 * abs_diff.0 as u128 / u64::MAX as u128) as u64
            } else {
                abs_diff.0
            }
        } else {
            ((u64::MAX / other.0) as u128 * abs_diff.0 as u128 / u64::MAX as u128) as u64
        };
        if let Some(acceptable_rounding_error) = acceptable_rounding_error {
            scaled_diff < acceptable_rounding_error as u64
        } else {
            scaled_diff < u64::MAX / 1_000_000_000_000_000
        }
    }
}

impl Debug for Prop {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_fmt(format_args!("Prop(0.{:016X})", self))
        //formatter.write_fmt(format_args!("Prop({:?})", f64::from(*self)))
    }
}

impl fmt::UpperHex for Prop {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, formatter)
    }
}

macro_rules! impl_to_from_float {
    ($float:ty, $number:ty) => {
        impl From<$float> for $number {
            fn from(arg: $float) -> Self {
                debug_assert!(0.0 <= arg && arg <= 1.0);
                // TODO: watch out for floating point not being proper reals
                Self((arg * u64::MAX as $float) as u64)
            }
        }

        impl From<$number> for $float {
            fn from(arg: $number) -> Self {
                arg.0 as $float / u64::MAX as $float
            }
        }
    };
}

impl_to_from_float!(f32, Prop);
impl_to_from_float!(f64, Prop);

macro_rules! impl_to_from_number {
    ($number:ty, $core:ty, $proportion:ty) => {
        impl From<$number> for $proportion {
            #[allow(unused_comparisons)]
            fn from(arg: $number) -> Self {
                debug_assert!(arg.0 >= 0 && arg.0 <= u64::MAX as $core);
                Self(arg.0 as u64)
            }
        }

        impl From<$proportion> for $number {
            fn from(arg: $proportion) -> Self {
                Self(arg.0 as $core)
            }
        }
    };
}

impl_to_from_number!(UFDRNumber, u128, Prop);
impl_to_from_number!(FDRNumber, i128, Prop);

macro_rules! impl_unsigned_to_from_prop {
    (u64) => {
        impl From<u64> for Prop {
            fn from(arg: u64) -> Self {
                Self(arg as u64)
            }
        }

        impl From<Prop> for u64 {
            fn from(arg: Prop) -> Self {
                arg.0 as u64
            }
        }
    };
    ($unsigned:ty) => {
        impl From<$unsigned> for Prop {
            fn from(arg: $unsigned) -> Self {
                let val = arg as u128 * u64::MAX as u128 / <$unsigned>::MAX as u128;
                Self(val as u64)
            }
        }

        impl From<Prop> for $unsigned {
            fn from(arg: Prop) -> Self {
                let val = arg.0 as u128 * <$unsigned>::MAX as u128 / u64::MAX as u128;
                val as $unsigned
            }
        }
    };
}

impl_unsigned_to_from_prop!(u8);
impl_unsigned_to_from_prop!(u16);
impl_unsigned_to_from_prop!(u32);
impl_unsigned_to_from_prop!(u64);

impl Mul for Prop {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self(((self.0 as u128 * rhs.0 as u128) / u64::MAX as u128) as u64)
    }
}

impl Mul<u8> for Prop {
    type Output = UFDRNumber;

    fn mul(self, rhs: u8) -> UFDRNumber {
        UFDRNumber(self.0 as u128 * rhs as u128)
    }
}

impl Div for Prop {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        let result = (self.0 as u128 * u64::MAX as u128) / rhs.0 as u128;
        Self(result as u64)
    }
}

impl Div<u8> for Prop {
    type Output = Self;

    fn div(self, rhs: u8) -> Self {
        Prop(self.0 / rhs as u64)
    }
}

impl Add for Prop {
    type Output = UFDRNumber;

    fn add(self, rhs: Self) -> UFDRNumber {
        UFDRNumber(self.0 as u128 + rhs.0 as u128)
    }
}

impl Sub for Prop {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        debug_assert!(self.0 >= rhs.0);
        Self(self.0 - rhs.0)
    }
}

impl Add<Prop> for UFDRNumber {
    type Output = Self;

    fn add(self, rhs: Prop) -> UFDRNumber {
        UFDRNumber(self.0 + rhs.0 as u128)
    }
}

impl Sub<Prop> for UFDRNumber {
    type Output = Self;

    fn sub(self, rhs: Prop) -> Self {
        debug_assert!(self.0 >= rhs.0 as u128);
        Self(self.0 - rhs.0 as u128)
    }
}

impl Mul<Prop> for UFDRNumber {
    type Output = Self;

    fn mul(self, rhs: Prop) -> Self {
        if rhs.0 == u64::MAX {
            self
        } else if self.0 >= u64::MAX as u128 {
            let one = u64::MAX as u128;
            let l_int = self.0 / one;
            let l_rem = self.0 % one;
            Self(l_int * rhs.0 as u128 + rhs.0 as u128 * l_rem / one)
        } else {
            Self((self.0 * rhs.0 as u128) / u64::MAX as u128)
        }
    }
}

impl Mul<u8> for UFDRNumber {
    type Output = Self;

    fn mul(self, rhs: u8) -> Self {
        Self(self.0 * rhs as u128)
    }
}

#[derive(
    Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default, Debug,
)]
pub struct Warmth(pub(crate) u64);

impl Warmth {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(u64::MAX);

    const K: Prop = Prop(u64::MAX / 3);
    const K_COMP: Prop = Prop(u64::MAX - Self::K.0);
    const B: UFDRNumber = UFDRNumber(u64::MAX as u128 / 2);

    pub fn calculate(chroma: Chroma, x_dash: Prop) -> Self {
        debug_assert_ne!(chroma, Chroma::ZERO);
        let temp = (Self::K + Self::K_COMP * x_dash) * chroma.prop();
        debug_assert!(temp <= UFDRNumber::ONE);
        match chroma {
            Chroma::Shade(prop) => {
                let warmth = Self::B - Self::B * prop + temp;
                debug_assert!(warmth <= UFDRNumber::ONE);
                warmth.into()
            }
            _ => temp.into(),
        }
    }

    pub(crate) fn calculate_monochrome_fm_sum(sum: UFDRNumber) -> Self {
        ((UFDRNumber::THREE - sum) / 6).into()
    }

    pub fn calculate_monochrome(value: Prop) -> Self {
        ((Prop::ONE - value) / 2).into()
    }

    pub fn abs_diff(&self, other: &Self) -> Warmth {
        match self.cmp(other) {
            Ordering::Greater => Warmth(self.0 - other.0),
            Ordering::Less => Warmth(other.0 - self.0),
            Ordering::Equal => Warmth(0),
        }
    }
}

#[cfg(test)]
impl Warmth {
    pub fn approx_eq(&self, other: &Self, acceptable_rounding_error: Option<u64>) -> bool {
        Prop::from(*self).approx_eq(&(*other).into(), acceptable_rounding_error)
    }
}

impl_to_from_float!(f32, Warmth);
impl_to_from_float!(f64, Warmth);

impl From<Prop> for Warmth {
    fn from(prop: Prop) -> Self {
        Self(prop.0)
    }
}

impl From<Warmth> for Prop {
    fn from(warmth: Warmth) -> Self {
        Self(warmth.0)
    }
}

impl_to_from_number!(UFDRNumber, u128, Warmth);
impl_to_from_number!(FDRNumber, i128, Warmth);
