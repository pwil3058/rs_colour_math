// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    cmp::Ordering,
    fmt::{self, Debug, Formatter},
    ops::{Add, Div, Mul, Rem, Sub},
};

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
        formatter.write_fmt(format_args!("UFDRNumber({:?})", f64::from(*self)))
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

impl_to_from_float!(f64, i128, FDRNumber);
impl_to_from_float!(f32, i128, FDRNumber);

#[derive(Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct UFDRNumber(pub(crate) u128);

impl UFDRNumber {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(u64::MAX as u128);
    pub const TWO: Self = Self(u64::MAX as u128 * 2);
    pub const THREE: Self = Self(u64::MAX as u128 * 3);

    pub(crate) const ONE_PT_5: Self = Self(u64::MAX as u128 * 3 / 2);
    //pub(crate) const ALMOST_ZERO: Self = Self(1);
    pub(crate) const ALMOST_ONE: Self = Self(u64::MAX as u128 - 1);
    pub(crate) const JUST_OVER_ONE: Self = Self(u64::MAX as u128 + 1);
    pub(crate) const ALMOST_TWO: Self = Self(u64::MAX as u128 * 2 - 1);
    pub(crate) const JUST_OVER_TWO: Self = Self(u64::MAX as u128 * 2 + 1);

    pub const SQRT_2: Self = Self(FDRNumber::SQRT_2.0 as u128);

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
        formatter.write_fmt(format_args!("UFDRNumber({:?})", f64::from(*self)))
    }
}

#[cfg(test)]
impl UFDRNumber {
    pub fn approx_eq(&self, other: &Self, acceptable_rounding_error: Option<u64>) -> bool {
        let abs_diff = self.abs_diff(other);
        let scaled_diff = if self >= other {
            if self.0 > 0 {
                (u128::MAX / self.0) * abs_diff.0 / u64::MAX as u128
            } else {
                abs_diff.0
            }
        } else {
            (u128::MAX / other.0) * abs_diff.0 / u64::MAX as u128
        };
        if let Some(acceptable_rounding_error) = acceptable_rounding_error {
            scaled_diff < acceptable_rounding_error as u128
        } else {
            scaled_diff < u64::MAX as u128 / 1_000_000_000_000_000
        }
    }
}

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

impl Div for UFDRNumber {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        match self.0.cmp(&rhs.0) {
            Ordering::Equal => Self::ONE,
            Ordering::Less | Ordering::Greater => self.mul(Self(u128::MAX / rhs.0)),
        }
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

#[cfg(test)]
mod fdrn_tests;
