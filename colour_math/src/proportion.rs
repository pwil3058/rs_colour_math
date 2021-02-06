// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
//use std::cmp::Ordering;
use std::{
    fmt::Debug,
    ops::{Add, Div, Mul, Sub},
};

use num_traits_plus::{
    float_plus::{FloatApproxEq, FloatPlus},
    num_traits::Num,
};

pub trait Validation {
    fn is_valid(&self) -> bool;
}

pub trait PropTraits {}

pub trait ProportionConstants: Clone + Copy {
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
        let val = Sum(self.0 * P::from(scalar).unwrap());
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
        debug_assert!(rhs.is_valid());
        let val = Self(self.0 / P::from(rhs.0).unwrap());
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
    P: FloatApproxEq<P>,
{
    fn approx_eq(&self, other: &Self, max_diff: Option<P>) -> bool {
        self.0.approx_eq(&other.0, max_diff)
    }
}

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq, Default, PartialOrd, Ord,
)]
pub struct Sum<S: PropTraits>(S);

impl<S: PropTraits> Validation for Sum<S> {
    fn is_valid(&self) -> bool {
        self.0 >= S::ZERO && self.0 <= S::ONE
    }
}

pub trait SumConstants {
    const ZERO: Self;
    const ONE: Self;
    const TWO: Self;
    const THREE: Self;
}

impl<S: PropTraits> SumConstants for Sum<S> {
    const ZERO: Self = Self(S::ZERO);
    const ONE: Self = Self(S::ONE);
    const TWO: Self = Self(S::TWO);
    const THREE: Self = Self(S::THREE);
}

impl<S: PropTraits> Sum<S> {
    pub fn value(&self) -> S {
        self.0
    }
}

impl<S: PropTraits> From<&[Proportion<S>; 3]> for Sum<S> {
    fn from(array: &[Proportion<S>; 3]) -> Self {
        let sum = (array[0].value() + array[1].value() + array[2].value()).min(S::THREE);
        Self(sum)
    }
}

impl<S: PropTraits> From<Proportion<S>> for Sum<S> {
    fn from(proportion: Proportion<S>) -> Self {
        debug_assert!(proportion.is_valid());
        Self(proportion.0)
    }
}

impl<S: PropTraits> Mul<Proportion<S>> for Sum<S> {
    type Output = Self;

    fn mul(self, proportion: Proportion<S>) -> Self::Output {
        debug_assert!(proportion.is_valid());
        let val = Self(self.0 * proportion.0);
        debug_assert!(val.is_valid());
        val
    }
}

impl<S: PropTraits> Div<u8> for Sum<S> {
    type Output = Proportion<S>;

    fn div(self, scalar: u8) -> Self::Output {
        debug_assert!(scalar >= 1 && scalar <= 3);
        let proportion = Proportion(self.0 / S::from(scalar).unwrap());
        debug_assert!(proportion.is_valid());
        proportion
    }
}

impl<S: PropTraits> Sub for Sum<S> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        debug_assert!(rhs.is_valid());
        let val = Self(self.0 - rhs.0);
        debug_assert!(val.is_valid());
        val
    }
}

impl<S: PropTraits> Sub<Proportion<S>> for Sum<S> {
    type Output = Self;

    fn sub(self, proportion: Proportion<S>) -> Self::Output {
        debug_assert!(proportion.is_valid());
        let val = Self(self.0 - proportion.0);
        debug_assert!(val.is_valid());
        val
    }
}

impl<S: PropTraits> Add<Proportion<S>> for Sum<S> {
    type Output = Self;

    fn add(self, proportion: Proportion<S>) -> Self::Output {
        debug_assert!(proportion.is_valid());
        let val = Self(self.0 + proportion.0);
        debug_assert!(val.is_valid());
        val
    }
}

impl<S: PropTraits> Add for Sum<S> {
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
