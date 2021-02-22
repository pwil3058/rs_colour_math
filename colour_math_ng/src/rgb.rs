// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::{
    cmp::Ordering,
    convert::TryInto,
    convert::{From, TryFrom},
    ops::Index,
    ops::Mul,
};

use crate::{
    fdrn::UFDRNumber,
    hue::{CMYHue, Hue, HueIfce, RGBHue, Sextant},
    proportion::Warmth,
    Chroma, ColourBasics, Float, HueConstants, LightLevel, Prop, RGBConstants, CCI, HCV,
};

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

    const BLUE_CYAN: Self = Self([T::ZERO, T::HALF, T::ONE]);
    const BLUE_MAGENTA: Self = Self([T::HALF, T::ZERO, T::ONE]);
    const RED_MAGENTA: Self = Self([T::ONE, T::ZERO, T::HALF]);
    const RED_YELLOW: Self = Self([T::ONE, T::HALF, T::ZERO]);
    const GREEN_YELLOW: Self = Self([T::HALF, T::ONE, T::ZERO]);
    const GREEN_CYAN: Self = Self([T::ZERO, T::ONE, T::HALF]);
}

impl<T: LightLevel> RGBConstants for RGB<T> {
    const WHITE: Self = Self([T::ONE, T::ONE, T::ONE]);
    const BLACK: Self = Self([T::ZERO, T::ZERO, T::ZERO]);
}

impl<T: LightLevel + Copy + From<Prop>> RGB<T> {
    pub fn new_grey(value: Prop) -> Self {
        Self::from([value, value, value])
    }

    pub fn new_warmth_rgb(warmth: Warmth) -> Self {
        // TODO: fix generation of warmth RGB
        Self::from([warmth.into(), warmth.into(), warmth.into()])
    }
}

impl<T: LightLevel + Into<Prop>> RGB<T> {
    pub fn sum(&self) -> UFDRNumber {
        let [red, green, blue] = <[Prop; 3]>::from(*self);
        red + green + blue
    }

    pub fn max_chroma_rgb(&self) -> RGB<T> {
        if let Ok(hue) = Hue::try_from(self) {
            hue.max_chroma_rgb::<T>()
        } else {
            RGB::<T>::new_grey(self.value())
        }
    }
}

impl<T: LightLevel + Into<Prop>> ColourBasics for RGB<T> {
    fn hue(&self) -> Option<Hue> {
        match self.try_into() {
            Ok(rgb) => Some(rgb),
            Err(_) => None,
        }
    }

    fn is_grey(&self) -> bool {
        self.0[0] == self.0[1] && self.0[1] == self.0[2]
    }

    fn chroma(&self) -> Chroma {
        if let Ok(hue) = Hue::try_from(self) {
            let [red, green, blue] = <[Prop; 3]>::from(*self);
            let sum = self.sum();
            match hue {
                Hue::Primary(RGBHue::Red) => Chroma::from((red - blue, hue, sum)),
                Hue::Primary(RGBHue::Green) => Chroma::from((green - red, hue, sum)),
                Hue::Primary(RGBHue::Blue) => Chroma::from((blue - green, hue, sum)),
                Hue::Secondary(CMYHue::Cyan) => Chroma::from((blue - red, hue, sum)),
                Hue::Secondary(CMYHue::Magenta) => Chroma::from((red - green, hue, sum)),
                Hue::Secondary(CMYHue::Yellow) => Chroma::from((green - blue, hue, sum)),
                Hue::Sextant(sextant_hue) => match sextant_hue.sextant() {
                    Sextant::RedYellow => Chroma::from((red - blue, hue, sum)),
                    Sextant::RedMagenta => Chroma::from((red - green, hue, sum)),
                    Sextant::GreenYellow => Chroma::from((green - blue, hue, sum)),
                    Sextant::GreenCyan => Chroma::from((green - red, hue, sum)),
                    Sextant::BlueCyan => Chroma::from((blue - red, hue, sum)),
                    Sextant::BlueMagenta => Chroma::from((blue - green, hue, sum)),
                },
            }
        } else {
            Chroma::ZERO
        }
    }
    fn value(&self) -> Prop {
        (self.sum() / 3).into()
    }

    fn hcv(&self) -> HCV {
        self.into()
    }

    fn rgb<L: LightLevel>(&self) -> RGB<L> {
        <[Prop; 3]>::from(*self).into()
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
                    Ordering::Equal => match self.chroma().cmp(&other.chroma()) {
                        Ordering::Equal => Some(self.sum().cmp(&self.sum())),
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

macro_rules! impl_from_unsigned_array {
    ($unsigned:ty) => {
        impl From<[$unsigned; 3]> for RGB<$unsigned> {
            fn from(array: [$unsigned; 3]) -> Self {
                Self(array)
            }
        }

        impl From<RGB<$unsigned>> for [$unsigned; 3] {
            fn from(rgb: RGB<$unsigned>) -> Self {
                rgb.0
            }
        }
    };
}

impl_from_unsigned_array!(u8);
impl_from_unsigned_array!(u16);
impl_from_unsigned_array!(u32);
impl_from_unsigned_array!(u64);

macro_rules! impl_from_float_array {
    ($float:ty) => {
        impl From<[$float; 3]> for RGB<$float> {
            fn from(array: [$float; 3]) -> Self {
                debug_assert!(array.iter().all(|a| *a >= 0.0 && *a <= 1.0));
                Self(array)
            }
        }

        impl From<RGB<$float>> for [$float; 3] {
            fn from(rgb: RGB<$float>) -> Self {
                rgb.0
            }
        }
    };
}

impl_from_float_array!(f32);
impl_from_float_array!(f64);

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
