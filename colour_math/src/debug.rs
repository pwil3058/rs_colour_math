// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::{
    cmp::{PartialEq, PartialOrd},
    ops::Sub,
};

pub trait ApproxEq: Copy + PartialEq + PartialOrd + Sized + Sub<Output = Self> {
    const DEFAULT_MAX_DIFF: Self;

    fn abs_diff(&self, other: &Self) -> Self {
        if self > other {
            *self - *other
        } else {
            *other - *self
        }
    }

    fn approx_eq(&self, other: &Self, max_diff: Option<Self>) -> bool {
        if self.eq(other) {
            true
        } else {
            let max_diff = if let Some(max_diff) = max_diff {
                max_diff
            } else {
                Self::DEFAULT_MAX_DIFF
            };
            self.abs_diff(other) <= max_diff
        }
    }
}

impl ApproxEq for u8 {
    const DEFAULT_MAX_DIFF: Self = 0x02;
}

impl ApproxEq for u16 {
    const DEFAULT_MAX_DIFF: Self = 0x0004;
}

impl ApproxEq for u32 {
    const DEFAULT_MAX_DIFF: Self = 0x000000010;
}

impl ApproxEq for u64 {
    const DEFAULT_MAX_DIFF: Self = 0x0000000000001000;
}

impl ApproxEq for u128 {
    const DEFAULT_MAX_DIFF: Self = 6;
}

impl ApproxEq for f32 {
    const DEFAULT_MAX_DIFF: Self = f32::EPSILON;
}

impl ApproxEq for f64 {
    const DEFAULT_MAX_DIFF: Self = f64::EPSILON;
}
