// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::{cmp::Ordering, convert::TryFrom, iter::Sum, ops::Index, ops::Mul};

use crate::{hue_ng::*, ColourComponent, HueConstants, RGBConstants};
use num_traits_plus::float_plus::FloatApproxEq;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Default)]
pub struct RGB<F: ColourComponent>(pub(crate) [F; 3]);

impl<F: ColourComponent> Eq for RGB<F> {}

impl<F: ColourComponent> HueConstants for RGB<F> {
    const RED: Self = Self([F::ONE, F::ZERO, F::ZERO]);
    const GREEN: Self = Self([F::ZERO, F::ONE, F::ZERO]);
    const BLUE: Self = Self([F::ZERO, F::ZERO, F::ONE]);

    const CYAN: Self = Self([F::ZERO, F::ONE, F::ONE]);
    const MAGENTA: Self = Self([F::ONE, F::ZERO, F::ONE]);
    const YELLOW: Self = Self([F::ONE, F::ONE, F::ZERO]);
}

impl<F: ColourComponent> RGBConstants for RGB<F> {
    const WHITE: Self = Self([F::ONE, F::ONE, F::ONE]);
    const BLACK: Self = Self([F::ZERO, F::ZERO, F::ZERO]);
}

impl<F: ColourComponent + Sum<F>> RGB<F> {
    #[cfg(test)]
    pub(crate) fn value(&self) -> F {
        self.sum() / F::THREE
    }

    pub(crate) fn sum(&self) -> F {
        self.0.iter().copied().sum()
    }

    #[cfg(test)]
    pub(crate) fn chroma(&self) -> F {
        F::ONE
    }

    #[cfg(test)]
    pub(crate) fn max_chroma_rgb(&self) -> Self {
        Self::BLACK
    }
}
impl<F: ColourComponent> Index<CCI> for RGB<F> {
    type Output = F;

    fn index(&self, index: CCI) -> &F {
        match index {
            CCI::Red => &self.0[0],
            CCI::Green => &self.0[1],
            CCI::Blue => &self.0[2],
        }
    }
}

// Comparisons
impl<F: ColourComponent> PartialOrd for RGB<F> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.0 == other.0 {
            Some(Ordering::Equal)
        } else if let Ok(hue) = Hue::<F>::try_from(self) {
            if let Ok(other_hue) = Hue::<F>::try_from(other) {
                // This orders via hue from CYAN to CYAN via GREEN, RED, BLUE in that order
                hue.partial_cmp(&other_hue)
            } else {
                Some(Ordering::Greater)
            }
        } else if Hue::<F>::try_from(other).is_ok() {
            Some(Ordering::Less)
        } else {
            // No need to look a chroma as it will be zero for both
            self.sum().partial_cmp(&other.sum())
        }
    }
}

impl<F: ColourComponent> Ord for RGB<F> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other)
            .expect("restricted range of values means this is OK")
    }
}

impl<F: ColourComponent + std::fmt::Debug> FloatApproxEq<F> for RGB<F> {
    fn approx_eq(&self, other: &Self, max_diff: Option<F>) -> bool {
        for i in 0..3 {
            if !self.0[i].approx_eq(&other.0[i], max_diff) {
                return false;
            }
        }
        true
    }
}

impl<F: ColourComponent> From<[F; 3]> for RGB<F> {
    fn from(array: [F; 3]) -> Self {
        debug_assert!(array.iter().all(|x| (*x).is_proportion()), "{:?}", array);
        Self(array)
    }
}

impl<F: ColourComponent> From<&[F]> for RGB<F> {
    fn from(array: &[F]) -> Self {
        debug_assert!(array.len() == 3);
        debug_assert!(array.iter().all(|x| (*x).is_proportion()), "{:?}", array);
        Self([array[0], array[1], array[2]])
    }
}

// Arithmetic
impl<F: ColourComponent> Mul<F> for RGB<F> {
    type Output = Self;

    fn mul(self, scalar: F) -> Self {
        let array: [F; 3] = [self.0[0] * scalar, self.0[1] * scalar, self.0[2] * scalar];
        array.into()
    }
}
