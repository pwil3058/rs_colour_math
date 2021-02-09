// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::{cmp::Ordering, convert::From, ops::Index, ops::Mul};

use crate::{hue::Hue, proportion::*, Float, HueConstants, LightLevel, RGBConstants, CCI};
use std::convert::TryFrom;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Default)]
pub struct RGB<T: LightLevel>(pub(crate) [T; 3]);

impl<T: LightLevel> Eq for RGB<T> where T: Eq {}

impl<T: LightLevel> HueConstants for RGB<T> {
    const RED: Self = Self([T::ONE, T::ZERO, T::ZERO]);
    const GREEN: Self = Self([T::ZERO, T::ONE, T::ZERO]);
    const BLUE: Self = Self([T::ZERO, T::ZERO, T::ONE]);

    const CYAN: Self = Self([T::ZERO, T::ONE, T::ONE]);
    const MAGENTA: Self = Self([T::ONE, T::ZERO, T::ONE]);
    const YELLOW: Self = Self([T::ONE, T::ONE, T::ZERO]);
}

impl<T: LightLevel> RGBConstants for RGB<T> {
    const WHITE: Self = Self([T::ONE, T::ONE, T::ONE]);
    const BLACK: Self = Self([T::ZERO, T::ZERO, T::ZERO]);
}

impl<T: LightLevel + Copy + From<UFDFraction>> RGB<T> {
    pub fn new_grey(value: UFDFraction) -> Self {
        debug_assert!(value.is_vp());
        Self::from([value, value, value])
    }
}

impl<T: LightLevel + Into<UFDFraction>> RGB<T> {
    pub fn sum(&self) -> UFDFraction {
        let [red, green, blue] = <[UFDFraction; 3]>::from(*self);
        red + green + blue
    }

    pub fn chroma_proportion(&self) -> UFDFraction {
        let [red, green, blue] = <[UFDFraction; 3]>::from(*self);
        match red.cmp(&green) {
            Ordering::Greater => match green.cmp(&blue) {
                Ordering::Greater => red - blue,
                Ordering::Less => match red.cmp(&blue) {
                    Ordering::Greater => red - green,
                    Ordering::Less => blue - green,
                    Ordering::Equal => blue - green,
                },
                Ordering::Equal => red - blue,
            },
            Ordering::Less => match red.cmp(&blue) {
                Ordering::Greater => green - blue,
                Ordering::Less => match green.cmp(&blue) {
                    Ordering::Greater => green - red,
                    Ordering::Less => blue - red,
                    Ordering::Equal => blue - red,
                },
                Ordering::Equal => green - blue,
            },
            Ordering::Equal => match red.cmp(&blue) {
                Ordering::Greater => red - blue,
                Ordering::Less => blue - red,
                Ordering::Equal => UFDFraction::ZERO,
            },
        }
    }
}

impl<T: LightLevel> Index<CCI> for RGB<T> {
    type Output = T;

    fn index(&self, index: CCI) -> &T {
        match index {
            CCI::Red => &self.0[0],
            CCI::Green => &self.0[1],
            CCI::Blue => &self.0[2],
        }
    }
}

// Comparisons
impl<T: LightLevel> PartialOrd for RGB<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.0 == other.0 {
            Some(Ordering::Equal)
        } else if let Ok(hue) = Hue::try_from(self) {
            if let Ok(other_hue) = Hue::try_from(other) {
                // This orders via hue from CYAN to CYAN via GREEN, RED, BLUE in that order
                match hue.cmp(&other_hue) {
                    Ordering::Equal => match self.sum().cmp(&other.sum()) {
                        Ordering::Equal => {
                            Some(self.chroma_proportion().cmp(&self.chroma_proportion()))
                        }
                        order => Some(order),
                    },
                    order => Some(order),
                }
            } else {
                Some(Ordering::Greater)
            }
        } else if Hue::try_from(other).is_ok() {
            Some(Ordering::Less)
        } else {
            // No need to look a chroma as it will be zero for both
            self.sum().partial_cmp(&other.sum())
        }
    }
}

impl<T: LightLevel> Ord for RGB<T>
where
    T: PartialOrd + Eq,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other)
            .expect("restricted range of values means this is OK")
    }
}

impl<T: LightLevel + Float> RGB<T> {
    pub fn approx_eq(&self, other: &Self, max_diff: Option<T>) -> bool {
        for i in 0..3 {
            if !self.0[i].approx_eq(&other.0[i], max_diff) {
                return false;
            }
        }
        true
    }
}

impl<T: LightLevel> From<[T; 3]> for RGB<T> {
    fn from(array: [T; 3]) -> Self {
        Self(array)
    }
}

impl<T: LightLevel> From<&[T; 3]> for RGB<T> {
    fn from(array: &[T; 3]) -> Self {
        Self(*array)
    }
}

impl<T: LightLevel + From<UFDFraction>> From<[UFDFraction; 3]> for RGB<T> {
    fn from(array: [UFDFraction; 3]) -> Self {
        let red: T = array[0].into();
        let green: T = array[1].into();
        let blue: T = array[2].into();
        Self([red, green, blue])
    }
}

impl<T: LightLevel + Into<UFDFraction>> From<RGB<T>> for [UFDFraction; 3] {
    fn from(rgb: RGB<T>) -> Self {
        [rgb.0[0].into(), rgb.0[1].into(), rgb.0[2].into()]
    }
}

// Arithmetic
impl<F: Float + LightLevel + From<UFDFraction>> Mul<UFDFraction> for RGB<F> {
    type Output = Self;

    fn mul(self, scalar: UFDFraction) -> Self {
        let [red, green, blue] = <[UFDFraction; 3]>::from(self);
        let array: [UFDFraction; 3] = [red * scalar, green * scalar, blue * scalar];
        Self::from(array)
    }
}
