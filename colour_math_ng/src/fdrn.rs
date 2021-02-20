// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use num_traits::FromPrimitive;
use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::ops::{Add, Div, Mul, Rem, Sub};

#[derive(
    Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default, Debug,
)]
pub struct FDRNumber(pub(crate) i128);

// NB: ONE is the same value as for UFDRNumber
impl FDRNumber {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(u64::MAX as i128);

    pub fn abs_diff(&self, other: &Self) -> FDRNumber {
        match self.cmp(other) {
            Ordering::Greater => FDRNumber(self.0 - other.0),
            Ordering::Less => FDRNumber(other.0 - self.0),
            Ordering::Equal => FDRNumber(0),
        }
    }
}

#[cfg(test)]
impl FDRNumber {
    pub fn approx_eq(&self, other: &Self, acceptable_rounding_error: Option<u64>) -> bool {
        if let Some(acceptable_rounding_error) = acceptable_rounding_error {
            self.abs_diff(other).0 < acceptable_rounding_error as i128
        } else {
            self.abs_diff(other).0 < 3
        }
    }
}

impl Div<u8> for FDRNumber {
    type Output = Self;

    fn div(self, rhs: u8) -> Self {
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

impl Mul<u8> for FDRNumber {
    type Output = Self;

    fn mul(self, rhs: u8) -> Self {
        Self(self.0.mul(rhs as i128))
    }
}

impl From<UFDRNumber> for FDRNumber {
    fn from(unsigned: UFDRNumber) -> Self {
        Self(unsigned.0 as i128)
    }
}

impl From<f64> for FDRNumber {
    fn from(arg: f64) -> Self {
        let one = f64::from_i128(u64::MAX as i128).unwrap();
        let val = i128::from_f64(arg * one).unwrap();
        Self(val)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct UFDRNumber(pub(crate) u128);

impl UFDRNumber {
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

    pub fn abs_diff(&self, other: &Self) -> UFDRNumber {
        match self.cmp(other) {
            Ordering::Greater => UFDRNumber(self.0 - other.0),
            Ordering::Less => UFDRNumber(other.0 - self.0),
            Ordering::Equal => UFDRNumber(0),
        }
    }
}

impl Debug for UFDRNumber {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        // let a = if self.0 > u64::MAX as u128 {
        //     self.0 - u64::MAX as u128
        // } else {
        //     0
        // };
        // let b = self.0 - a;
        //formatter.write_fmt(format_args!("UFDRNumber({:X}.{:08X})", a as u64, b as u64))
        formatter.write_fmt(format_args!("UFDRNumber({:?})", f64::from(*self)))
    }
}

#[cfg(test)]
impl UFDRNumber {
    pub fn approx_eq(&self, other: &Self, acceptable_rounding_error: Option<u64>) -> bool {
        if let Some(acceptable_rounding_error) = acceptable_rounding_error {
            self.abs_diff(other).0 < acceptable_rounding_error as u128
        } else {
            self.abs_diff(other).0 < 3
        }
    }
}

impl From<f64> for UFDRNumber {
    fn from(arg: f64) -> Self {
        let one = f64::from_u128(u64::MAX as u128).unwrap();
        let val = u128::from_f64(arg * one).unwrap();
        Self(val)
    }
}

impl From<UFDRNumber> for f64 {
    fn from(arg: UFDRNumber) -> Self {
        let one = f64::from_u128(u64::MAX as u128).unwrap();
        f64::from_u128(arg.0).unwrap() / one
    }
}

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

impl Div for UFDRNumber {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
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
        Self(result)
    }
}

impl Div<u8> for UFDRNumber {
    type Output = Self;

    fn div(self, rhs: u8) -> Self {
        Self(self.0 / rhs as u128)
    }
}

impl Rem for UFDRNumber {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self {
        Self(self.0 % rhs.0)
    }
}
