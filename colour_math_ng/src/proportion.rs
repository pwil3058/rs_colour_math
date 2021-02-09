// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
//use std::cmp::Ordering;
use std::{
    cmp::Ordering,
    fmt::Debug,
    ops::{Add, Div, Mul, Sub},
};

use normalised_angles::{DegreesConst, RadiansConst};
use num_traits::{FromPrimitive, Num, ToPrimitive};
use num_traits_plus::{float_plus::*, NumberConstants};

pub trait Validation: Sized {
    fn is_valid(&self) -> bool;
    fn validated(self) -> Self {
        debug_assert!(self.is_valid());
        self
    }
}

pub trait ProportionConstants: Sized {
    const P_ZERO: Self;
    const P_ONE: Self;
}

impl ProportionConstants for f32 {
    const P_ZERO: Self = 0.0;
    const P_ONE: Self = 1.0;
}

impl ProportionConstants for f64 {
    const P_ZERO: Self = 0.0;
    const P_ONE: Self = 1.0;
}

impl ProportionConstants for u8 {
    const P_ZERO: Self = 0;
    const P_ONE: Self = Self::MAX;
}

impl ProportionConstants for u16 {
    const P_ZERO: Self = 0;
    const P_ONE: Self = Self::MAX;
}

pub trait Number:
    ProportionConstants
    + FromPrimitive
    + ToPrimitive
    + PartialOrd
    + Clone
    + Copy
    + Debug
    + Default
    + PartialEq
    + Num
{
}

impl Number for f32 {}
impl Number for f64 {}
impl Number for u8 {}
impl Number for u16 {}

pub trait Float:
    Number + FloatPlus + DegreesConst + RadiansConst + std::iter::Sum + FloatApproxEq<Self>
{
}

impl Float for f32 {}
impl Float for f64 {}

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

    pub fn is_vp(self) -> bool {
        self <= Self::ONE
    }

    pub fn val_vp(self) -> Self {
        debug_assert!(self.is_vp());
        self
    }

    pub fn is_vs(self) -> bool {
        self <= Self::THREE
    }

    pub fn val_vs(self) -> Self {
        debug_assert!(self.is_vp());
        self
    }

    pub fn approx_eq(&self, other: &Self, max_diff: Option<f64>) -> bool {
        let me = f64::from(*self);
        let other = f64::from(*other);
        me.approx_eq(&other, max_diff)
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Default, PartialOrd)]
pub struct Proportion<N: Number>(pub(crate) N);

impl<N: Number> ProportionConstants for Proportion<N> {
    const P_ZERO: Self = Self(N::P_ZERO);
    const P_ONE: Self = Self(N::P_ONE);
}

impl<N: Number + NumberConstants> NumberConstants for Proportion<N> {
    const BYTES: usize = N::BYTES;
    const DIGITS: u32 = N::DIGITS;
    const MIN: Self = Self(N::MIN);
    const MAX: Self = Self(N::MAX);
    const ZERO: Self = Self(N::ZERO);
    const ONE: Self = Self(N::ONE);
    const TWO: Self = Self(N::TWO);
    const THREE: Self = Self(N::THREE);
}

impl<F: Float> Eq for Proportion<F> {}

impl<F: Float> Ord for Proportion<F> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other)
            .expect("both operands should always ve valid floats")
    }
}

impl<N: Number> Validation for Proportion<N> {
    fn is_valid(&self) -> bool {
        self.0 >= N::P_ZERO || self.0 <= N::P_ONE
    }
}

impl<N: Number> Proportion<N> {
    pub fn value(&self) -> N {
        self.0
    }
}

impl<N: Number> From<N> for Proportion<N> {
    fn from(arg: N) -> Self {
        let proportion = Self(arg);
        debug_assert!(proportion.is_valid());
        proportion
    }
}

impl<N: Number> From<&N> for Proportion<N> {
    fn from(arg: &N) -> Self {
        let proportion = Self(*arg);
        debug_assert!(proportion.is_valid());
        proportion
    }
}

impl<F: Float> From<Sum<F>> for Proportion<F> {
    fn from(sum: Sum<F>) -> Self {
        let proportion = Self(sum.0);
        debug_assert!(proportion.is_valid());
        proportion
    }
}

impl<F: Float + Copy> From<&Sum<F>> for Proportion<F> {
    fn from(sum: &Sum<F>) -> Self {
        let proportion = Self(sum.0);
        debug_assert!(proportion.is_valid());
        proportion
    }
}

macro_rules! impl_op {
    ($op_name:ident, $op_fn:ident, $type:ident) => {
        impl<F: Float> $op_name for $type<F> {
            type Output = Self;

            fn $op_fn(self, rhs: Self) -> Self {
                Self(self.0.$op_fn(rhs.0))
            }
        }
    };
    ($op_name:ident, $op_fn:ident, $type:ident, $rhs:ident) => {
        impl<F: Float> $op_name<$rhs<F>> for $type<F> {
            type Output = Self;

            fn $op_fn(self, rhs: $rhs<F>) -> Self {
                Self(self.0.$op_fn(rhs.0))
            }
        }
    };
}

impl_op!(Sub, sub, Proportion);
impl_op!(Add, add, Proportion);
impl_op!(Div, div, Proportion);
impl_op!(Mul, mul, Proportion);

impl_op!(Sub, sub, Proportion, Sum);
impl_op!(Add, add, Proportion, Sum);
impl_op!(Div, div, Proportion, Sum);
impl_op!(Mul, mul, Proportion, Sum);

impl<F: Float> FloatApproxEq<F> for Proportion<F> {
    fn approx_eq(&self, other: &Self, max_diff: Option<F>) -> bool {
        self.0.approx_eq(&other.0, max_diff)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Default, PartialOrd)]
pub struct Sum<F: Float>(pub(crate) F);

impl<F: Float> Eq for Sum<F> {}

impl<F: Float> Ord for Sum<F> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other)
            .expect("both operands should always ve valid floats")
    }
}

impl<F: Float> ProportionConstants for Sum<F> {
    const P_ZERO: Self = Self(F::P_ZERO);
    const P_ONE: Self = Self(F::P_ONE);
}

impl<F: Float> Validation for Sum<F> {
    fn is_valid(&self) -> bool {
        self.0 >= F::ZERO && self.0 <= F::THREE
    }
}

impl<F: Float> NumberConstants for Sum<F> {
    const BYTES: usize = F::BYTES;
    const DIGITS: u32 = F::DIGITS;
    const MIN: Self = Self(F::MIN);
    const MAX: Self = Self(F::MAX);
    const ZERO: Self = Self(F::ZERO);
    const ONE: Self = Self(F::ONE);
    const TWO: Self = Self(F::TWO);
    const THREE: Self = Self(F::THREE);
}

impl<F: Float> Sum<F> {
    pub fn value(&self) -> F {
        self.0
    }
}

impl<F: Float> From<&[Proportion<F>; 3]> for Sum<F> {
    fn from(array: &[Proportion<F>; 3]) -> Self {
        debug_assert!(array[0].is_valid() && array[1].is_valid() && array[2].is_valid());
        Self((array[0].0 + array[1].0 + array[2].0).min(F::THREE))
    }
}

impl<F: Float> From<Proportion<F>> for Sum<F> {
    fn from(proportion: Proportion<F>) -> Self {
        debug_assert!(proportion.is_valid());
        Self(proportion.0)
    }
}

impl_op!(Sub, sub, Sum);
impl_op!(Add, add, Sum);
impl_op!(Div, div, Sum);
impl_op!(Mul, mul, Sum);

impl_op!(Sub, sub, Sum, Proportion);
impl_op!(Add, add, Sum, Proportion);
impl_op!(Div, div, Sum, Proportion);
impl_op!(Mul, mul, Sum, Proportion);

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

impl Validation for Chroma {
    fn is_valid(&self) -> bool {
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
