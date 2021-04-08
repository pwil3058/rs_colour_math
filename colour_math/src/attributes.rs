// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
//use std::cmp::Ordering;

use std::{
    cmp::Ordering,
    fmt::Debug,
    ops::{Div, Mul},
};

use crate::{impl_prop_to_from_float, impl_to_from_number};

use crate::{
    debug::{ApproxEq, PropDiff},
    fdrn::{FDRNumber, IntoProp, Prop, UFDRNumber},
    hue::{Hue, HueBasics},
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
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

    pub fn is_shade(self) -> bool {
        match self {
            Chroma::Shade(_) => true,
            _ => false,
        }
    }

    pub fn is_tint(self) -> bool {
        match self {
            Chroma::Tint(_) => true,
            _ => false,
        }
    }

    pub fn is_neither(self) -> bool {
        match self {
            Chroma::Neither(_) => true,
            _ => false,
        }
    }

    pub fn is_valid(self) -> bool {
        match self {
            Chroma::Neither(_) => true,
            Chroma::Shade(c_prop) | Chroma::Tint(c_prop) => {
                c_prop > Prop::ZERO && c_prop < Prop::ONE
            }
        }
    }

    pub fn is_valid_re(self, hue: Hue, sum: UFDRNumber) -> bool {
        debug_assert!(sum.is_valid_sum());
        match self {
            Chroma::Neither(_) => sum == hue.sum_for_max_chroma(),
            Chroma::Shade(c_prop) => {
                c_prop > Prop::ZERO && c_prop < Prop::ONE && sum < hue.sum_for_max_chroma()
            }
            Chroma::Tint(c_prop) => {
                c_prop > Prop::ZERO && c_prop < Prop::ONE && sum > hue.sum_for_max_chroma()
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

impl From<(Prop, Hue, UFDRNumber)> for Chroma {
    fn from((prop, hue, sum): (Prop, Hue, UFDRNumber)) -> Self {
        match prop {
            Prop::ZERO => Chroma::ZERO,
            Prop::ONE => Chroma::ONE,
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Greyness {
    Shade(Prop),
    Tint(Prop),
    Neither(Prop),
}

impl Greyness {
    pub const ZERO: Self = Self::Neither(Prop::ZERO);
    pub const ONE: Self = Self::Neither(Prop::ONE);

    pub fn is_shade(self) -> bool {
        match self {
            Greyness::Shade(_) => true,
            _ => false,
        }
    }

    pub fn is_tint(self) -> bool {
        match self {
            Greyness::Tint(_) => true,
            _ => false,
        }
    }

    pub fn is_neither(self) -> bool {
        match self {
            Greyness::Neither(_) => true,
            _ => false,
        }
    }

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
    pub fn approx_eq(&self, other: &Self, acceptable_rounding_error: Option<u64>) -> bool {
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

#[derive(
    Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default, Debug,
)]
pub struct Warmth(pub(crate) u64);

impl Warmth {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(u64::MAX);

    const K: Prop = Prop(u64::MAX / 3);
    const K_COMP: Prop = Prop(u64::MAX - Self::K.0);
    const B: UFDRNumber = UFDRNumber(u64::MAX as u128 / 2);

    pub fn calculate(chroma: Chroma, x_dash: Prop) -> Self {
        debug_assert_ne!(chroma, Chroma::ZERO);
        let temp = (Self::K + Self::K_COMP * x_dash) * chroma.into_prop();
        debug_assert!(temp <= UFDRNumber::ONE);
        match chroma {
            Chroma::Shade(prop) => {
                let warmth = Self::B - Self::B * prop + temp;
                debug_assert!(warmth <= UFDRNumber::ONE);
                warmth.into()
            }
            _ => temp.into(),
        }
    }

    pub(crate) fn calculate_monochrome_fm_sum(sum: UFDRNumber) -> Self {
        ((UFDRNumber::THREE - sum) / 6).into()
    }

    pub fn calculate_monochrome(value: Value) -> Self {
        ((Prop::ONE - Prop::from(value)) / 2).into()
    }

    pub fn abs_diff(&self, other: &Self) -> Warmth {
        match self.cmp(other) {
            Ordering::Greater => Warmth(self.0 - other.0),
            Ordering::Less => Warmth(other.0 - self.0),
            Ordering::Equal => Warmth(0),
        }
    }
}

#[cfg(test)]
impl Warmth {
    pub fn approx_eq(&self, other: &Self, acceptable_rounding_error: Option<u64>) -> bool {
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

impl_to_from_number!(UFDRNumber, u128, Warmth);
impl_to_from_number!(FDRNumber, i128, Warmth);

#[derive(
    Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default, Debug,
)]
pub struct Value(pub(crate) u64);

impl Value {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(u64::MAX);

    pub fn abs_diff(&self, other: &Self) -> Value {
        match self.cmp(other) {
            Ordering::Greater => Value(self.0 - other.0),
            Ordering::Less => Value(other.0 - self.0),
            Ordering::Equal => Value(0),
        }
    }
}

#[cfg(test)]
impl Value {
    pub fn approx_eq(&self, other: &Self, acceptable_rounding_error: Option<u64>) -> bool {
        (*self)
            .into_prop()
            .approx_eq(&(*other).into_prop(), acceptable_rounding_error)
    }
}

impl Div<i32> for Value {
    type Output = Value;

    fn div(self, rhs: i32) -> Self {
        debug_assert!(rhs >= 0);
        Self(self.0 / rhs as u64)
    }
}

impl Mul<i32> for Value {
    type Output = UFDRNumber;

    fn mul(self, rhs: i32) -> UFDRNumber {
        debug_assert!(rhs >= 0);
        UFDRNumber(self.0 as u128 * rhs as u128)
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

impl_to_from_number!(UFDRNumber, u128, Value);
impl_to_from_number!(FDRNumber, i128, Value);
