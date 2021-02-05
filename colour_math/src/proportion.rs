// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
//use std::cmp::Ordering;
use std::fmt::Debug;

use num_traits_plus::float_plus::FloatPlus;
use num_traits_plus::num_traits::Num;

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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq, Default)]
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

impl<N: Num> Proportion<N> {
    pub fn value(&self) -> &N {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq, Default)]
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

impl<F: FloatPlus> From<&[Proportion<F>; 3]> for Sum<F> {
    fn from(array: &[Proportion<F>; 3]) -> Self {
        let sum = (*array[0].value() + *array[1].value() + *array[2].value()).min(F::THREE);
        Self(sum)
    }
}

impl<F: FloatPlus> SumConstants for Sum<F> {
    const ZERO: Self = Self(F::ZERO);
    const ONE: Self = Self(F::ONE);
    const TWO: Self = Self(F::TWO);
    const THREE: Self = Self(F::THREE);
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Chroma<F: FloatPlus> {
    Shade(Proportion<F>),
    Tint(Proportion<F>),
}

impl<F: FloatPlus> Validation for Chroma<F> {
    fn is_valid(&self) -> bool {
        match self {
            Chroma::Shade(val) => val.0 >= F::ZERO && val.0 <= F::ONE,
            Chroma::Tint(val) => val.0 >= F::ZERO && val.0 <= F::ONE,
        }
    }
}
