// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::ops::Neg;
use std::{
    cmp::Ordering,
    fmt::{self, Debug, Formatter},
    ops::{Add, Div, Mul, Rem, Sub},
};

use crate::debug::{AbsDiff, ApproxEq, PropDiff};

macro_rules! impl_to_from_float {
    ($float:ty, $core:ty, $number:ty) => {
        impl From<$float> for $number {
            fn from(arg: $float) -> Self {
                Self((arg * u64::MAX as $float) as $core)
            }
        }

        impl From<$number> for $float {
            fn from(arg: $number) -> Self {
                arg.0 as $float / u64::MAX as $float
            }
        }
    };
}

#[derive(Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct FDRNumber(pub(crate) i128);

// NB: ONE is the same value as for UFDRNumber
impl FDRNumber {
    pub const ZERO: Self = Self(0);
    // u64::MAX: 18446744073709551615
    pub const ONE: Self = Self(u64::MAX as i128);
    // SQRT_2: 1.41421356237309504880168872420969808
    pub const SQRT_2: Self =
        Self(u64::MAX as i128 + 4142135623730950488 * u64::MAX as i128 / 10000000000000000000);
    pub const SQRT_3: Self =
        Self(u64::MAX as i128 + 17320508075688772 * u64::MAX as i128 / 10000000000000000);

    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }

    pub fn is_valid_sum(self) -> bool {
        self >= Self::ZERO && self <= Self::ONE * 3
    }

    pub fn is_hue_valid(self) -> bool {
        self > Self::ZERO && self < Self::ONE * 3
    }

    pub fn is_proportion(self) -> bool {
        self >= Self::ZERO && self <= Self::ONE
    }

    pub fn abs_diff(&self, other: &Self) -> FDRNumber {
        match self.cmp(other) {
            Ordering::Greater => FDRNumber(self.0 - other.0),
            Ordering::Less => FDRNumber(other.0 - self.0),
            Ordering::Equal => FDRNumber(0),
        }
    }
}

impl Debug for FDRNumber {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        let int = self.0 / u64::MAX as i128;
        let frac = self.0 % u64::MAX as i128;
        formatter.write_fmt(format_args!("UFDRNumber({:X}.{:016X})", int, frac.abs()))
        //formatter.write_fmt(format_args!("UFDRNumber({:?})", f64::from(*self)))
    }
}

impl fmt::UpperHex for FDRNumber {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, formatter)
    }
}

#[cfg(test)]
impl FDRNumber {
    pub fn approx_eq(&self, other: &Self, acceptable_rounding_error: Option<u64>) -> bool {
        let abs_diff = self.abs_diff(other);
        let scaled_diff = if self.0.abs() >= other.0.abs() {
            if self.0.abs() > 0 {
                (u128::MAX / self.0.abs() as u128) * abs_diff.0 as u128 / u64::MAX as u128
            } else {
                abs_diff.0 as u128
            }
        } else {
            (u128::MAX / other.0.abs() as u128) * abs_diff.0 as u128 / u64::MAX as u128
        };
        if let Some(acceptable_rounding_error) = acceptable_rounding_error {
            scaled_diff < acceptable_rounding_error as u128
        } else {
            scaled_diff < u64::MAX as u128 / 1_000_000_000_000_000
        }
    }
}

impl Div for FDRNumber {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        if rhs == Self::ONE {
            self
        } else {
            let val: Self = UFDRNumber::from(self.abs())
                .div(UFDRNumber::from(rhs.abs()))
                .into();
            if self.0.is_negative() {
                if rhs.0.is_negative() {
                    val
                } else {
                    val.neg()
                }
            } else if rhs.0.is_negative() {
                val.neg()
            } else {
                val
            }
        }
    }
}

impl Div<i32> for FDRNumber {
    type Output = Self;

    fn div(self, rhs: i32) -> Self {
        Self(self.0.div(rhs as i128))
    }
}

impl Add for FDRNumber {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0.add(other.0))
    }
}

impl Sub for FDRNumber {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0.sub(other.0))
    }
}

impl Neg for FDRNumber {
    type Output = Self;

    fn neg(self) -> Self {
        Self(self.0.neg())
    }
}

impl Mul for FDRNumber {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let val: Self = UFDRNumber::from(self.abs())
            .mul(UFDRNumber::from(rhs.abs()))
            .into();
        if self.0.is_negative() {
            if rhs.0.is_negative() {
                val
            } else {
                val.neg()
            }
        } else if rhs.0.is_negative() {
            val.neg()
        } else {
            val
        }
    }
}

impl Mul<i32> for FDRNumber {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self {
        Self(self.0.mul(rhs as i128))
    }
}

impl From<UFDRNumber> for FDRNumber {
    fn from(unsigned: UFDRNumber) -> Self {
        Self(unsigned.0 as i128)
    }
}

impl_to_from_float!(f64, i128, FDRNumber);
impl_to_from_float!(f32, i128, FDRNumber);

macro_rules! impl_to_from_signed {
    (i128) => {
        impl From<i128> for FDRNumber {
            fn from(signed: i128) -> Self {
                Self(signed)
            }
        }

        impl From<FDRNumber> for i128 {
            fn from(arg: FDRNumber) -> Self {
                arg.0
            }
        }
    };
    ($signed:ty) => {
        impl From<$signed> for FDRNumber {
            fn from(signed: $signed) -> Self {
                Self(signed as i128 * u64::max as i128)
            }
        }

        impl From<FDRNumber> for $signed {
            fn from(arg: FDRNumber) -> Self {
                (arg.0 / u64::MAX as i128) as $signed
            }
        }
    };
}

impl_to_from_signed!(i8);
impl_to_from_signed!(i16);
impl_to_from_signed!(i32);
impl_to_from_signed!(i64);
impl_to_from_signed!(i128);

#[derive(Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct UFDRNumber(pub(crate) u128);

impl UFDRNumber {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(u64::MAX as u128);
    pub const TWO: Self = Self(u64::MAX as u128 * 2);
    pub const THREE: Self = Self(u64::MAX as u128 * 3);

    pub const SQRT_2: Self = Self(FDRNumber::SQRT_2.0 as u128);
    pub const SQRT_3: Self = Self(FDRNumber::SQRT_3.0 as u128);

    pub fn is_valid_sum(self) -> bool {
        self <= Self::THREE
    }

    pub fn is_hue_valid(self) -> bool {
        self > Self::ZERO && self < Self::THREE
    }

    pub fn is_proportion(self) -> bool {
        self <= Self::ONE
    }

    pub fn to_prop(&self) -> Prop {
        debug_assert!(self.is_proportion());
        Prop(self.0 as u64)
    }
}

impl Debug for UFDRNumber {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        let int = self.0 / u64::MAX as u128;
        let frac = self.0 % u64::MAX as u128;
        formatter.write_fmt(format_args!("UFDRNumber({int:X}.{frac:016X})"))
    }
}

impl fmt::UpperHex for UFDRNumber {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, formatter)
    }
}

impl AbsDiff for UFDRNumber {}

impl PropDiff for UFDRNumber {
    fn prop_diff(&self, other: &Self) -> Option<Prop> {
        if self.is_proportion() && other.is_proportion() {
            Some(self.abs_diff(other).into())
        } else {
            let denom = self.max(other);
            if *denom == UFDRNumber::ZERO {
                Some(Prop::ZERO)
            } else {
                Some(Prop::from(self.abs_diff(other) / *denom))
            }
        }
    }
}

impl ApproxEq for UFDRNumber {}

impl From<FDRNumber> for UFDRNumber {
    fn from(signed: FDRNumber) -> Self {
        debug_assert!(signed.0 >= 0);
        Self(signed.0 as u128)
    }
}

impl_to_from_float!(f64, u128, UFDRNumber);
impl_to_from_float!(f32, u128, UFDRNumber);

impl From<i32> for UFDRNumber {
    fn from(signed: i32) -> Self {
        debug_assert!(signed > 0);
        Self(signed as u128 * u64::MAX as u128)
    }
}

macro_rules! impl_to_from_unsigned {
    (u128) => {
        impl From<u128> for UFDRNumber {
            fn from(unsigned: u128) -> Self {
                Self(unsigned)
            }
        }

        impl From<UFDRNumber> for u128 {
            fn from(arg: UFDRNumber) -> Self {
                arg.0
            }
        }
    };
    ($unsigned:ty) => {
        impl From<$unsigned> for UFDRNumber {
            fn from(unsigned: $unsigned) -> Self {
                Self(unsigned as u128 * u64::max as u128)
            }
        }

        impl From<UFDRNumber> for $unsigned {
            fn from(arg: UFDRNumber) -> Self {
                (arg.0 / u64::MAX as u128) as $unsigned
            }
        }
    };
}

impl_to_from_unsigned!(u8);
impl_to_from_unsigned!(u16);
impl_to_from_unsigned!(u32);
impl_to_from_unsigned!(u64);
impl_to_from_unsigned!(u128);

impl Add for UFDRNumber {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl Sub for UFDRNumber {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        debug_assert!(self.0 >= rhs.0);
        Self(self.0 - rhs.0)
    }
}

impl Mul for UFDRNumber {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let one = u64::MAX as u128;
        let l_int = self.0 / one;
        let l_rem = self.0 % one;
        let r_int = rhs.0 / one;
        let r_rem = rhs.0 % one;
        Self(l_int * r_int * one + l_int * r_rem + r_int * l_rem + r_rem * l_rem / one)
    }
}

impl Mul<i32> for UFDRNumber {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self {
        debug_assert!(rhs >= 0);
        Self(self.0.mul(rhs as u128))
    }
}

impl Div for UFDRNumber {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        match rhs.0 % u64::MAX as u128 {
            // NB: faster AND more accurate
            0 => Self(self.0 / (rhs.0 / u64::MAX as u128)),
            _ => match self.0.cmp(&rhs.0) {
                Ordering::Equal => Self::ONE,
                Ordering::Less | Ordering::Greater => self.mul(Self(u128::MAX / rhs.0)),
            },
        }
    }
}

impl Div<i32> for UFDRNumber {
    type Output = Self;

    fn div(self, rhs: i32) -> Self {
        debug_assert!(rhs >= 0);
        Self(self.0.div(rhs as u128))
    }
}

impl Rem for UFDRNumber {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self {
        Self(self.0 % rhs.0)
    }
}

impl Rem<i32> for UFDRNumber {
    type Output = Self;

    fn rem(self, rhs: i32) -> Self {
        debug_assert!(rhs >= 0);
        Self(self.0 % rhs as u128)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Prop(pub(crate) u64);

impl Prop {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(u64::MAX);

    //pub(crate) const ALMOST_ZERO: Self = Self(1);
    pub(crate) const ALMOST_ONE: Self = Self(u64::MAX - 1);
    pub(crate) const HALF: Self = Self(u64::MAX / 2);
}

impl AbsDiff for Prop {}

impl PropDiff for Prop {
    fn prop_diff(&self, other: &Self) -> Option<Prop> {
        Some(self.abs_diff(other))
    }
}

impl ApproxEq for Prop {}

impl Debug for Prop {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        let int = self.0 / u64::MAX;
        let frac = self.0 % u64::MAX;
        formatter.write_fmt(format_args!("Prop({int:X}.{frac:016X})"))
    }
}

impl fmt::UpperHex for Prop {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, formatter)
    }
}

pub trait IntoProp: Sized + Copy + Into<Prop> {
    fn into_prop(self) -> Prop {
        self.into()
    }
}

impl From<[u64; 2]> for Prop {
    fn from(fraction: [u64; 2]) -> Self {
        debug_assert!(fraction[1] >= fraction[0]);
        Self(((fraction[0] as u128 * u64::MAX as u128) / fraction[1] as u128) as u64)
    }
}

#[macro_export]
macro_rules! impl_prop_to_from_float {
    ($float:ty, $number:ty) => {
        impl From<$float> for $number {
            fn from(arg: $float) -> Self {
                debug_assert!((0.0..=1.0).contains(&arg));
                // NB: watch out for floating point not being proper reals
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

impl IntoProp for f32 {}

impl IntoProp for f64 {}

impl_prop_to_from_float!(f32, Prop);
impl_prop_to_from_float!(f64, Prop);

#[macro_export]
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

impl IntoProp for UFDRNumber {}

impl IntoProp for FDRNumber {}

#[macro_export]
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

        impl IntoProp for u64 {}
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

        impl IntoProp for $unsigned {}
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
        if rhs == Self::ONE {
            self
        } else {
            Self((((u128::MAX / rhs.0 as u128) * self.0 as u128) / u64::MAX as u128) as u64)
        }
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

impl Div<Prop> for UFDRNumber {
    type Output = Self;

    fn div(self, rhs: Prop) -> Self {
        if rhs == Prop::ONE {
            self
        } else {
            match self.0.cmp(&(rhs.0 as u128)) {
                Ordering::Equal => Self::ONE,
                Ordering::Less | Ordering::Greater => self.mul(Self(u128::MAX / rhs.0 as u128)),
            }
        }
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

#[cfg(test)]
mod fdrn_tests;
