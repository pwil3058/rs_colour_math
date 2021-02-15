// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
//use std::cmp::Ordering;

#[cfg(test)]
mod proportion_tests;

use std::{
    cmp::Ordering,
    fmt::{self, Debug, Formatter},
    ops::{Add, Div, Mul, Sub},
};

use crate::hue::HueIfce;
use crate::Hue;
use num_traits::FromPrimitive;
#[cfg(test)]
use num_traits_plus::float_plus::*;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, Eq)]
pub enum Chroma {
    Shade(Prop),
    Tint(Prop),
    Either(Prop),
}

impl Chroma {
    pub const ZERO: Self = Self::Either(Prop::ZERO);
    pub const ONE: Self = Self::Either(Prop::ONE);

    pub fn is_zero(&self) -> bool {
        self.prop() == Prop::ZERO
    }

    pub fn prop(&self) -> Prop {
        use Chroma::*;
        match self {
            Shade(proportion) | Tint(proportion) | Either(proportion) => *proportion,
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

impl From<(Prop, Hue, Sum)> for Chroma {
    fn from((prop, hue, sum): (Prop, Hue, Sum)) -> Self {
        match prop {
            Prop::ZERO => Chroma::ZERO,
            Prop::ONE => Chroma::ONE,
            prop => match sum.cmp(&hue.sum_for_max_chroma()) {
                Ordering::Greater => Self::Tint(prop),
                Ordering::Less => Self::Shade(prop),
                Ordering::Equal => Self::Either(prop),
            },
        }
    }
}

impl PartialEq for Chroma {
    fn eq(&self, rhs: &Self) -> bool {
        use Chroma::*;
        match self {
            Shade(proportion) => match rhs {
                Shade(other_proportion) | Either(other_proportion) => {
                    proportion.eq(&other_proportion)
                }
                Tint(_) => false,
            },
            Tint(proportion) => match rhs {
                Shade(_) => false,
                Tint(other_proportion) | Either(other_proportion) => {
                    proportion.eq(&other_proportion)
                }
            },
            Either(proportion) => proportion.eq(&rhs.prop()),
        }
    }
}

impl PartialOrd for Chroma {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        use Chroma::*;
        match self {
            Shade(proportion) => match rhs {
                Shade(other_proportion) | Either(other_proportion) => {
                    proportion.partial_cmp(&other_proportion)
                }
                Tint(_) => Some(Ordering::Less),
            },
            Tint(proportion) => match rhs {
                Shade(_) => Some(Ordering::Greater),
                Tint(other_proportion) | Either(other_proportion) => {
                    proportion.partial_cmp(&other_proportion)
                }
            },
            Either(proportion) => proportion.partial_cmp(&rhs.prop()),
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
                Shade(other_proportion) | Either(other_proportion) => {
                    proportion.approx_eq(other_proportion, acceptable_rounding_error)
                }
                Tint(_) => false,
            },
            Tint(proportion) => match other {
                Shade(_) => false,
                Tint(other_proportion) | Either(other_proportion) => {
                    proportion.approx_eq(other_proportion, acceptable_rounding_error)
                }
            },
            Either(proportion) => proportion.approx_eq(&other.prop(), acceptable_rounding_error),
        }
    }
}

impl Add<Prop> for Chroma {
    type Output = Chroma;

    fn add(self, rhs: Prop) -> Chroma {
        match self.prop() {
            Prop::ONE => Chroma::ONE,
            Prop::ZERO => Chroma::Either(rhs),
            cur_prop => {
                let available_space = Prop::ONE - cur_prop;
                if available_space > rhs {
                    let new_prop: Prop = (cur_prop + rhs).into();
                    match self {
                        Chroma::Shade(_) => Chroma::Shade(new_prop),
                        Chroma::Tint(_) => Chroma::Tint(new_prop),
                        Chroma::Either(_) => Chroma::Either(new_prop),
                    }
                } else {
                    Chroma::ONE
                }
            }
        }
    }
}

impl Sub<Prop> for Chroma {
    type Output = Chroma;

    fn sub(self, rhs: Prop) -> Chroma {
        match self.prop() {
            Prop::ONE => Chroma::Either(Prop::ONE - rhs),
            Prop::ZERO => Chroma::ZERO,
            cur_prop => {
                if cur_prop > rhs {
                    let new_prop: Prop = cur_prop - rhs;
                    match self {
                        Chroma::Shade(_) => Chroma::Shade(new_prop),
                        Chroma::Tint(_) => Chroma::Tint(new_prop),
                        Chroma::Either(_) => Chroma::Either(new_prop),
                    }
                } else {
                    Chroma::ZERO
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Prop(pub(crate) u64);

impl Prop {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(u64::MAX);

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
        if let Some(acceptable_rounding_error) = acceptable_rounding_error {
            self.abs_diff(other) < Prop(acceptable_rounding_error)
        } else {
            self.0 < 3
        }
    }
}

impl Debug for Prop {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        //formatter.write_fmt(format_args!("Prop(0.{:08X})", self.0))
        formatter.write_fmt(format_args!("Prop({:?})", f64::from(*self)))
    }
}

impl From<f32> for Prop {
    fn from(arg: f32) -> Self {
        debug_assert!(arg <= 1.0);
        let one = f32::from_u64(u64::MAX).unwrap();
        let val = u64::from_f32(arg * one).unwrap();
        Self(val)
    }
}

impl From<Prop> for f32 {
    fn from(arg: Prop) -> Self {
        let one = f32::from_u64(u64::MAX).unwrap();
        f32::from_u64(arg.0).unwrap() / one
    }
}

impl From<f64> for Prop {
    fn from(arg: f64) -> Self {
        debug_assert!(0.0 <= arg && arg <= 1.0);
        let one = f64::from_u64(u64::MAX).unwrap();
        let prod = arg * one;
        // NB: watch out for floating point not being proper reals
        if prod >= one {
            Self(u64::MAX)
        } else {
            Self(u64::from_f64(arg * one).unwrap())
        }
    }
}

impl From<Prop> for f64 {
    fn from(arg: Prop) -> Self {
        let one = f64::from_u64(u64::MAX).unwrap();
        f64::from_u64(arg.0).unwrap() / one
    }
}

impl From<Sum> for Prop {
    fn from(arg: Sum) -> Self {
        debug_assert!(arg.0 <= u64::MAX as u128);
        Self(arg.0 as u64)
    }
}

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
    type Output = Sum;

    fn mul(self, rhs: u8) -> Sum {
        Sum(self.0 as u128 * rhs as u128)
    }
}

impl Div for Prop {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        debug_assert!(self.0 <= rhs.0);
        let result = (self.0 as u128 * u64::MAX as u128) / rhs.0 as u128;
        Self(result as u64)
    }
}

impl Add for Prop {
    type Output = Sum;

    fn add(self, rhs: Self) -> Sum {
        Sum(self.0 as u128 + rhs.0 as u128)
    }
}

impl Sub for Prop {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        debug_assert!(self.0 >= rhs.0);
        Self(self.0 - rhs.0)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Sum(pub(crate) u128);

impl Sum {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(u64::MAX as u128);
    pub const TWO: Self = Self(u64::MAX as u128 * 2);
    pub const THREE: Self = Self(u64::MAX as u128 * 3);

    pub fn is_valid(self) -> bool {
        self <= Self::THREE
    }

    pub fn is_hue_valid(self) -> bool {
        self > Self::ZERO && self < Self::THREE
    }

    pub fn abs_diff(&self, other: &Self) -> Sum {
        match self.cmp(other) {
            Ordering::Greater => Sum(self.0 - other.0),
            Ordering::Less => Sum(other.0 - self.0),
            Ordering::Equal => Sum(0),
        }
    }
}

impl Debug for Sum {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        // let a = if self.0 > u64::MAX as u128 {
        //     self.0 - u64::MAX as u128
        // } else {
        //     0
        // };
        // let b = self.0 - a;
        //formatter.write_fmt(format_args!("Sum({:X}.{:08X})", a as u64, b as u64))
        formatter.write_fmt(format_args!("Sum({:?})", f64::from(*self)))
    }
}

#[cfg(test)]
impl Sum {
    pub fn approx_eq(&self, other: &Self, acceptable_rounding_error: Option<u64>) -> bool {
        if let Some(acceptable_rounding_error) = acceptable_rounding_error {
            self.abs_diff(other) < Sum(acceptable_rounding_error as u128)
        } else {
            self.0 < 3
        }
    }
}

#[cfg(test)]
impl From<f64> for Sum {
    fn from(arg: f64) -> Self {
        debug_assert!(arg <= 3.0);
        let one = f64::from_u128(u64::MAX as u128).unwrap();
        let val = u128::from_f64(arg * one).unwrap();
        Self(val)
    }
}

//#[cfg(test)]
impl From<Sum> for f64 {
    fn from(arg: Sum) -> Self {
        let one = f64::from_u128(u64::MAX as u128).unwrap();
        f64::from_u128(arg.0).unwrap() / one
    }
}

impl From<Prop> for Sum {
    fn from(arg: Prop) -> Self {
        Self(arg.0 as u128)
    }
}

impl Add for Sum {
    type Output = Self;

    fn add(self, rhs: Self) -> Sum {
        Sum(self.0 + rhs.0)
    }
}

impl Sub for Sum {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        debug_assert!(self.0 >= rhs.0);
        Self(self.0 - rhs.0)
    }
}

impl Div for Sum {
    type Output = Prop;

    fn div(self, rhs: Self) -> Prop {
        let result = if rhs.0 == u64::MAX as u128 {
            self.0
        // Avoid subtraction overflow
        } else if self.0 >= u64::MAX as u128 {
            // do the operation in two parts to avoid multiply overflow problems
            let a = self.0 - u64::MAX as u128;
            let b = self.0 - a;
            let adiv = (a * u64::MAX as u128) / rhs.0;
            let bdiv = (b * u64::MAX as u128) / rhs.0;
            adiv + bdiv
        } else {
            (self.0 * u64::MAX as u128) / rhs.0
        };
        // NB: this requirement enforces policy made about when this operation should be used
        // and is designed to detect policy violations
        debug_assert!(result <= u64::MAX as u128);
        Prop(result as u64)
    }
}

impl Div<u8> for Sum {
    type Output = Prop;

    fn div(self, rhs: u8) -> Prop {
        let result = self.0 as u128 / rhs as u128;
        // this requirement enforces decisions made about when this operation should be used
        debug_assert!(result <= u64::MAX as u128);
        Prop(result as u64)
    }
}

impl Add<Prop> for Sum {
    type Output = Self;

    fn add(self, rhs: Prop) -> Sum {
        Sum(self.0 + rhs.0 as u128)
    }
}

impl Sub<Prop> for Sum {
    type Output = Self;

    fn sub(self, rhs: Prop) -> Self {
        debug_assert!(self.0 >= rhs.0 as u128);
        Self(self.0 - rhs.0 as u128)
    }
}

impl Mul<Prop> for Sum {
    type Output = Self;

    fn mul(self, rhs: Prop) -> Self {
        if rhs.0 == u64::MAX {
            self
        } else if self.0 >= u64::MAX as u128 {
            // NB this is being done in two parts to avoid overflow problems
            let a = self.0 - u64::MAX as u128;
            let b = self.0 - a;
            let amul = (a * rhs.0 as u128) / u64::MAX as u128;
            let bmul = (b * rhs.0 as u128) / u64::MAX as u128;
            Self(amul + bmul)
        } else {
            Self((self.0 * rhs.0 as u128) / u64::MAX as u128)
        }
    }
}
