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

pub trait Validation {
    fn is_valid(&self) -> bool;
}

pub trait ProportionConstants {
    const P_ZERO: Self;
    const P_ONE: Self;
}

impl ProportionConstants for f32 {
    const P_ZERO: Self = Self::ZERO;
    const P_ONE: Self = Self::ONE;
}

impl ProportionConstants for f64 {
    const P_ZERO: Self = Self::ZERO;
    const P_ONE: Self = Self::ONE;
}

impl ProportionConstants for u8 {
    const P_ZERO: Self = Self::ZERO;
    const P_ONE: Self = Self::MAX;
}

impl ProportionConstants for u16 {
    const P_ZERO: Self = Self::ZERO;
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Default, PartialOrd)]
pub struct Proportion<N: Number>(pub(crate) N);

impl<N: Number> ProportionConstants for Proportion<N> {
    const P_ZERO: Self = Self(N::P_ZERO);
    const P_ONE: Self = Self(N::P_ONE);
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

impl<F: Float> Sub for Proportion<F> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let val = Self(self.0 - rhs.0);
        val
    }
}

impl<F: Float> Add for Proportion<F> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl<F: Float> Mul for Proportion<F> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let val = Self(self.0 * rhs.0);
        val
    }
}

impl<F: Float> Div for Proportion<F> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let val = Self(self.0 / rhs.0);
        val
    }
}

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

impl<F: Float> Mul for Sum<F> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl<F: Float> Div for Sum<F> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl<F: Float> Sub for Sum<F> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl<F: Float> Add for Sum<F> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Chroma<F: Float> {
    Shade(Proportion<F>),
    Tint(Proportion<F>),
}

impl<F: Float + ProportionConstants> Chroma<F> {
    pub const ZERO: Self = Self::Shade(Proportion::P_ZERO);
    pub const ONE: Self = Self::Tint(Proportion::P_ONE);

    pub fn is_zero(&self) -> bool {
        match self {
            Chroma::Shade(proportion) => *proportion == Proportion::<F>::P_ZERO,
            Chroma::Tint(proportion) => *proportion == Proportion::<F>::P_ZERO,
        }
    }

    pub fn proportion(&self) -> Proportion<F> {
        match self {
            Chroma::Shade(proportion) => *proportion,
            Chroma::Tint(proportion) => *proportion,
        }
    }
}

impl<F: Float + ProportionConstants> Validation for Chroma<F> {
    fn is_valid(&self) -> bool {
        match self {
            Chroma::Shade(proportion) => proportion.is_valid(),
            Chroma::Tint(proportion) => proportion.is_valid(),
        }
    }
}

impl<F: Float> FloatApproxEq<F> for Chroma<F> {
    fn approx_eq(&self, other: &Self, max_diff: Option<F>) -> bool {
        match self {
            Chroma::Shade(proportion) => match other {
                Chroma::Shade(other_proportion) => proportion.approx_eq(other_proportion, max_diff),
                Chroma::Tint(other_proportion) => false,
            },
            Chroma::Tint(proportion) => match other {
                Chroma::Shade(other_proportion) => false,
                Chroma::Tint(other_proportion) => proportion.approx_eq(other_proportion, max_diff),
            },
        }
    }
}
