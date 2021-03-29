// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::{cmp::PartialOrd, ops::Sub};

use crate::fdrn::Prop;

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
    fn prop_diff(&self, other: &Self) -> Prop;
}

impl PropDiff for u8 {
    fn prop_diff(&self, other: &Self) -> Prop {
        Prop((self.abs_diff(other) as u128 * u64::MAX as u128 / *self.max(other) as u128) as u64)
    }
}

impl PropDiff for u16 {
    fn prop_diff(&self, other: &Self) -> Prop {
        Prop((self.abs_diff(other) as u128 * u64::MAX as u128 / *self.max(other) as u128) as u64)
    }
}

impl PropDiff for u32 {
    fn prop_diff(&self, other: &Self) -> Prop {
        Prop((self.abs_diff(other) as u128 * u64::MAX as u128 / *self.max(other) as u128) as u64)
    }
}

impl PropDiff for u64 {
    fn prop_diff(&self, other: &Self) -> Prop {
        Prop((self.abs_diff(other) as u128 * u64::MAX as u128 / *self.max(other) as u128) as u64)
    }
}

impl PropDiff for u128 {
    fn prop_diff(&self, other: &Self) -> Prop {
        Prop((self.abs_diff(other) * u64::MAX as u128 / self.max(other)) as u64)
    }
}

impl PropDiff for f32 {
    fn prop_diff(&self, other: &Self) -> Prop {
        Prop((self.abs_diff(other) * u64::MAX as f32 / self.max(*other)) as u64)
    }
}

impl PropDiff for f64 {
    fn prop_diff(&self, other: &Self) -> Prop {
        Prop((self.abs_diff(other) * u64::MAX as f64 / self.max(*other)) as u64)
    }
}

pub trait ApproxEq: Copy + PropDiff + PartialEq {
    const DEFAULT_MAX_DIFF: Prop = Prop(0x0000000000001000);

    fn approx_eq(&self, other: &Self, max_diff: Option<Prop>) -> bool {
        if self.eq(other) {
            true
        } else {
            let max_diff = if let Some(max_diff) = max_diff {
                max_diff
            } else {
                Self::DEFAULT_MAX_DIFF
            };
            self.prop_diff(other) <= max_diff
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
