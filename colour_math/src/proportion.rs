// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
//use std::cmp::Ordering;
use std::{
    fmt::Debug,
    ops::{Add, Div, Mul, Sub},
};

use num_traits::{FromPrimitive, Num, NumOps};
use num_traits_plus::float_plus::{FloatApproxEq, FloatPlus};
use num_traits_plus::NumberConstants;

pub trait Validation {
    fn is_valid(&self) -> bool;
}

pub trait PropTraits:
    NumberConstants
    + ProportionConstants
    + PartialOrd
    + Ord
    + NumOps
    + Sized
    + Num
    + FromPrimitive
    + Clone
    + Copy
{
}

pub trait ProportionConstants {
    const PROPORTION_MIN: Self;
    const PROPORTION_MAX: Self;
}

impl ProportionConstants for f32 {
    const PROPORTION_MIN: Self = 0.0;
    const PROPORTION_MAX: Self = 1.0;
}

impl ProportionConstants for f64 {
    const PROPORTION_MIN: Self = 0.0;
    const PROPORTION_MAX: Self = 1.0;
}

impl ProportionConstants for u8 {
    const PROPORTION_MIN: Self = 0;
    const PROPORTION_MAX: Self = u8::MAX;
}

impl ProportionConstants for u16 {
    const PROPORTION_MIN: Self = 0;
    const PROPORTION_MAX: Self = u16::MAX;
}

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq, Default, PartialOrd, Ord,
)]
pub struct Proportion<P: PropTraits>(P);

impl<P: PropTraits> Proportion<P> {
    pub const ZERO: Self = Self(P::PROPORTION_MIN);
    pub const ONE: Self = Self(P::PROPORTION_MAX);
}

impl<P: PropTraits> Validation for Proportion<P> {
    fn is_valid(&self) -> bool {
        self.0 >= P::PROPORTION_MIN || self.0 <= P::PROPORTION_MAX
    }
}

impl<P: PropTraits> From<Sum<P>> for Proportion<P> {
    fn from(sum: Sum<P>) -> Self {
        debug_assert!(sum.is_valid());
        let proportion = Self(sum.0);
        debug_assert!(proportion.is_valid());
        proportion
    }
}

impl<P: PropTraits> Proportion<P> {
    pub fn value(&self) -> P {
        self.0
    }
}

impl<P: PropTraits> Sub for Proportion<P> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        debug_assert!(rhs.is_valid());
        let val = Self(self.0 - rhs.0);
        debug_assert!(val.is_valid());
        val
    }
}

impl<P: PropTraits> Add for Proportion<P> {
    type Output = Sum<P>;

    fn add(self, rhs: Self) -> Self::Output {
        debug_assert!(rhs.is_valid());
        let val = Sum(self.0 + rhs.0);
        debug_assert!(val.is_valid());
        val
    }
}

impl<P: PropTraits> Mul<u8> for Proportion<P> {
    type Output = Sum<P>;

    fn mul(self, scalar: u8) -> Self::Output {
        debug_assert!(scalar >= 1 && scalar <= 3);
        let val = Sum(self.0 * P::from_u8(scalar).unwrap());
        debug_assert!(val.is_valid());
        val
    }
}

impl<P: PropTraits> Mul for Proportion<P> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        debug_assert!(rhs.is_valid());
        let val = Self(self.0 * rhs.0);
        debug_assert!(val.is_valid());
        val
    }
}

impl<P: PropTraits> Div for Proportion<P> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        debug_assert!(rhs.is_valid());
        let val = Self(self.0 / rhs.0);
        debug_assert!(val.is_valid());
        val
    }
}

impl<P: PropTraits> Div<u8> for Proportion<P> {
    type Output = Self;

    fn div(self, rhs: u8) -> Self::Output {
        let val = Self(self.0 / P::from_u8(rhs).unwrap());
        debug_assert!(val.is_valid());
        val
    }
}

impl<P: PropTraits> From<P> for Proportion<P> {
    fn from(arg: P) -> Self {
        let proportion = Self(arg);
        debug_assert!(proportion.is_valid());
        proportion
    }
}

impl<P: PropTraits> FloatApproxEq<P> for Proportion<P>
where
    P: FloatApproxEq<P> + FloatPlus,
{
    fn approx_eq(&self, other: &Self, max_diff: Option<P>) -> bool {
        self.0.approx_eq(&other.0, max_diff)
    }
}

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq, Default, PartialOrd, Ord,
)]
pub struct Sum<P: PropTraits>(P);

impl<P: PropTraits> Validation for Sum<P> {
    fn is_valid(&self) -> bool {
        self.0 >= P::ZERO && self.0 <= P::ONE
    }
}

impl<P: PropTraits> Sum<P> {
    pub const ZERO: Self = Self(P::PROPORTION_MIN);
    pub const ONE: Self = Self(P::PROPORTION_MAX);
    pub const TWO: Self = Self(P::TWO);
    pub const THREE: Self = Self(P::THREE);
}

impl<P: PropTraits> Sum<P> {
    pub fn value(&self) -> P {
        self.0
    }
}

impl<P: PropTraits> From<&[Proportion<P>; 3]> for Sum<P> {
    fn from(array: &[Proportion<P>; 3]) -> Self {
        let sum = (array[0].value() + array[1].value() + array[2].value()).min(P::THREE);
        Self(sum)
    }
}

impl<P: PropTraits> From<Proportion<P>> for Sum<P> {
    fn from(proportion: Proportion<P>) -> Self {
        debug_assert!(proportion.is_valid());
        Self(proportion.0)
    }
}

impl<P: PropTraits> Mul<Proportion<P>> for Sum<P> {
    type Output = Self;

    fn mul(self, proportion: Proportion<P>) -> Self::Output {
        debug_assert!(proportion.is_valid());
        let val = Self(self.0 * proportion.0);
        debug_assert!(val.is_valid());
        val
    }
}

impl<P: PropTraits> Div<u8> for Sum<P> {
    type Output = Proportion<P>;

    fn div(self, scalar: u8) -> Self::Output {
        debug_assert!(scalar >= 1 && scalar <= 3);
        let proportion = Proportion(self.0 / P::from_u8(scalar).unwrap());
        debug_assert!(proportion.is_valid());
        proportion
    }
}

impl<P: PropTraits> Sub for Sum<P> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        debug_assert!(rhs.is_valid());
        let val = Self(self.0 - rhs.0);
        debug_assert!(val.is_valid());
        val
    }
}

impl<P: PropTraits> Sub<Proportion<P>> for Sum<P> {
    type Output = Self;

    fn sub(self, proportion: Proportion<P>) -> Self::Output {
        debug_assert!(proportion.is_valid());
        let val = Self(self.0 - proportion.0);
        debug_assert!(val.is_valid());
        val
    }
}

impl<P: PropTraits> Add<Proportion<P>> for Sum<P> {
    type Output = Self;

    fn add(self, proportion: Proportion<P>) -> Self::Output {
        debug_assert!(proportion.is_valid());
        let val = Self(self.0 + proportion.0);
        debug_assert!(val.is_valid());
        val
    }
}

impl<P: PropTraits> Add for Sum<P> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        debug_assert!(rhs.is_valid());
        let val = Self(self.0 + rhs.0);
        debug_assert!(val.is_valid());
        val
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Chroma<P: PropTraits> {
    Shade(Proportion<P>),
    Tint(Proportion<P>),
}

impl<P: PropTraits + ProportionConstants> Chroma<P> {
    pub const ZERO: Self = Self::Shade(Proportion::ZERO);
    pub const ONE: Self = Self::Tint(Proportion::ONE);
}

impl<P: PropTraits + ProportionConstants> Validation for Chroma<P> {
    fn is_valid(&self) -> bool {
        match self {
            Chroma::Shade(val) => val.0 >= P::ZERO && val.0 <= P::ONE,
            Chroma::Tint(val) => val.0 >= P::ZERO && val.0 <= P::ONE,
        }
    }
}
