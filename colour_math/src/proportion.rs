// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
//use std::cmp::Ordering;
use std::fmt::Debug;
use std::ops::{Add, Div, Mul, Sub};

use num_traits_plus::float_plus::{FloatApproxEq, FloatPlus};
use num_traits_plus::num_traits::Num;
use std::path::Component::Prefix;

pub trait Validation {
    fn is_valid(&self) -> bool;
}

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
pub struct Proportion<N: Num>(N);

impl<N: Num> Proportion<N>
where
    N: ProportionConstants + Clone + Copy,
{
    pub const ZERO: Self = Self(N::PROPORTION_MIN);
    pub const ONE: Self = Self(N::PROPORTION_MAX);
}

impl<N: Num> Validation for Proportion<N>
where
    N: ProportionConstants + PartialOrd,
{
    fn is_valid(&self) -> bool {
        self.0 >= N::PROPORTION_MIN || self.0 <= N::PROPORTION_MAX
    }
}

impl<F: FloatPlus> From<Sum<F>> for Proportion<F>
where
    F: ProportionConstants + PartialOrd,
{
    fn from(sum: Sum<F>) -> Self {
        debug_assert!(sum.is_valid());
        let proportion = Self(sum.0);
        debug_assert!(proportion.is_valid());
        proportion
    }
}

impl<N: Num> Proportion<N>
where
    N: Copy,
{
    pub fn value(&self) -> N {
        self.0
    }
}

impl<N: Num> Sub for Proportion<N>
where
    N: ProportionConstants + PartialOrd,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        debug_assert!(rhs.is_valid());
        let val = Self(self.0 - rhs.0);
        debug_assert!(val.is_valid());
        val
    }
}

impl<N: FloatPlus> Add for Proportion<N>
where
    N: ProportionConstants + PartialOrd,
{
    type Output = Sum<N>;

    fn add(self, rhs: Self) -> Self::Output {
        debug_assert!(rhs.is_valid());
        let val = Sum(self.0 + rhs.0);
        debug_assert!(val.is_valid());
        val
    }
}

impl<F: FloatPlus> Mul<u8> for Proportion<F>
where
    F: ProportionConstants + PartialOrd,
{
    type Output = Sum<F>;

    fn mul(self, scalar: u8) -> Self::Output {
        debug_assert!(scalar >= 1 && scalar <= 3);
        let val = Sum(self.0 * F::from(scalar).unwrap());
        debug_assert!(val.is_valid());
        val
    }
}

impl<F: FloatPlus> Mul for Proportion<F>
where
    F: ProportionConstants + PartialOrd,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        debug_assert!(rhs.is_valid());
        let val = Self(self.0 * rhs.0);
        debug_assert!(val.is_valid());
        val
    }
}

impl<F: FloatPlus> Div for Proportion<F>
where
    F: ProportionConstants + PartialOrd,
{
    type Output = F;

    fn div(self, rhs: Self) -> Self::Output {
        debug_assert!(rhs.is_valid());
        self.0 / rhs.0
    }
}

impl<N: Num> From<N> for Proportion<N>
where
    N: ProportionConstants + PartialOrd,
{
    fn from(arg: N) -> Self {
        let proportion = Self(arg);
        debug_assert!(proportion.is_valid());
        proportion
    }
}

impl<F: FloatPlus> FloatApproxEq<F> for Proportion<F>
where
    F: FloatApproxEq<F>,
{
    fn approx_eq(&self, other: &Self, max_diff: Option<F>) -> bool {
        self.0.approx_eq(&other.0, max_diff)
    }
}

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq, Default, PartialOrd, Ord,
)]
pub struct Sum<F: FloatPlus>(F);

impl<F: FloatPlus> Validation for Sum<F> {
    fn is_valid(&self) -> bool {
        self.0 >= F::ZERO && self.0 <= F::ONE
    }
}

pub trait SumConstants {
    const ZERO: Self;
    const ONE: Self;
    const TWO: Self;
    const THREE: Self;
}

impl<F: FloatPlus> SumConstants for Sum<F> {
    const ZERO: Self = Self(F::ZERO);
    const ONE: Self = Self(F::ONE);
    const TWO: Self = Self(F::TWO);
    const THREE: Self = Self(F::THREE);
}

impl<F: FloatPlus> Sum<F>
where
    F: Copy,
{
    pub fn value(&self) -> F {
        self.0
    }
}

impl<F: FloatPlus> From<&[Proportion<F>; 3]> for Sum<F> {
    fn from(array: &[Proportion<F>; 3]) -> Self {
        let sum = (array[0].value() + array[1].value() + array[2].value()).min(F::THREE);
        Self(sum)
    }
}

impl<F: FloatPlus> From<Proportion<F>> for Sum<F>
where
    F: ProportionConstants,
{
    fn from(proportion: Proportion<F>) -> Self {
        debug_assert!(proportion.is_valid());
        Self(proportion.0)
    }
}

impl<F: FloatPlus> Mul<Proportion<F>> for Sum<F>
where
    F: ProportionConstants,
{
    type Output = Self;

    fn mul(self, proportion: Proportion<F>) -> Self::Output {
        debug_assert!(proportion.is_valid());
        let val = Self(self.0 * proportion.0);
        debug_assert!(val.is_valid());
        val
    }
}

impl<F: FloatPlus> Div<u8> for Sum<F>
where
    F: ProportionConstants,
{
    type Output = Proportion<F>;

    fn div(self, scalar: u8) -> Self::Output {
        debug_assert!(scalar >= 1 && scalar <= 3);
        let proportion = Proportion(self.0 / F::from(scalar).unwrap());
        debug_assert!(proportion.is_valid());
        proportion
    }
}

impl<F: FloatPlus> Sub for Sum<F> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        debug_assert!(rhs.is_valid());
        let val = Self(self.0 - rhs.0);
        debug_assert!(val.is_valid());
        val
    }
}

impl<F: FloatPlus> Sub<Proportion<F>> for Sum<F>
where
    F: ProportionConstants,
{
    type Output = Self;

    fn sub(self, proportion: Proportion<F>) -> Self::Output {
        debug_assert!(proportion.is_valid());
        let val = Self(self.0 - proportion.0);
        debug_assert!(val.is_valid());
        val
    }
}

impl<F: FloatPlus> Add<Proportion<F>> for Sum<F>
where
    F: ProportionConstants,
{
    type Output = Self;

    fn add(self, proportion: Proportion<F>) -> Self::Output {
        debug_assert!(proportion.is_valid());
        let val = Self(self.0 + proportion.0);
        debug_assert!(val.is_valid());
        val
    }
}

impl<F: FloatPlus> Add for Sum<F> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        debug_assert!(rhs.is_valid());
        let val = Self(self.0 + rhs.0);
        debug_assert!(val.is_valid());
        val
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Chroma<F: FloatPlus> {
    Shade(Proportion<F>),
    Tint(Proportion<F>),
}

impl<F: FloatPlus + ProportionConstants> Chroma<F> {
    pub const ZERO: Self = Self::Shade(Proportion::ZERO);
    pub const ONE: Self = Self::Tint(Proportion::ONE);
}

impl<F: FloatPlus + ProportionConstants> Validation for Chroma<F> {
    fn is_valid(&self) -> bool {
        match self {
            Chroma::Shade(val) => val.0 >= F::ZERO && val.0 <= F::ONE,
            Chroma::Tint(val) => val.0 >= F::ZERO && val.0 <= F::ONE,
        }
    }
}
