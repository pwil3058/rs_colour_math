// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    cmp::Ordering,
    ops::{Add, Div, Mul, Rem, Sub},
};

use crate::debug::{AbsDiff, ApproxEq, PropDiff};

#[macro_export]
macro_rules! impl_wrapped_op {
    ( $op:ident, $op_fn:ident, $wrapper:ident ) => {
        impl $op for $wrapper {
            type Output = Self;

            fn $op_fn(self, rhs: Self) -> Self::Output {
                Self(self.0.$op_fn(rhs.0))
            }
        }
    };
    ( $op:ident<$rhs:ident>, $op_fn:ident, $wrapper:ident ) => {
        impl $op<$rhs> for $wrapper {
            type Output = Self;

            fn $op_fn(self, rhs: $rhs) -> Self::Output {
                Self(self.0.$op_fn(rhs.0))
            }
        }
    };
    ( $op:ident, $op_fn:ident, $wrapper:ident, $output:ident ) => {
        impl $op for $wrapper {
            type Output = $output;

            fn $op_fn(self, rhs: Self) -> Self::Output {
                $output(self.0.$op_fn(rhs.0))
            }
        }
    };
    ( $op:ident<$rhs:ident>, $op_fn:ident, $wrapper:ident, $output:ident ) => {
        impl $op<$rhs> for $wrapper {
            type Output = $output;

            fn $op_fn(self, rhs: $rhs) -> Self::Output {
                $output(self.0.$op_fn(rhs.0))
            }
        }
    };
    ( $op:ident, $op_fn:ident, $wrapper:ident, $doc:meta ) => {
        impl $op for $wrapper {
            type Output = Self;

            #[$doc]
            fn $op_fn(self, rhs: Self) -> Self::Output {
                Self(self.0.$op_fn(rhs.0))
            }
        }
    };
}

#[macro_export]
macro_rules! impl_prop_to_from_float {
    ($float:ty, $number:ty) => {
        impl From<$float> for $number {
            fn from(arg: $float) -> Self {
                debug_assert!(0.0 <= arg && arg <= 1.0);
                // NB: watch out for floating point not being proper reals
                Self(arg as f64)
            }
        }

        impl From<$number> for $float {
            fn from(arg: $number) -> Self {
                arg.0 as $float
            }
        }
    };
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, PartialOrd, Default, Debug)]
pub struct Real(pub(crate) f64);

impl Eq for Real {}

impl Ord for Real {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.partial_cmp(rhs).unwrap()
    }
}

impl Real {
    pub const ZERO: Self = Self(0.0);
    pub const ONE: Self = Self(1.0);
    pub const TWO: Self = Self(2.0);
    pub const THREE: Self = Self(3.0);

    pub fn is_valid_sum(self) -> bool {
        self.0 >= 0.0 && self.0 <= 3.0
    }

    pub fn is_hue_valid(self) -> bool {
        self.0 > 0.0 && self.0 < 3.0
    }

    pub fn is_proportion(self) -> bool {
        self.0 >= 0.0 && self.0 <= 1.0
    }
}

impl AbsDiff for Real {}

impl PropDiff for Real {
    fn prop_diff(&self, other: &Self) -> Option<Prop> {
        self.0.prop_diff(&other.0)
    }
}

#[cfg(test)]
impl ApproxEq for Real {}

impl_prop_to_from_float!(f32, Real);
impl_prop_to_from_float!(f64, Real);

impl_wrapped_op!(Add, add, Real);
impl_wrapped_op!(Add<Prop>, add, Real);
impl_wrapped_op!(Div, div, Real);
impl_wrapped_op!(Mul, mul, Real);
impl_wrapped_op!(Mul<Prop>, mul, Real);
impl_wrapped_op!(Rem, rem, Real);
impl_wrapped_op!(Sub, sub, Real);
impl_wrapped_op!(Sub<Prop>, sub, Real);

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, PartialOrd, Default, Debug)]
pub struct Prop(pub(crate) f64);

impl Eq for Prop {}

impl Ord for Prop {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.partial_cmp(rhs).unwrap()
    }
}

impl Prop {
    pub const ZERO: Self = Self(0.0);
    pub const ONE: Self = Self(1.0);
    pub(crate) const HALF: Self = Self(0.5);

    pub(crate) const ALMOST_ONE: Self = Self(1.0 - f64::EPSILON);

    pub fn is_valid(self) -> bool {
        self.0 >= 0.0 && self.0 <= 1.0
    }
}

impl AbsDiff for Prop {}

impl PropDiff for Prop {
    fn prop_diff(&self, other: &Self) -> Option<Prop> {
        self.0.prop_diff(&other.0)
    }
}

#[cfg(test)]
impl ApproxEq for Prop {}

pub trait IntoProp: Sized + Copy + Into<Prop> {
    fn into_prop(self) -> Prop {
        self.into()
    }
}

impl IntoProp for f32 {}

impl IntoProp for f64 {}

impl_prop_to_from_float!(f32, Prop);
impl_prop_to_from_float!(f64, Prop);

#[macro_export]
macro_rules! impl_to_from_number {
    ($number:ty, $proportion:ty) => {
        impl From<$number> for $proportion {
            #[allow(unused_comparisons)]
            fn from(arg: $number) -> Self {
                debug_assert!(arg.0 >= 0.0 && arg.0 <= 1.0);
                Self(arg.0)
            }
        }

        impl From<$proportion> for $number {
            fn from(arg: $proportion) -> Self {
                assert!(arg.is_valid());
                Self(arg.0)
            }
        }
    };
}

impl_to_from_number!(Real, Prop);

impl IntoProp for Real {}

#[macro_export]
macro_rules! impl_unsigned_to_from_prop {
    (u64) => {
        impl From<u64> for Prop {
            fn from(arg: u64) -> Self {
                Self(arg as f64 / u64::MAX as f64)
            }
        }

        impl From<Prop> for u64 {
            fn from(arg: Prop) -> Self {
                (arg.0 * u64::MAX as f64) as u64
            }
        }

        impl IntoProp for u64 {}
    };
    ($unsigned:ty) => {
        impl From<$unsigned> for Prop {
            fn from(arg: $unsigned) -> Self {
                let val = arg as f64 / <$unsigned>::MAX as f64;
                Self(val)
            }
        }

        impl From<Prop> for $unsigned {
            fn from(arg: Prop) -> Self {
                let val = arg.0 * <$unsigned>::MAX as f64;
                val as $unsigned
            }
        }

        impl IntoProp for $unsigned {}
    };
}

impl_unsigned_to_from_prop!(u8);
impl_unsigned_to_from_prop!(u16);
impl_unsigned_to_from_prop!(u32);
impl_unsigned_to_from_prop!(u64);

impl_wrapped_op!(Add, add, Prop, Real);
impl_wrapped_op!(Div, div, Prop);
impl_wrapped_op!(Div<Real>, div, Prop);
impl_wrapped_op!(Mul, mul, Prop);
impl_wrapped_op!(Mul<Real>, mul, Prop, Real);
impl_wrapped_op!(Sub, sub, Prop);
