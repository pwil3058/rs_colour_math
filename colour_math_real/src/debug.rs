// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::{cmp::PartialOrd, ops::Sub};

use crate::real::Prop;

pub trait AbsDiff: Copy + Sized + PartialOrd + Sub<Output = Self> {
    fn abs_diff(&self, other: &Self) -> Self {
        if self > other {
            *self - *other
        } else {
            *other - *self
        }
    }
}

impl AbsDiff for u8 {}
impl AbsDiff for u16 {}
impl AbsDiff for u32 {}
impl AbsDiff for u64 {}
impl AbsDiff for u128 {}

impl AbsDiff for f32 {}
impl AbsDiff for f64 {}

pub trait PropDiff {
    fn prop_diff(&self, _other: &Self) -> Option<Prop> {
        None
    }
}

macro_rules! impl_prop_diff_for_unsigned {
    (u128) => {
        impl PropDiff for u128 {
            fn prop_diff(&self, other: &Self) -> Option<Prop> {
                match self.max(other) {
                    0 => Some(Prop::ZERO),
                    denom => Some(Prop((self.abs_diff(other) as f64 / *denom as f64))),
                }
            }
        }
    };
    ($unsigned:ty) => {
        impl PropDiff for $unsigned {
            fn prop_diff(&self, other: &Self) -> Option<Prop> {
                match *self.max(other) as u128 {
                    0 => Some(Prop::ZERO),
                    denom => Some(Prop((self.abs_diff(other) as f64 / denom as f64))),
                }
            }
        }
    };
}

impl_prop_diff_for_unsigned!(u8);
impl_prop_diff_for_unsigned!(u16);
impl_prop_diff_for_unsigned!(u32);
impl_prop_diff_for_unsigned!(u64);
impl_prop_diff_for_unsigned!(u128);

macro_rules! impl_prop_diff_for_float {
    ($float:ty) => {
        impl PropDiff for $float {
            fn prop_diff(&self, other: &Self) -> Option<Prop> {
                let denom = self.max(*other);
                if denom == 0.0 {
                    Some(Prop::ZERO)
                } else {
                    Some(Prop((self.abs_diff(other) / denom) as f64))
                }
            }
        }
    };
}

impl_prop_diff_for_float!(f32);
impl_prop_diff_for_float!(f64);

pub trait ApproxEq: Copy + PropDiff + PartialEq {
    const DEFAULT_MAX_DIFF: Prop = Prop(f64::EPSILON);

    fn approx_eq(&self, other: &Self, max_diff: Option<Prop>) -> bool {
        if self.eq(other) {
            true
        } else {
            let max_diff = if let Some(max_diff) = max_diff {
                max_diff
            } else {
                Self::DEFAULT_MAX_DIFF
            };
            match self.prop_diff(other) {
                None => false,
                Some(prop_diff) => prop_diff <= max_diff,
            }
        }
    }
}

impl ApproxEq for u8 {}
impl ApproxEq for u16 {}
impl ApproxEq for u32 {}
impl ApproxEq for u64 {}
impl ApproxEq for u128 {}

impl ApproxEq for f32 {}
impl ApproxEq for f64 {}
