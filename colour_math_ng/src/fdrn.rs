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

impl_to_from_float!(f64, i128, FDRNumber);
impl_to_from_float!(f32, i128, FDRNumber);

#[derive(Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct UFDRNumber(pub(crate) u128);

impl UFDRNumber {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(u64::MAX as u128);
    pub const TWO: Self = Self(u64::MAX as u128 * 2);
    pub const THREE: Self = Self(u64::MAX as u128 * 3);

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
        if let Some(acceptable_rounding_error) = acceptable_rounding_error {
            self.abs_diff(other).0 < acceptable_rounding_error as u128
        } else {
            self.abs_diff(other).0 < 3
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

#[cfg(test)]
mod fdrn_tests {
    use super::*;

    #[test]
    fn sqrt_2() {
        assert_eq!(f64::from(FDRNumber::SQRT_2), std::f64::consts::SQRT_2);
        assert_eq!(f64::from(UFDRNumber::SQRT_2), std::f64::consts::SQRT_2);
    }
}
