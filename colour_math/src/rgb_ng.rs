// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::{
    cmp::Ordering,
    convert::{From, TryFrom},
    iter::Sum,
    ops::Index,
    ops::Mul,
};

use crate::{hue_ng::*, proportion::*, ColourComponent, HueConstants, RGBConstants};
use num_traits_plus::float_plus::{FloatApproxEq, FloatPlus};
use num_traits_plus::num_traits::Num;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Default)]
pub struct RGB<T: Num>(pub(crate) [Proportion<T>; 3]);

impl<T: Num> Eq for RGB<T> where T: Eq {}

impl<T: Num> HueConstants for RGB<T>
where
    T: Copy,
{
    const RED: Self = Self([Proportion::ONE, Proportion::ZERO, Proportion::ZERO]);
    const GREEN: Self = Self([Proportion::ZERO, Proportion::ONE, Proportion::ZERO]);
    const BLUE: Self = Self([Proportion::ZERO, Proportion::ZERO, Proportion::ONE]);

    const CYAN: Self = Self([Proportion::ZERO, Proportion::ONE, Proportion::ONE]);
    const MAGENTA: Self = Self([Proportion::ONE, Proportion::ZERO, Proportion::ONE]);
    const YELLOW: Self = Self([Proportion::ONE, Proportion::ONE, Proportion::ZERO]);
}

impl<T: Copy + Num> RGBConstants for RGB<T> {
    const WHITE: Self = Self([Proportion::ONE, Proportion::ONE, Proportion::ONE]);
    const BLACK: Self = Self([Proportion::ZERO, Proportion::ZERO, Proportion::ZERO]);
}

impl<T: Num> RGB<T> {
    // #[cfg(test)]
    // pub(crate) fn value(&self) -> Proportion<T> {
    //     (self.sum() / T::THREE).into()
    // }
    //
    // pub(crate) fn sum<T: ColourComponent>(&self) -> Sum {
    //     self.0.iter().copied().sum()
    // }

    #[cfg(test)]
    pub(crate) fn chroma(&self) -> Proportion<T> {
        Proportion::ONE
    }

    #[cfg(test)]
    pub(crate) fn max_chroma_rgb(&self) -> Self {
        Self::BLACK
    }
}

impl<T: Num> Index<CCI> for RGB<T> {
    type Output = Proportion<T>;

    fn index(&self, index: CCI) -> &Proportion<T> {
        match index {
            CCI::Red => &self.0[0],
            CCI::Green => &self.0[1],
            CCI::Blue => &self.0[2],
        }
    }
}

// Comparisons
impl<T: Num> PartialOrd for RGB<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.0 == other.0 {
            Some(Ordering::Equal)
        } else if let Ok(hue) = Hue::<T, Proportion<T>>::try_from(self) {
            if let Ok(other_hue) = Hue::<T, Proportion<T>>::try_from(other) {
                // This orders via hue from CYAN to CYAN via GREEN, RED, BLUE in that order
                hue.partial_cmp(&other_hue)
            } else {
                Some(Ordering::Greater)
            }
        } else if Hue::<T, Proportion<T>>::try_from(other).is_ok() {
            Some(Ordering::Less)
        } else {
            // No need to look a chroma as it will be zero for both
            self.sum().partial_cmp(&other.sum())
        }
    }
}

impl<T: Num> Ord for RGB<T>
where
    T: PartialOrd + Eq,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other)
            .expect("restricted range of values means this is OK")
    }
}

impl<T: Num> FloatApproxEq<T> for RGB<T>
where
    T: Copy + FloatPlus + FloatApproxEq<T>,
{
    fn approx_eq(&self, other: &Self, max_diff: Option<T>) -> bool {
        for i in 0..3 {
            if !self.0[i].approx_eq(&other.0[i], max_diff) {
                return false;
            }
        }
        true
    }
}

impl<T: Num> From<[Proportion<T>; 3]> for RGB<T> {
    fn from(array: [Proportion<T>; 3]) -> Self {
        Self(array)
    }
}

impl<T: Num> From<&[Proportion<T>]> for RGB<T> {
    fn from(array: &[Proportion<T>]) -> Self {
        Self(*array)
    }
}

// Arithmetic
impl<T: Num> Mul<Proportion<T>> for RGB<T> {
    type Output = Self;

    fn mul(self, scalar: Proportion<T>) -> Self {
        let array: [Proportion<T>; 3] =
            [self.0[0] * scalar, self.0[1] * scalar, self.0[2] * scalar];
        array.into()
    }
}
