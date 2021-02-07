// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::{cmp::Ordering, convert::From, ops::Index, ops::Mul};

use num_traits_plus::float_plus::FloatApproxEq;

use crate::{proportion::*, HueConstants, RGBConstants, CCI};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Default)]
pub struct RGB<T: Number>(pub(crate) [Proportion<T>; 3]);

impl<T: Number> Eq for RGB<T> where T: Eq {}

impl<T: Number> HueConstants for RGB<T>
where
    T: Copy,
{
    const RED: Self = Self([Proportion::P_ONE, Proportion::P_ZERO, Proportion::P_ZERO]);
    const GREEN: Self = Self([Proportion::P_ZERO, Proportion::P_ONE, Proportion::P_ZERO]);
    const BLUE: Self = Self([Proportion::P_ZERO, Proportion::P_ZERO, Proportion::P_ONE]);

    const CYAN: Self = Self([Proportion::P_ZERO, Proportion::P_ONE, Proportion::P_ONE]);
    const MAGENTA: Self = Self([Proportion::P_ONE, Proportion::P_ZERO, Proportion::P_ONE]);
    const YELLOW: Self = Self([Proportion::P_ONE, Proportion::P_ONE, Proportion::P_ZERO]);
}

impl<T: Copy + Number> RGBConstants for RGB<T> {
    const WHITE: Self = Self([Proportion::P_ONE, Proportion::P_ONE, Proportion::P_ONE]);
    const BLACK: Self = Self([Proportion::P_ZERO, Proportion::P_ZERO, Proportion::P_ZERO]);
}

impl<T: Number> Index<CCI> for RGB<T> {
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
impl<T: Number> PartialOrd for RGB<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, _other: &Self) -> Option<Ordering> {
        //         if self.0 == other.0 {
        //             Some(Ordering::Equal)
        //         } else if let Ok(hue) = Hue::<T>::try_from(self) {
        //             if let Ok(other_hue) = Hue::<T>::try_from(other) {
        //                 // This orders via hue from CYAN to CYAN via GREEN, RED, BLUE in that order
        //                 hue.partial_cmp(&other_hue)
        //             } else {
        //                 Some(Ordering::Greater)
        //             }
        //         } else if Hue::<T>::try_from(other).is_ok() {
        //             Some(Ordering::Less)
        //         } else {
        //             // No need to look a chroma as it will be zero for both
        //             //self.sum().partial_cmp(&other.sum())
        //             Some(Ordering::Equal)
        //         }
        //     }
        None
    }
}

impl<T: Number> Ord for RGB<T>
where
    T: PartialOrd + Eq,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other)
            .expect("restricted range of values means this is OK")
    }
}

impl<F: Float> FloatApproxEq<F> for RGB<F> {
    fn approx_eq(&self, other: &RGB<F>, max_diff: Option<F>) -> bool {
        for i in 0..3 {
            if !self.0[i].approx_eq(&other.0[i], max_diff) {
                return false;
            }
        }
        true
    }
}

impl<T: Number> From<[Proportion<T>; 3]> for RGB<T> {
    fn from(array: [Proportion<T>; 3]) -> Self {
        Self(array)
    }
}

impl<T: Number> From<&[Proportion<T>; 3]> for RGB<T> {
    fn from(array: &[Proportion<T>; 3]) -> Self {
        Self(*array)
    }
}

impl<T: Number> From<[T; 3]> for RGB<T> {
    fn from(array: [T; 3]) -> Self {
        let red = Proportion::from(&array[0]);
        let green = Proportion::from(&array[1]);
        let blue = Proportion::from(&array[2]);
        Self([red, green, blue])
    }
}

// Arithmetic
impl<F: Float> Mul<Proportion<F>> for RGB<F> {
    type Output = Self;

    fn mul(self, scalar: Proportion<F>) -> Self {
        let array: [Proportion<F>; 3] =
            [self.0[0] * scalar, self.0[1] * scalar, self.0[2] * scalar];
        array.into()
    }
}
