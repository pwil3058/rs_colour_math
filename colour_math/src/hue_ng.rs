// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    cmp::Ordering,
    convert::{From, Into, TryFrom},
    ops::{Add, Index, Mul},
};

pub use crate::{
    chroma, hcv::*, hue::*, rgba::RGBA, urgb::URGB, ColourComponent, ColourInterface, HueConstants,
    RGBConstants, CCI,
};

use crate::chroma::HueData;
use crate::HueIfce;
use normalised_angles::Degrees;
use num_traits_plus::float_plus::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord)]
pub enum CMY {
    Magenta,
    Yellow,
    Cyan,
}

impl CMY {
    pub fn indices(&self) -> (CCI, CCI) {
        match self {
            CMY::Magenta => (CCI::Red, CCI::Blue),
            CMY::Yellow => (CCI::Red, CCI::Green),
            CMY::Cyan => (CCI::Green, CCI::Blue),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Sextant {
    BlueCyan,
    BlueMagenta,
    RedMagenta,
    RedYellow,
    GreenYellow,
    GreenCyan,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum IndicesValueOrder {
    Primary(CCI),
    Secondary(CMY),
    Other(OneSixth),
}

impl HueConstants for IndicesValueOrder {
    const RED: Self = Self::Primary(CCI::Red);
    const GREEN: Self = Self::Primary(CCI::Green);
    const BLUE: Self = Self::Primary(CCI::Blue);

    const CYAN: Self = Self::Secondary(CMY::Cyan);
    const MAGENTA: Self = Self::Secondary(CMY::Magenta);
    const YELLOW: Self = Self::Secondary(CMY::Yellow);
}

impl IndicesValueOrder {
    pub fn ord_index(&self) -> u8 {
        match *self {
            IndicesValueOrder::BLUE => 0,
            IndicesValueOrder::MAGENTA => 1,
            IndicesValueOrder::RED => 2,
            IndicesValueOrder::YELLOW => 3,
            IndicesValueOrder::GREEN => 4,
            IndicesValueOrder::CYAN => 5,
            _ => panic!("illegal IndicesValueOrder: {:?}", self),
        }
    }
}

// impl Index<u8> for IndicesValueOrder {
//     type Output = CCI;
//
//     fn index(&self, index: u8) -> &CCI {
//         &self.0[index as usize]
//     }
// }

// impl From<[CCI; 3]> for IndicesValueOrder {
//     fn from(array: [CCI; 3]) -> Self {
//         // debug_assert!(array.iter().all(|x| (*x).is_proportion()), "{:?}", array);
//         Self(array)
//     }
// }
//
// impl Default for IndicesValueOrder {
//     fn default() -> Self {
//         Self([CCI::Red, CCI::Green, CCI::Blue])
//     }
// }
//
// impl From<&[CCI; 3]> for IndicesValueOrder {
//     fn from(array: &[CCI; 3]) -> Self {
//         Self(*array)
//     }
// }

impl PartialOrd for IndicesValueOrder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.ord_index().partial_cmp(&other.ord_index())
    }
}

impl Ord for IndicesValueOrder {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Default)]
pub struct RGB<F: ColourComponent>(pub(crate) [F; 3]);

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

impl<F: ColourComponent> RGB<F> {
    pub(crate) fn is_valid(&self) -> bool {
        self.0.iter().all(|x| (*x).is_proportion())
    }

    pub fn iter(&self) -> impl Iterator<Item = &F> {
        self.0.iter()
    }

    pub(crate) fn sum(self) -> F {
        //self.0[CCI::Red] + self.0[CCI::Green] + self.0[CCI::Blue]
        self.0.iter().copied().sum()
    }

    pub(crate) fn x(self) -> F {
        self[CCI::Red] + (self[CCI::Green] + self[CCI::Blue]) * F::COS_120
    }

    pub(crate) fn y(self) -> F {
        (self[CCI::Green] - self[CCI::Blue]) * F::SIN_120
    }

    pub(crate) fn xy(self) -> (F, F) {
        (self.x(), self.y())
    }

    pub(crate) fn hypot(self) -> F {
        self.x().hypot(self.y())
    }

    pub(crate) fn indices_value_order(self) -> Option<IndicesValueOrder> {
        match self[CCI::Red].partial_cmp(&self[CCI::Green]).unwrap() {
            Ordering::Greater => match self[CCI::Green].partial_cmp(&self[CCI::Blue]).unwrap() {
                Ordering::Greater => {
                    Some(IndicesValueOrder::Other([CCI::Red, CCI::Green, CCI::Blue]))
                }
                Ordering::Less => match self[CCI::Red].partial_cmp(&self[CCI::Blue]).unwrap() {
                    Ordering::Greater => {
                        Some(IndicesValueOrder::Other([CCI::Red, CCI::Blue, CCI::Green]))
                    }
                    Ordering::Less => {
                        Some(IndicesValueOrder::Other([CCI::Blue, CCI::Red, CCI::Green]))
                    }
                    Ordering::Equal => Some(IndicesValueOrder::MAGENTA),
                },
                Ordering::Equal => Some(IndicesValueOrder::RED),
            },
            Ordering::Less => match self[CCI::Red].partial_cmp(&self[CCI::Blue]).unwrap() {
                Ordering::Greater => {
                    Some(IndicesValueOrder::Other([CCI::Green, CCI::Red, CCI::Blue]))
                }
                Ordering::Less => match self[CCI::Green].partial_cmp(&self[CCI::Blue]).unwrap() {
                    Ordering::Greater => {
                        Some(IndicesValueOrder::Other([CCI::Green, CCI::Blue, CCI::Red]))
                    }
                    Ordering::Less => {
                        Some(IndicesValueOrder::Other([CCI::Blue, CCI::Green, CCI::Red]))
                    }
                    Ordering::Equal => Some(IndicesValueOrder::CYAN),
                },
                Ordering::Equal => Some(IndicesValueOrder::GREEN),
            },
            Ordering::Equal => match self[CCI::Red].partial_cmp(&self[CCI::Blue]).unwrap() {
                Ordering::Greater => Some(IndicesValueOrder::YELLOW),
                Ordering::Less => Some(IndicesValueOrder::BLUE),
                Ordering::Equal => None, // actually grey
            },
        }
    }

    fn ff(&self, indices: (CCI, CCI), ks: (F, F)) -> F {
        self[indices.0] * ks.0 + self[indices.1] * ks.1
    }

    //Return a copy of the rgb with each component rotated by the specified
    //angle. This results in an rgb the same value but the hue angle rotated
    //by the specified amount.
    //NB the chroma will change when there are less than 3 non zero
    //components and in the case of 2 non zero components this change may
    //be undesirable.  If it is undesirable it can be avoided by using a
    //higher level wrapper function to adjust/restore the chroma value.
    //In some cases maintaining bof chroma and value will not be
    //possible due to the complex relationship between value and chroma.
    pub fn components_rotated(&self, delta_hue_angle: Degrees<F>) -> RGB<F> {
        fn calc_ks<F: ColourComponent>(delta_hue_angle: Degrees<F>) -> (F, F) {
            let a = delta_hue_angle.sin();
            let b = (Degrees::DEG_120 - delta_hue_angle).sin();
            let c = a + b;
            (b / c, a / c)
        }
        if delta_hue_angle > Degrees::DEG_0 {
            if delta_hue_angle > Degrees::DEG_120 {
                let ks = calc_ks(delta_hue_angle - Degrees::DEG_120);
                return RGB([
                    self.ff((CCI::Blue, CCI::Green), ks),
                    self.ff((CCI::Red, CCI::Blue), ks),
                    self.ff((CCI::Green, CCI::Red), ks),
                ]);
            } else {
                let ks = calc_ks(delta_hue_angle);
                return RGB([
                    self.ff((CCI::Red, CCI::Blue), ks),
                    self.ff((CCI::Green, CCI::Red), ks),
                    self.ff((CCI::Blue, CCI::Green), ks),
                ]);
            }
        } else if delta_hue_angle < Degrees::DEG_0 {
            if delta_hue_angle < -Degrees::DEG_120 {
                let ks = calc_ks(delta_hue_angle.abs() - Degrees::DEG_120);
                return RGB([
                    self.ff((CCI::Green, CCI::Blue), ks),
                    self.ff((CCI::Blue, CCI::Red), ks),
                    self.ff((CCI::Red, CCI::Green), ks),
                ]);
            } else {
                let ks = calc_ks(delta_hue_angle.abs());
                return RGB([
                    self.ff((CCI::Red, CCI::Green), ks),
                    self.ff((CCI::Green, CCI::Blue), ks),
                    self.ff((CCI::Blue, CCI::Red), ks),
                ]);
            }
        }
        *self
    }

    pub fn pango_string(&self) -> String {
        URGB::<u8>::from(*self).pango_string()
    }
}

impl<F: ColourComponent> Eq for RGB<F> {}

impl<F: ColourComponent> PartialOrd for RGB<F> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.0 == other.0 {
            Some(Ordering::Equal)
        } else if let Ok(hue_data) = HueData::<F>::try_from(self) {
            if let Ok(other_hue_data) = HueData::<F>::try_from(other) {
                // This orders via hue from CYAN to CYAN via GREEN, RED, BLUE in that order
                match hue_data.partial_cmp(&other_hue_data) {
                    Some(Ordering::Equal) => match self.sum().partial_cmp(&other.sum()) {
                        // since chroma correction for both is the same so hypot() will be OK
                        Some(Ordering::Equal) => self.hypot().partial_cmp(&other.hypot()),
                        ord => ord,
                    },
                    ord => ord,
                }
            } else {
                Some(Ordering::Greater)
            }
        } else if other.hue_angle().is_some() {
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

impl<F: ColourComponent + std::fmt::Debug + std::iter::Sum> FloatApproxEq<F> for RGB<F> {
    fn abs_diff(&self, other: &Self) -> F {
        let sum: F = self
            .0
            .iter()
            .zip(other.0.iter())
            .map(|(a, b)| (*a - *b).powi(2))
            .sum();
        sum.sqrt() / F::THREE
    }

    fn rel_diff_scale_factor(&self, other: &Self) -> F {
        self.value().max(other.value())
    }
}

impl<F: ColourComponent> Index<CCI> for RGB<F> {
    type Output = F;

    fn index(&self, index: CCI) -> &F {
        let i: usize = index.into();
        &self.0[i]
    }
}

impl<F: ColourComponent> Add for RGB<F> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let array: [F; 3] = [
            self.0[0] + other.0[0],
            self.0[1] + other.0[1],
            self.0[2] + other.0[2],
        ];
        array.into()
    }
}

impl<F: ColourComponent> Mul<F> for RGB<F> {
    type Output = Self;

    fn mul(self, scalar: F) -> Self {
        let array: [F; 3] = [self.0[0] * scalar, self.0[1] * scalar, self.0[2] * scalar];
        array.into()
    }
}

impl<F: ColourComponent> From<[F; 3]> for RGB<F> {
    fn from(array: [F; 3]) -> Self {
        debug_assert!(array.iter().all(|x| (*x).is_proportion()), "{:?}", array);
        Self(array)
    }
}

impl<F: ColourComponent> From<&[F; 3]> for RGB<F> {
    fn from(array: &[F; 3]) -> Self {
        debug_assert!(array.iter().all(|x| (*x).is_proportion()), "{:?}", array);
        Self(*array)
    }
}

impl<F: ColourComponent> From<&[F]> for RGB<F> {
    fn from(array: &[F]) -> Self {
        debug_assert!(array.len() == 3);
        debug_assert!(array.iter().all(|x| (*x).is_proportion()), "{:?}", array);
        Self([array[0], array[1], array[2]])
    }
}

impl<F: ColourComponent> From<&[u8]> for RGB<F> {
    fn from(array: &[u8]) -> Self {
        debug_assert_eq!(array.len(), 3);
        let divisor = F::from(255.0).unwrap();
        Self([
            F::from_u8(array[0]).unwrap() / divisor,
            F::from_u8(array[1]).unwrap() / divisor,
            F::from_u8(array[2]).unwrap() / divisor,
        ])
    }
}

impl<F: ColourComponent> From<&[u8; 3]> for RGB<F> {
    fn from(array: &[u8; 3]) -> Self {
        let divisor = F::from(255.0).unwrap();
        Self([
            F::from_u8(array[0]).unwrap() / divisor,
            F::from_u8(array[1]).unwrap() / divisor,
            F::from_u8(array[2]).unwrap() / divisor,
        ])
    }
}

impl<F: ColourComponent> From<&RGB<F>> for (F, F, F) {
    fn from(rgb: &RGB<F>) -> (F, F, F) {
        (rgb[CCI::Red], rgb[CCI::Green], rgb[CCI::Blue])
    }
}

impl<F: ColourComponent> From<&RGB<F>> for [F; 3] {
    fn from(rgb: &RGB<F>) -> [F; 3] {
        rgb.0
    }
}

impl<F: ColourComponent, G: ColourComponent> From<&RGB<F>> for RGB<G> {
    fn from(rgb: &RGB<F>) -> RGB<G> {
        Self([
            G::from(rgb[CCI::Red]).unwrap(),
            G::from(rgb[CCI::Green]).unwrap(),
            G::from(rgb[CCI::Blue]).unwrap(),
        ])
    }
}

impl<F: ColourComponent> ColourInterface<F> for RGB<F> {
    fn rgb(&self) -> RGB<F> {
        *self
    }

    fn rgba(&self) -> RGBA<F> {
        [self.0[0], self.0[1], self.0[2], F::ONE].into()
    }

    fn hcv(&self) -> HCV<F> {
        self.into()
    }

    fn hue(&self) -> Option<Hue<F>> {
        use std::convert::TryInto;
        if let Ok(hue) = (*self).try_into() {
            Some(hue)
        } else {
            None
        }
    }

    fn hue_angle(&self) -> Option<Degrees<F>> {
        if let Ok(hue_data) = HueData::<F>::try_from(*self) {
            Some(hue_data.hue_angle())
        } else {
            None
        }
    }

    fn is_grey(&self) -> bool {
        self.hypot() == F::ZERO
    }

    fn chroma(&self) -> F {
        let xy = self.xy();
        let hypot = xy.0.hypot(xy.1);
        if hypot == F::ZERO {
            F::ZERO
        } else {
            let second = chroma::calc_other_from_xy(xy);
            (hypot * chroma::calc_chroma_correction(second)).min(F::ONE)
        }
    }

    fn max_chroma_rgb(&self) -> RGB<F> {
        if let Some(io) = self.indices_value_order() {
            let xy = self.xy();
            let mut array: [F; 3] = [F::ZERO, F::ZERO, F::ZERO];
            array[io[0] as usize] = F::ONE;
            array[io[1] as usize] = chroma::calc_other_from_xy_alt(xy);
            array.into()
        } else {
            *self
        }
    }

    fn greyness(&self) -> F {
        let xy = self.xy();
        let hypot = xy.0.hypot(xy.1);
        if hypot == F::ZERO {
            F::ONE
        } else {
            let second = chroma::calc_other_from_xy(xy);
            (F::ONE - hypot * chroma::calc_chroma_correction(second)).max(F::ZERO)
        }
    }

    fn value(&self) -> F {
        (self.sum() / F::THREE).min(F::ONE)
    }

    fn monochrome_rgb(&self) -> RGB<F> {
        let value = self.value();
        [value, value, value].into()
    }

    fn warmth(&self) -> F {
        ((self.x() + F::ONE).max(F::ZERO) / F::TWO).min(F::ONE)
    }

    fn warmth_rgb(&self) -> RGB<F> {
        let x = self.x();
        if x < F::ZERO {
            let temp = x.abs() + (F::ONE + x) * F::HALF;
            [F::ZERO, temp, temp].into()
        } else if x > F::ZERO {
            [x + (F::ONE - x) * F::HALF, F::ZERO, F::ZERO].into()
        } else {
            [F::HALF, F::HALF, F::HALF].into()
        }
    }

    fn best_foreground_rgb(&self) -> RGB<F> {
        if self[CCI::Red] * F::from(0.299).unwrap()
            + self[CCI::Green] * F::from(0.587).unwrap()
            + self[CCI::Blue] * F::from(0.114).unwrap()
            > F::HALF
        {
            Self::BLACK
        } else {
            Self::WHITE
        }
    }
}

#[cfg(test)]
mod rgb_tests {
    use super::*;

    #[test]
    fn rgb16_to_and_from_rgb() {
        assert_eq!(
            URGB::<u16>::from([0xffff, 0xffff, 0x0]),
            RGB::<f64>::YELLOW.into()
        );
        assert_eq!(
            RGB::<f32>::CYAN,
            URGB::<u16>::from([0, 0xffff, 0xffff]).into()
        );
    }

    #[test]
    fn rgb8_to_and_from_rgb() {
        assert_eq!(
            URGB::<u8>::from([0xff, 0xff, 0x0]),
            RGB::<f64>::YELLOW.into()
        );
        assert_eq!(RGB::<f32>::CYAN, URGB::<u8>::from([0, 0xff, 0xff]).into());
    }

    #[test]
    fn indices_order() {
        assert_eq!(RGB::<f64>::WHITE.indices_value_order(), None);
        assert_eq!(RGB::<f64>::BLACK.indices_value_order(), None);
        assert_eq!(
            RGB::<f64>::RED.indices_value_order(),
            Some(IndicesValueOrder::RED)
        );
        assert_eq!(
            RGB::<f64>::GREEN.indices_value_order(),
            Some(IndicesValueOrder::GREEN)
        );
        assert_eq!(
            RGB::<f64>::BLUE.indices_value_order(),
            Some(IndicesValueOrder::BLUE)
        );
        assert_eq!(
            RGB::<f64>::CYAN.indices_value_order(),
            Some(IndicesValueOrder::CYAN)
        );
        assert_eq!(
            RGB::<f64>::MAGENTA.indices_value_order(),
            Some(IndicesValueOrder::MAGENTA)
        );
        assert_eq!(
            RGB::<f64>::YELLOW.indices_value_order(),
            Some(IndicesValueOrder::YELLOW)
        );
    }

    #[test]
    fn rgb_order() {
        assert!(RGB::<f64>::BLACK < RGB::<f64>::WHITE);
        for rgb in RGB::<f64>::PRIMARIES.iter() {
            assert!(RGB::<f64>::BLACK < *rgb);
            assert!(RGB::<f64>::WHITE < *rgb);
        }
        for rgb in RGB::<f64>::SECONDARIES.iter() {
            assert!(RGB::<f64>::BLACK < *rgb);
            assert!(RGB::<f64>::WHITE < *rgb);
        }
        let ordered = [
            RGB::<f64>::BLACK,
            RGB::WHITE,
            RGB::BLUE,
            RGB::MAGENTA,
            RGB::RED,
            RGB::YELLOW,
            RGB::GREEN,
            RGB::CYAN,
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
