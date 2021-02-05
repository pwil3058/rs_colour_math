// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::cmp::Ordering;
use std::fmt::Debug;

use num_traits_plus::{float_plus::FloatPlus, num_traits::Unsigned, NumberConstants};

pub trait ProportionConstants {
    const ZERO: Self;
    const ONE: Self;
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Proportion<T>(T);

impl<F: FloatPlus> ProportionConstants for Proportion<F> {
    const ZERO: Self = Self(F::ZERO);
    const ONE: Self = Self(F::ONE);
}

impl<U: Unsigned + NumberConstants> ProportionConstants for Proportion<U> {
    const ZERO: Self = Self(U::ZERO);
    const ONE: Self = Self(U::MAX);
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Chroma<F: FloatPlus> {
    Shade(Proportion<F>),
    Tint(Proportion<F>),
}
