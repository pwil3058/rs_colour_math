// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
//use std::cmp::Ordering;

use std::{
    cmp::Eq,
    cmp::Ordering,
    fmt::Debug,
    ops::{Div, Mul, Sub},
};

use crate::{impl_prop_to_from_float, impl_to_from_number, impl_wrapped_op};

use crate::{
    debug::{AbsDiff, ApproxEq, PropDiff},
    hue::{Hue, HueBasics},
    real::{IntoProp, Prop, Real},
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chroma {
    Shade(Prop),
    Tint(Prop),
    Neither(Prop),
}

impl Chroma {
    pub const ZERO: Self = Self::Neither(Prop::ZERO);
    pub const ONE: Self = Self::Neither(Prop::ONE);

    pub fn is_zero(self) -> bool {
        self.into_prop() == Prop::ZERO
    }

    pub fn is_valid(self) -> bool {
        match self {
            Chroma::Neither(c_prop) => c_prop.is_valid(),
            Chroma::Shade(c_prop) | Chroma::Tint(c_prop) => {
                c_prop > Prop::ZERO && c_prop < Prop::ONE
            }
        }
    }

    pub fn abs_diff(&self, other: &Self) -> Prop {
        self.into_prop().abs_diff(&other.into_prop())
    }
}

impl From<Chroma> for Prop {
    fn from(chroma: Chroma) -> Prop {
        use Chroma::*;
        match chroma {
            Shade(proportion) | Tint(proportion) | Neither(proportion) => proportion,
        }
    }
}

impl IntoProp for Chroma {}

impl Default for Chroma {
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<(Prop, Hue, Real)> for Chroma {
    fn from((prop, hue, sum): (Prop, Hue, Real)) -> Self {
        match prop {
            prop if prop == Prop::ZERO => Chroma::ZERO,
            prop if prop == Prop::ONE => Chroma::ONE,
            prop => match sum.cmp(&hue.sum_for_max_chroma()) {
                Ordering::Greater => Self::Tint(prop),
                Ordering::Less => Self::Shade(prop),
                Ordering::Equal => Self::Neither(prop),
            },
        }
    }
}

impl PartialOrd for Chroma {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        use Chroma::*;
        match self {
            Shade(proportion) => match rhs {
                Shade(other_proportion) => proportion.partial_cmp(&other_proportion),
                _ => Some(Ordering::Less),
            },
            Tint(proportion) => match rhs {
                Tint(other_proportion) => proportion.partial_cmp(&other_proportion),
                Shade(_) => Some(Ordering::Greater),
                Neither(_) => Some(Ordering::Less),
            },
            Neither(proportion) => match rhs {
                Neither(other_proportion) => proportion.partial_cmp(&other_proportion),
                _ => Some(Ordering::Greater),
            },
        }
    }
}

impl Ord for Chroma {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.partial_cmp(rhs).unwrap()
    }
}

impl PropDiff for Chroma {
    fn prop_diff(&self, other: &Self) -> Option<Prop> {
        use Chroma::*;
        match self {
            Shade(my_prop) => match other {
                Shade(other_prop) => my_prop.prop_diff(other_prop),
                _ => None,
            },
            Tint(my_prop) => match other {
                Tint(other_prop) => my_prop.prop_diff(other_prop),
                _ => None,
            },
            Neither(my_prop) => match other {
                Neither(other_prop) => my_prop.prop_diff(other_prop),
                _ => None,
            },
        }
    }
}

impl ApproxEq for Chroma {}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Greyness {
    Shade(Prop),
    Tint(Prop),
    Neither(Prop),
}

impl Greyness {
    pub const ZERO: Self = Self::Neither(Prop::ZERO);
    pub const ONE: Self = Self::Neither(Prop::ONE);

    pub fn is_zero(&self) -> bool {
        self.into_prop() == Prop::ZERO
    }

    pub fn abs_diff(&self, other: &Self) -> Prop {
        self.into_prop().abs_diff(&other.into_prop())
    }
}

impl From<Greyness> for Prop {
    fn from(greyness: Greyness) -> Prop {
        use Greyness::*;
        match greyness {
            Shade(proportion) | Tint(proportion) | Neither(proportion) => proportion,
        }
    }
}

impl IntoProp for Greyness {}

impl Default for Greyness {
    fn default() -> Self {
        Self::ZERO
    }
}

impl PartialOrd for Greyness {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        use Greyness::*;
        match self {
            Shade(proportion) => match rhs {
                Shade(other_proportion) => proportion.partial_cmp(&other_proportion),
                _ => Some(Ordering::Less),
            },
            Tint(proportion) => match rhs {
                Tint(other_proportion) => proportion.partial_cmp(&other_proportion),
                Shade(_) => Some(Ordering::Greater),
                Neither(_) => Some(Ordering::Less),
            },
            Neither(proportion) => match rhs {
                Neither(other_proportion) => proportion.partial_cmp(&other_proportion),
                _ => Some(Ordering::Greater),
            },
        }
    }
}

impl Ord for Greyness {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.partial_cmp(rhs).unwrap()
    }
}

#[cfg(test)]
impl Greyness {
    fn approx_eq(&self, other: &Self, acceptable_rounding_error: Option<Prop>) -> bool {
        use Greyness::*;
        match self {
            Shade(proportion) => match other {
                Shade(other_proportion) | Neither(other_proportion) => {
                    proportion.approx_eq(other_proportion, acceptable_rounding_error)
                }
                Tint(_) => false,
            },
            Tint(proportion) => match other {
                Shade(_) => false,
                Tint(other_proportion) | Neither(other_proportion) => {
                    proportion.approx_eq(other_proportion, acceptable_rounding_error)
                }
            },
            Neither(proportion) => {
                proportion.approx_eq(&other.into_prop(), acceptable_rounding_error)
            }
        }
    }
}

impl From<Chroma> for Greyness {
    fn from(chroma: Chroma) -> Self {
        match chroma {
            Chroma::Shade(prop) => Greyness::Shade(Prop::ONE - prop),
            Chroma::Tint(prop) => Greyness::Tint(Prop::ONE - prop),
            Chroma::Neither(prop) => Greyness::Neither(Prop::ONE - prop),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, PartialOrd, Default, Debug)]
pub struct Warmth(pub(crate) f64);

impl Eq for Warmth {}

impl Ord for Warmth {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.partial_cmp(rhs).unwrap()
    }
}

impl Warmth {
    pub const ZERO: Self = Self(0.0);
    pub const ONE: Self = Self(1.0);

    const K: Prop = Prop(1.0 / 3.0);
    const K_COMP: Prop = Prop(1.0 - Self::K.0);
    const B: Real = Real(1.0 / 2.0);

    pub fn is_valid(self) -> bool {
        self.0 >= 0.0 && self.0 <= 1.0
    }

    pub fn calculate(chroma: Chroma, x_dash: Prop) -> Self {
        debug_assert_ne!(chroma, Chroma::ZERO);
        let temp = (Self::K + Self::K_COMP * x_dash) * chroma.into_prop();
        debug_assert!(temp <= Real::ONE);
        match chroma {
            Chroma::Shade(prop) => {
                let warmth = Self::B - Self::B * prop + temp;
                debug_assert!(warmth <= Real::ONE);
                warmth.into()
            }
            _ => temp.into(),
        }
    }

    pub(crate) fn calculate_monochrome_fm_sum(sum: Real) -> Self {
        ((Real::THREE - sum) / Real(6.0)).into()
    }

    pub fn calculate_monochrome(value: Value) -> Self {
        ((Prop::ONE - Prop::from(value)) / Real(2.0)).into()
    }
}

impl_wrapped_op!(Sub, sub, Warmth);

impl AbsDiff for Warmth {}

#[cfg(test)]
impl Warmth {
    pub fn approx_eq(&self, other: &Self, acceptable_rounding_error: Option<Prop>) -> bool {
        (*self)
            .into_prop()
            .approx_eq(&(*other).into_prop(), acceptable_rounding_error)
    }
}

impl_prop_to_from_float!(f32, Warmth);
impl_prop_to_from_float!(f64, Warmth);

impl From<Prop> for Warmth {
    fn from(prop: Prop) -> Self {
        Self(prop.0)
    }
}

impl From<Warmth> for Prop {
    fn from(warmth: Warmth) -> Self {
        Self(warmth.0)
    }
}

impl IntoProp for Warmth {}

impl_to_from_number!(Real, Warmth);

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, PartialOrd, Default, Debug)]
pub struct Value(pub(crate) f64);

impl Eq for Value {}

impl Ord for Value {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.partial_cmp(rhs).unwrap()
    }
}

impl Value {
    pub const ZERO: Self = Self(0.0);
    pub const ONE: Self = Self(1.0);

    pub fn is_valid(self) -> bool {
        self.0 >= 0.0 && self.0 <= 1.0
    }
}

impl_wrapped_op!(Sub, sub, Value);

impl AbsDiff for Value {}

#[cfg(test)]
impl Value {
    pub fn approx_eq(&self, other: &Self, acceptable_rounding_error: Option<Prop>) -> bool {
        (*self)
            .into_prop()
            .approx_eq(&(*other).into_prop(), acceptable_rounding_error)
    }
}

impl Div<i32> for Value {
    type Output = Value;

    fn div(self, rhs: i32) -> Self {
        debug_assert!(rhs >= 0);
        Self(self.0 / rhs as f64)
    }
}

impl Mul<i32> for Value {
    type Output = Real;

    fn mul(self, rhs: i32) -> Real {
        debug_assert!(rhs >= 0);
        Real(self.0 * rhs as f64)
    }
}

impl_prop_to_from_float!(f32, Value);
impl_prop_to_from_float!(f64, Value);

impl From<Prop> for Value {
    fn from(prop: Prop) -> Self {
        Self(prop.0)
    }
}

impl From<Value> for Prop {
    fn from(value: Value) -> Self {
        Self(value.0)
    }
}

impl IntoProp for Value {}

impl_to_from_number!(Real, Value);
