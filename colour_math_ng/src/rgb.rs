// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::{
    cmp::Ordering,
    convert::{From, TryFrom},
    ops::Index,
    ops::Mul,
};

use crate::hue::HueIfceTmp;
use crate::{hue::Hue, Float, HueConstants, LightLevel, Prop, RGBConstants, Sum, CCI};

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

impl<T: LightLevel + Copy + From<Prop>> RGB<T> {
    pub fn new_grey(value: Prop) -> Self {
        Self::from([value, value, value])
    }
}

impl<T: LightLevel + Into<Prop>> RGB<T> {
    pub fn is_grey(&self) -> bool {
        self.0[0] == self.0[1] && self.0[1] == self.0[2]
    }

    pub fn sum(&self) -> Sum {
        let [red, green, blue] = <[Prop; 3]>::from(*self);
        red + green + blue
    }

    pub fn value(&self) -> Prop {
        self.sum() / 3
    }

    pub fn warmth(&self) -> Prop {
        let [red, green, blue] = <[Prop; 3]>::from(*self);
        (Sum::ONE + red - (blue + green) / 2) / 2
    }

    pub fn max_chroma_rgb(&self) -> RGB<T> {
        if let Ok(hue) = Hue::try_from(self) {
            hue.max_chroma_rgb::<T>()
        } else {
            RGB::<T>::new_grey(self.value())
        }
    }

    pub fn chroma_proportion(&self) -> Prop {
        let [red, green, blue] = <[Prop; 3]>::from(*self);
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
                Ordering::Equal => Prop::ZERO,
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
                    Ordering::Equal => {
                        match self.chroma_proportion().cmp(&other.chroma_proportion()) {
                            Ordering::Equal => Some(self.sum().cmp(&self.sum())),
                            order => Some(order),
                        }
                    }
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

impl<T: LightLevel + From<Prop>> From<[Prop; 3]> for RGB<T> {
    fn from(array: [Prop; 3]) -> Self {
        let red: T = array[0].into();
        let green: T = array[1].into();
        let blue: T = array[2].into();
        Self([red, green, blue])
    }
}

impl<T: LightLevel + Into<Prop>> From<RGB<T>> for [Prop; 3] {
    fn from(rgb: RGB<T>) -> Self {
        [rgb.0[0].into(), rgb.0[1].into(), rgb.0[2].into()]
    }
}

// Arithmetic
impl<F: Float + LightLevel + From<Prop>> Mul<Prop> for RGB<F> {
    type Output = Self;

    fn mul(self, scalar: Prop) -> Self {
        let [red, green, blue] = <[Prop; 3]>::from(self);
        let array: [Prop; 3] = [red * scalar, green * scalar, blue * scalar];
        Self::from(array)
    }
}
