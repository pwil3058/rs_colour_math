// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
//use std::cmp::Ordering;
use std::{
    fmt::Debug,
    ops::{Add, Div, Mul, Sub},
};

use num_traits::FromPrimitive;
use num_traits_plus::float_plus::*;

pub trait ProportionValidation: Sized + Copy {
    fn is_vp(self) -> bool;

    fn validated_p(self) -> Self {
        debug_assert!(self.is_vp());
        self
    }
}

pub trait SumValidation: Sized + Copy {
    fn is_vs(self) -> bool;

    fn validated_s(self) -> Self {
        debug_assert!(self.is_vs());
        self
    }
}

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq, Default, PartialOrd, Ord,
)]
pub struct UFDFraction(pub(crate) u64);

impl UFDFraction {
    const DENOM: u64 = u32::MAX as u64;
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(Self::DENOM);
    pub const TWO: Self = Self(Self::DENOM * 2);
    pub const THREE: Self = Self(Self::DENOM * 3);

    pub fn approx_eq(&self, other: &Self, max_diff: Option<f64>) -> bool {
        let me = f64::from(*self);
        let other = f64::from(*other);
        me.approx_eq(&other, max_diff)
    }
}

impl ProportionValidation for UFDFraction {
    fn is_vp(self) -> bool {
        self <= Self::ONE
    }
}

impl SumValidation for UFDFraction {
    fn is_vs(self) -> bool {
        self <= Self::THREE
    }
}

macro_rules! impl_ufdr_add_sub {
    ($op_name:ident, $op_fn:ident) => {
        impl $op_name for UFDFraction {
            type Output = Self;

            fn $op_fn(self, rhs: Self) -> Self {
                Self(self.0.$op_fn(rhs.0))
            }
        }
    };
}

impl_ufdr_add_sub!(Add, add);
impl_ufdr_add_sub!(Sub, sub);

impl Div for UFDFraction {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        let mut ws: u128 = self.0 as u128 * Self::DENOM as u128;
        ws /= rhs.0 as u128;
        Self(ws as u64)
    }
}

impl Mul for UFDFraction {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let mut ws: u128 = self.0 as u128 * rhs.0 as u128;
        ws /= Self::DENOM as u128;
        Self(ws as u64)
    }
}

impl From<f64> for UFDFraction {
    fn from(arg: f64) -> Self {
        let one = f64::from_u64(Self::DENOM).unwrap();
        let val = u64::from_f64(arg * one).unwrap();
        Self(val)
    }
}

impl From<UFDFraction> for f64 {
    fn from(arg: UFDFraction) -> Self {
        let one = f64::from_u64(UFDFraction::DENOM).unwrap();
        f64::from_u64(arg.0).unwrap() / one
    }
}

impl From<f32> for UFDFraction {
    fn from(arg: f32) -> Self {
        let one = f32::from_u64(Self::DENOM).unwrap();
        let val = u64::from_f32(arg * one).unwrap();
        Self(val)
    }
}

impl From<UFDFraction> for f32 {
    fn from(arg: UFDFraction) -> Self {
        let one = f32::from_u64(UFDFraction::DENOM).unwrap();
        f32::from_u64(arg.0).unwrap() / one
    }
}

macro_rules! impl_unsigned_to_from {
    ($unsigned:ty) => {
        impl From<$unsigned> for UFDFraction {
            fn from(arg: $unsigned) -> Self {
                let val = arg as u64 * Self::DENOM / <$unsigned>::MAX as u64;
                Self(val)
            }
        }

        impl From<UFDFraction> for $unsigned {
            fn from(arg: UFDFraction) -> Self {
                debug_assert!(arg <= UFDFraction::ONE);
                let val = arg.0 * <$unsigned>::MAX as u64 / UFDFraction::DENOM;
                val as $unsigned
            }
        }
    };
}

impl_unsigned_to_from!(u8);
impl_unsigned_to_from!(u16);

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Chroma {
    Shade(UFDFraction),
    Tint(UFDFraction),
}

impl Chroma {
    pub const ZERO: Self = Self::Shade(UFDFraction::ZERO);
    pub const ONE: Self = Self::Tint(UFDFraction::ONE);

    pub fn is_zero(&self) -> bool {
        match self {
            Chroma::Shade(proportion) => *proportion == UFDFraction::ZERO,
            Chroma::Tint(proportion) => *proportion == UFDFraction::ZERO,
        }
    }

    pub fn proportion(&self) -> UFDFraction {
        match self {
            Chroma::Shade(proportion) => *proportion,
            Chroma::Tint(proportion) => *proportion,
        }
    }

    pub fn approx_eq(&self, other: &Self, max_diff: Option<f64>) -> bool {
        match self {
            Chroma::Shade(proportion) => match other {
                Chroma::Shade(other_proportion) => proportion.approx_eq(other_proportion, max_diff),
                Chroma::Tint(_) => false,
            },
            Chroma::Tint(proportion) => match other {
                Chroma::Shade(_) => false,
                Chroma::Tint(other_proportion) => proportion.approx_eq(other_proportion, max_diff),
            },
        }
    }
}

impl ProportionValidation for Chroma {
    fn is_vp(self) -> bool {
        match self {
            Chroma::Shade(proportion) => proportion.is_vp(),
            Chroma::Tint(proportion) => proportion.is_vp(),
        }
    }
}

#[cfg(test)]
mod proportion_tests {
    use super::*;
    use num_traits_plus::assert_approx_eq;

    #[test]
    fn to_from_ufdf() {
        assert_eq!(UFDFraction::from(1.0_f64), UFDFraction::ONE);
        assert_eq!(f64::from(UFDFraction::ONE), 1.0);
        for f in &[0.0f64, 24.0, 0.8, 0.5, 2.0] {
            assert_approx_eq!(f64::from(UFDFraction::from(*f)), *f, 0.000_000_001);
        }
        assert_approx_eq!(UFDFraction::from(1.0_f32), UFDFraction::ONE, 0.000_000_001);
        assert_eq!(f32::from(UFDFraction::ONE), 1.0);
        for f in &[0.0f32, 24.0, 0.8, 0.5, 2.0] {
            assert_approx_eq!(f32::from(UFDFraction::from(*f)), *f, 0.000_000_001);
        }
        assert_eq!(UFDFraction::from(u8::MAX), UFDFraction::ONE);
        assert_eq!(u8::from(UFDFraction::ONE), u8::MAX);
        for u in 0_u8..u8::MAX {
            assert_eq!(u8::from(UFDFraction::from(u)), u);
        }
        assert_eq!(UFDFraction::from(u16::MAX), UFDFraction::ONE);
        assert_eq!(u16::from(UFDFraction::ONE), u16::MAX);
        for u in 0_u16..u16::MAX {
            assert_eq!(u16::from(UFDFraction::from(u)), u);
        }
    }

    #[test]
    fn add_ufdf() {
        for [a, b] in &[[0.0f64, 1.0], [24.0, 0.5], [0.8, 0.5]] {
            let expected = UFDFraction::from(a + b);
            println!("{:?} | {:?} {:?}", a, b, expected);
            let result = UFDFraction::from(*a) + UFDFraction::from(*b);
            assert_eq!(result, expected);
            println!(
                "ADD{:?} == {:?} == {:?} == {:?}",
                result,
                expected,
                f64::from(result),
                f64::from(expected)
            );
            assert_approx_eq!(&f64::from(result), &(a + b), 0.000_000_001);
        }
    }

    #[test]
    fn sub_ufdf() {
        for [a, b] in &[[2.0f64, 1.0], [24.0, 0.5], [0.8, 0.5]] {
            let expected = UFDFraction::from(a - b);
            println!("{:?} | {:?} {:?}", a, b, expected);
            let result = UFDFraction::from(*a) - UFDFraction::from(*b);
            println!(
                "SUB{:?} == {:?} == {:?} == {:?}",
                result,
                expected,
                f64::from(result),
                f64::from(expected)
            );
            assert_approx_eq!(result, expected, 0.000_000_001);
            assert_approx_eq!(f64::from(result), &(a - b), 0.000_000_001);
        }
    }

    #[test]
    fn div_ufdf() {
        for [a, b] in &[[2.0f64, 4.0], [24.0, 0.5], [0.8, 0.5]] {
            let expected = UFDFraction::from(a / b);
            println!("{:?} | {:?} {:?}", a, b, expected);
            let result = UFDFraction::from(*a) / UFDFraction::from(*b);
            println!(
                "DIV {:?} == {:?} == {:?} == {:?}",
                result,
                expected,
                f64::from(result),
                f64::from(expected)
            );
            assert_approx_eq!(result, expected, 0.000_000_001);
            assert_approx_eq!(f64::from(result), &(a / b), 0.000_000_01);
        }
    }

    #[test]
    fn mul_ufdf() {
        for [a, b] in &[[2.0f64, 4.0], [24.0, 0.5], [0.8, 0.5]] {
            let expected = UFDFraction::from(a * b);
            println!("{:?} | {:?} {:?}", a, b, expected);
            let result = UFDFraction::from(*a) * UFDFraction::from(*b);
            println!(
                "DIV {:?} == {:?} == {:?} == {:?}",
                result,
                expected,
                f64::from(result),
                f64::from(expected)
            );
            assert_approx_eq!(result, expected, 0.000_000_001);
            assert_approx_eq!(f64::from(result), &(a * b), 0.000_000_01);
        }
    }
}
