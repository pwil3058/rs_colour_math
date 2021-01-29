// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    cmp::Ordering,
    convert::From,
    ops::{Add, Index, Mul},
};

pub use crate::{chroma, hcv::*, hue::*, ColourComponent, ColourInterface};

use crate::{rgb::RGB, HueConstants, RGBConstants};

use normalised_angles::Degrees;
use num_traits_plus::float_plus::*;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Default)]
pub struct RGBA<F: ColourComponent>([F; 4]);

impl<F: ColourComponent> HueConstants for RGBA<F> {
    const RED: Self = Self([F::ONE, F::ZERO, F::ZERO, F::ONE]);
    const GREEN: Self = Self([F::ZERO, F::ONE, F::ZERO, F::ONE]);
    const BLUE: Self = Self([F::ZERO, F::ZERO, F::ONE, F::ONE]);

    const CYAN: Self = Self([F::ZERO, F::ONE, F::ONE, F::ONE]);
    const MAGENTA: Self = Self([F::ONE, F::ZERO, F::ONE, F::ONE]);
    const YELLOW: Self = Self([F::ONE, F::ONE, F::ZERO, F::ONE]);
}

impl<F: ColourComponent> RGBConstants for RGBA<F> {
    const WHITE: Self = Self([F::ONE, F::ONE, F::ONE, F::ONE]);
    const BLACK: Self = Self([F::ZERO, F::ZERO, F::ZERO, F::ONE]);
}

impl<F: ColourComponent> RGBA<F> {
    pub fn iter(&self) -> impl Iterator<Item = &F> {
        self.0.iter()
    }

    pub(crate) fn sum(self) -> F {
        //self.0[I_RED] + self.0[I_GREEN] + self.0[I_BLUE]
        self.0.iter().copied().sum()
    }

    pub fn pango_string(&self) -> String {
        unimplemented!()
        //URGB::<u8>::from(*self).pango_string()
    }
}

impl<F: ColourComponent> Eq for RGBA<F> {}

impl<F: ColourComponent> PartialOrd for RGBA<F> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.rgb().partial_cmp(&other.rgb()) {
            Some(ordering) => match ordering {
                Ordering::Equal => self.0[3].partial_cmp(&other.0[3]),
                _ => Some(ordering),
            },
            None => None,
        }
    }
}

impl<F: ColourComponent> Ord for RGBA<F> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other)
            .expect("restricted range of values means this is OK")
    }
}

impl<F: ColourComponent + std::fmt::Debug + std::iter::Sum> FloatApproxEq<F> for RGBA<F> {
    fn abs_diff(&self, other: &Self) -> F {
        let sum: F = self
            .0
            .iter()
            .zip(other.0.iter())
            .map(|(a, b)| (*a - *b).powi(2))
            .sum();
        sum.sqrt() / F::FOUR
    }

    fn rel_diff_scale_factor(&self, other: &Self) -> F {
        self.value().max(other.value())
    }
}

impl<F: ColourComponent> Index<u8> for RGBA<F> {
    type Output = F;

    fn index(&self, index: u8) -> &F {
        &self.0[index as usize]
    }
}

impl<F: ColourComponent> Add for RGBA<F> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let array: [F; 4] = [
            self.0[0] + other.0[0],
            self.0[1] + other.0[1],
            self.0[2] + other.0[2],
            self.0[3] + other.0[3],
        ];
        array.into()
    }
}

impl<F: ColourComponent> Mul<F> for RGBA<F> {
    type Output = Self;

    fn mul(self, scalar: F) -> Self {
        let array: [F; 4] = [
            self.0[0] * scalar,
            self.0[1] * scalar,
            self.0[2] * scalar,
            self.0[3],
        ];
        array.into()
    }
}

impl<F: ColourComponent> From<[F; 4]> for RGBA<F> {
    fn from(array: [F; 4]) -> Self {
        debug_assert!(array.iter().all(|x| (*x).is_proportion()), "{:?}", array);
        Self(array)
    }
}

impl<F: ColourComponent> From<&[F; 4]> for RGBA<F> {
    fn from(array: &[F; 4]) -> Self {
        debug_assert!(array.iter().all(|x| (*x).is_proportion()), "{:?}", array);
        Self(*array)
    }
}

impl<F: ColourComponent> From<&[F]> for RGBA<F> {
    fn from(array: &[F]) -> Self {
        debug_assert!(array.len() == 4);
        debug_assert!(array.iter().all(|x| (*x).is_proportion()), "{:?}", array);
        Self([array[0], array[1], array[2], array[3]])
    }
}

impl<F: ColourComponent> From<&[u8]> for RGBA<F> {
    fn from(array: &[u8]) -> Self {
        debug_assert_eq!(array.len(), 3);
        let divisor = F::from(255.0).unwrap();
        Self([
            F::from_u8(array[0]).unwrap() / divisor,
            F::from_u8(array[1]).unwrap() / divisor,
            F::from_u8(array[2]).unwrap() / divisor,
            F::from_u8(array[3]).unwrap() / divisor,
        ])
    }
}

impl<F: ColourComponent> From<&[u8; 4]> for RGBA<F> {
    fn from(array: &[u8; 4]) -> Self {
        let divisor = F::from(255.0).unwrap();
        Self([
            F::from_u8(array[0]).unwrap() / divisor,
            F::from_u8(array[1]).unwrap() / divisor,
            F::from_u8(array[2]).unwrap() / divisor,
            F::from_u8(array[3]).unwrap() / divisor,
        ])
    }
}

impl<F: ColourComponent> From<&RGBA<F>> for (F, F, F, F) {
    fn from(rgb: &RGBA<F>) -> (F, F, F, F) {
        (rgb[0], rgb[1], rgb[2], rgb[3])
    }
}

impl<F: ColourComponent> From<&RGBA<F>> for [F; 4] {
    fn from(rgb: &RGBA<F>) -> [F; 4] {
        rgb.0
    }
}

impl<F: ColourComponent, G: ColourComponent> From<&RGBA<F>> for RGBA<G> {
    fn from(rgb: &RGBA<F>) -> RGBA<G> {
        Self([
            G::from(rgb[0]).unwrap(),
            G::from(rgb[1]).unwrap(),
            G::from(rgb[2]).unwrap(),
            G::from(rgb[3]).unwrap(),
        ])
    }
}

impl<F: ColourComponent> ColourInterface<F> for RGBA<F> {
    fn rgb(&self) -> RGB<F> {
        (&self.0[0..3]).into()
    }

    fn rgba(&self) -> RGBA<F> {
        *self
    }

    fn hcv(&self) -> HCV<F> {
        RGB::from(&self.0[0..3]).into()
    }

    fn hue(&self) -> Option<Hue<F>> {
        self.rgb().hue()
    }

    fn hue_angle(&self) -> Option<Degrees<F>> {
        self.rgb().hue_angle()
    }

    fn is_grey(&self) -> bool {
        self.rgb().is_grey()
    }

    fn chroma(&self) -> F {
        self.rgb().chroma()
    }

    fn max_chroma_rgb(&self) -> RGB<F> {
        self.rgb().max_chroma_rgb()
    }

    fn greyness(&self) -> F {
        self.rgb().greyness()
    }

    fn value(&self) -> F {
        (self.sum() / F::THREE).min(F::ONE)
    }

    fn monochrome_rgb(&self) -> RGB<F> {
        self.rgb().monochrome_rgb()
        //let value = self.value();
        //[value, value, value].into()
    }

    fn warmth(&self) -> F {
        self.rgb().warmth()
    }

    fn warmth_rgb(&self) -> RGB<F> {
        self.rgb().warmth_rgb()
    }

    fn best_foreground_rgb(&self) -> RGB<F> {
        self.rgb().best_foreground_rgb()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn rgb16_to_and_from_rgb() {
    //     assert_eq!(
    //         URGB::<u16>::from([0xffff, 0xffff, 0x0]),
    //         RGBA::<f64>::YELLOW.into()
    //     );
    //     assert_eq!(
    //         RGBA::<f32>::CYAN,
    //         URGB::<u16>::from([0, 0xffff, 0xffff]).into()
    //     );
    // }
    //
    // #[test]
    // fn rgb8_to_and_from_rgb() {
    //     assert_eq!(
    //         URGB::<u8>::from([0xff, 0xff, 0x0]),
    //         RGBA::<f64>::YELLOW.into()
    //     );
    //     assert_eq!(RGBA::<f32>::CYAN, URGB::<u8>::from([0, 0xff, 0xff]).into());
    // }
    //
    // #[test]
    // fn indices_order() {
    //     assert_eq!(
    //         RGBA::<f64>::WHITE.indices_value_order(),
    //         [I_RED, I_GREEN, I_BLUE]
    //     );
    //     assert_eq!(
    //         RGBA::<f64>::BLACK.indices_value_order(),
    //         [I_RED, I_GREEN, I_BLUE]
    //     );
    //     assert_eq!(
    //         RGBA::<f64>::RED.indices_value_order(),
    //         [I_RED, I_GREEN, I_BLUE]
    //     );
    //     assert_eq!(
    //         RGBA::<f64>::GREEN.indices_value_order(),
    //         [I_GREEN, I_RED, I_BLUE]
    //     );
    //     assert_eq!(
    //         RGBA::<f64>::BLUE.indices_value_order(),
    //         [I_BLUE, I_RED, I_GREEN]
    //     );
    //     assert_eq!(
    //         RGBA::<f64>::CYAN.indices_value_order(),
    //         [I_GREEN, I_BLUE, I_RED]
    //     );
    //     assert_eq!(
    //         RGBA::<f64>::MAGENTA.indices_value_order(),
    //         [I_RED, I_BLUE, I_GREEN]
    //     );
    //     assert_eq!(
    //         RGBA::<f64>::YELLOW.indices_value_order(),
    //         [I_RED, I_GREEN, I_BLUE]
    //     );
    // }

    #[test]
    fn rgb_order() {
        assert!(RGBA::<f64>::BLACK < RGBA::<f64>::WHITE);
        for rgb in RGBA::<f64>::PRIMARIES.iter() {
            assert!(RGBA::<f64>::BLACK < *rgb);
            assert!(RGBA::<f64>::WHITE < *rgb);
        }
        for rgb in RGBA::<f64>::SECONDARIES.iter() {
            assert!(RGBA::<f64>::BLACK < *rgb);
            assert!(RGBA::<f64>::WHITE < *rgb);
        }
        let ordered = [
            RGBA::<f64>::BLACK,
            RGBA::WHITE,
            RGBA::BLUE,
            RGBA::MAGENTA,
            RGBA::RED,
            RGBA::YELLOW,
            RGBA::GREEN,
            RGBA::CYAN,
        ];
        for (i, i_rgb) in ordered.iter().enumerate() {
            for (j, j_rgb) in ordered.iter().enumerate() {
                println!(
                    "i: {} {:?} j: {} {:?}",
                    i,
                    i_rgb.hue_angle(),
                    j,
                    j_rgb.hue_angle()
                );
                if i < j {
                    assert!(i_rgb < j_rgb);
                } else if i > j {
                    assert!(i_rgb > j_rgb);
                } else {
                    assert_eq!(i_rgb, j_rgb);
                }
            }
        }
    }
}
