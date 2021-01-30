// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    cmp::Ordering,
    convert::{Into, TryFrom},
};

pub use crate::{
    chroma, hcv::*, rgb::RGB, urgb::URGB, ColourComponent, ColourInterface, HueConstants,
    RGBConstants, CCI,
};
use normalised_angles::Degrees;

pub trait HueIfceTmp<F: ColourComponent> {
    fn hue_angle(&self) -> Degrees<F>;
    fn chroma_correction(&self) -> F;
    fn sum_range_for_chroma(&self, chroma: F) -> (F, F);
    fn max_chroma_for_sum(&self, sum: F) -> F;

    fn max_chroma_rgb(&self) -> RGB<F>;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord)]
pub enum RGBHue {
    Red = 5,
    Green = 9,
    Blue = 1,
}

impl<F: ColourComponent> HueIfceTmp<F> for RGBHue {
    fn hue_angle(&self) -> Degrees<F> {
        match self {
            RGBHue::Red => Degrees::RED,
            RGBHue::Green => Degrees::GREEN,
            RGBHue::Blue => Degrees::BLUE,
        }
    }

    fn chroma_correction(&self) -> F {
        F::ONE
    }

    fn sum_range_for_chroma(&self, chroma: F) -> (F, F) {
        debug_assert!(chroma.is_proportion(), "chroma: {:?}", chroma);
        if chroma == F::ONE {
            (F::ONE, F::ONE)
        } else {
            (chroma, (F::THREE - F::TWO * chroma).min(F::THREE))
        }
    }

    fn max_chroma_for_sum(&self, sum: F) -> F {
        debug_assert!(sum >= F::ZERO && sum <= F::THREE, "sum: {:?}", sum);
        if sum == F::ZERO || sum == F::THREE {
            F::ZERO
        } else if sum < F::ONE {
            sum
        } else if sum > F::ONE {
            ((F::THREE - sum) / F::TWO).min(F::ONE)
        } else {
            F::ONE
        }
    }

    fn max_chroma_rgb(&self) -> RGB<F> {
        match self {
            RGBHue::Red => RGB::RED,
            RGBHue::Green => RGB::GREEN,
            RGBHue::Blue => RGB::BLUE,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord)]
pub enum CMYHue {
    Cyan = 113,
    Magenta = 3,
    Yellow = 7,
}

impl<F: ColourComponent> HueIfceTmp<F> for CMYHue {
    fn hue_angle(&self) -> Degrees<F> {
        match self {
            CMYHue::Cyan => Degrees::CYAN,
            CMYHue::Magenta => Degrees::MAGENTA,
            CMYHue::Yellow => Degrees::YELLOW,
        }
    }

    fn chroma_correction(&self) -> F {
        F::ONE
    }

    fn sum_range_for_chroma(&self, chroma: F) -> (F, F) {
        debug_assert!(chroma.is_proportion(), "chroma: {:?}", chroma);
        if chroma == F::ONE {
            (F::TWO, F::TWO)
        } else {
            (chroma * F::TWO, F::THREE - chroma)
        }
    }

    fn max_chroma_for_sum(&self, sum: F) -> F {
        debug_assert!(sum >= F::ZERO && sum <= F::THREE, "sum: {:?}", sum);
        if sum == F::ZERO || sum == F::THREE {
            F::ZERO
        } else if sum < F::TWO {
            (sum / (F::TWO)).min(F::ONE)
        } else if sum > F::TWO {
            (F::THREE - sum).min(F::ONE)
        } else {
            F::ONE
        }
    }

    fn max_chroma_rgb(&self) -> RGB<F> {
        match self {
            CMYHue::Cyan => RGB::CYAN,
            CMYHue::Magenta => RGB::MAGENTA,
            CMYHue::Yellow => RGB::YELLOW,
        }
    }
}

impl CMYHue {
    pub fn indices(&self) -> (CCI, CCI) {
        match self {
            CMYHue::Magenta => (CCI::Red, CCI::Blue),
            CMYHue::Yellow => (CCI::Red, CCI::Green),
            CMYHue::Cyan => (CCI::Green, CCI::Blue),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Sextant {
    RedMagenta = 4,
    RedYellow = 6,
    GreenYellow = 8,
    GreenCyan = 10,
    BlueCyan = 0,
    BlueMagenta = 2,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord)]
pub struct SextantHue<F: ColourComponent>(Sextant, F);

impl<F: ColourComponent> Eq for SextantHue<F> {}

impl<F: ColourComponent> From<(Sextant, &RGB<F>)> for SextantHue<F> {
    fn from(arg: (Sextant, &RGB<F>)) -> Self {
        let (sextant, rgb) = arg;
        use Sextant::*;
        use CCI::*;
        match sextant {
            RedMagenta => Self(sextant, (rgb[Blue] - rgb[Green]) / rgb[Red]),
            RedYellow => Self(sextant, (rgb[Green] - rgb[Blue]) / rgb[Red]),
            GreenYellow => Self(sextant, (rgb[Red] - rgb[Blue]) / rgb[Green]),
            GreenCyan => Self(sextant, (rgb[Blue] - rgb[Red]) / rgb[Green]),
            BlueCyan => Self(sextant, (rgb[Green] - rgb[Red]) / rgb[Blue]),
            BlueMagenta => Self(sextant, (rgb[Red] - rgb[Green]) / rgb[Blue]),
        }
    }
}

impl<F: ColourComponent> HueIfceTmp<F> for SextantHue<F> {
    fn hue_angle(&self) -> Degrees<F> {
        let sin = F::SQRT_3 * self.1 / F::TWO / (F::ONE - self.1 + self.1.powi(2)).sqrt();
        let angle = Degrees::asin(sin);
        match self.0 {
            Sextant::RedMagenta => -angle,
            Sextant::RedYellow => angle,
            Sextant::GreenYellow => Degrees::GREEN - angle,
            Sextant::GreenCyan => Degrees::GREEN + angle,
            Sextant::BlueCyan => Degrees::BLUE - angle,
            Sextant::BlueMagenta => Degrees::BLUE + angle,
        }
    }

    fn chroma_correction(&self) -> F {
        // Careful of fact floats only approximate real numbers
        (F::ONE + self.1 * self.1 - self.1)
            .sqrt()
            .min(F::ONE)
            .recip()
    }

    fn sum_range_for_chroma(&self, chroma: F) -> (F, F) {
        debug_assert!(chroma.is_proportion(), "chroma: {:?}", chroma);
        if chroma == F::ONE {
            let temp = (F::ONE + self.1).min(F::TWO);
            (temp, temp)
        } else {
            let temp = self.1 * chroma;
            (
                (chroma + temp).min(F::THREE),
                (F::THREE + temp - F::TWO * chroma).min(F::THREE),
            )
        }
    }

    fn max_chroma_for_sum(&self, sum: F) -> F {
        debug_assert!(sum >= F::ZERO && sum <= F::THREE, "sum: {:?}", sum);
        if sum == F::ZERO || sum == F::THREE {
            F::ZERO
        } else if sum < F::ONE + self.1 {
            (sum / (F::ONE + self.1)).min(F::ONE)
        } else if sum > F::ONE + self.1 {
            ((F::THREE - sum) / (F::TWO - self.1)).min(F::ONE)
        } else {
            F::ONE
        }
    }

    fn max_chroma_rgb(&self) -> RGB<F> {
        match self.0 {
            Sextant::RedMagenta => [F::ONE, F::ZERO, self.1].into(),
            Sextant::RedYellow => [F::ONE, self.1, F::ZERO].into(),
            Sextant::GreenYellow => [self.1, F::ONE, F::ZERO].into(),
            Sextant::GreenCyan => [F::ZERO, F::ONE, self.1].into(),
            Sextant::BlueCyan => [F::ZERO, self.1, F::ONE].into(),
            Sextant::BlueMagenta => [self.1, F::ZERO, F::ONE].into(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Hue<F: ColourComponent> {
    Primary(RGBHue),
    Secondary(CMYHue),
    Other(SextantHue<F>),
}

impl<F: ColourComponent> Eq for Hue<F> {}

impl<F: ColourComponent> HueConstants for Hue<F> {
    const RED: Self = Self::Primary(RGBHue::Red);
    const GREEN: Self = Self::Primary(RGBHue::Green);
    const BLUE: Self = Self::Primary(RGBHue::Blue);

    const CYAN: Self = Self::Secondary(CMYHue::Cyan);
    const MAGENTA: Self = Self::Secondary(CMYHue::Magenta);
    const YELLOW: Self = Self::Secondary(CMYHue::Yellow);
}

impl<F: ColourComponent> TryFrom<&RGB<F>> for Hue<F> {
    type Error = &'static str;

    fn try_from(rgb: &RGB<F>) -> Result<Self, Self::Error> {
        use Sextant::*;
        match rgb[CCI::Red].partial_cmp(&rgb[CCI::Green]).unwrap() {
            Ordering::Greater => match rgb[CCI::Green].partial_cmp(&rgb[CCI::Blue]).unwrap() {
                Ordering::Greater => Ok(Hue::Other(SextantHue::from((RedYellow, rgb)))),
                Ordering::Less => match rgb[CCI::Red].partial_cmp(&rgb[CCI::Blue]).unwrap() {
                    Ordering::Greater => Ok(Hue::Other(SextantHue::from((RedMagenta, rgb)))),
                    Ordering::Less => Ok(Hue::Other(SextantHue::from((BlueMagenta, rgb)))),
                    Ordering::Equal => Ok(Hue::Secondary(CMYHue::Magenta)),
                },
                Ordering::Equal => Ok(Hue::Primary(RGBHue::Red)),
            },
            Ordering::Less => match rgb[CCI::Red].partial_cmp(&rgb[CCI::Blue]).unwrap() {
                Ordering::Greater => Ok(Hue::Other(SextantHue::from((GreenYellow, rgb)))),
                Ordering::Less => match rgb[CCI::Green].partial_cmp(&rgb[CCI::Blue]).unwrap() {
                    Ordering::Greater => Ok(Hue::Other(SextantHue::from((GreenCyan, rgb)))),
                    Ordering::Less => Ok(Hue::Other(SextantHue::from((BlueCyan, rgb)))),
                    Ordering::Equal => Ok(Hue::Secondary(CMYHue::Cyan)),
                },
                Ordering::Equal => Ok(Hue::Primary(RGBHue::Green)),
            },
            Ordering::Equal => match rgb[CCI::Red].partial_cmp(&rgb[CCI::Blue]).unwrap() {
                Ordering::Greater => Ok(Hue::Secondary(CMYHue::Yellow)),
                Ordering::Less => Ok(Hue::Primary(RGBHue::Blue)),
                Ordering::Equal => Err("RGB is grey and hs no hue"),
            },
        }
    }
}

impl<F: ColourComponent> HueIfceTmp<F> for Hue<F> {
    fn hue_angle(&self) -> Degrees<F> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.hue_angle(),
            Self::Secondary(cmy_hue) => cmy_hue.hue_angle(),
            Self::Other(sextant_hue) => sextant_hue.hue_angle(),
        }
    }

    fn chroma_correction(&self) -> F {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.chroma_correction(),
            Self::Secondary(cmy_hue) => cmy_hue.chroma_correction(),
            Self::Other(sextant_hue) => sextant_hue.chroma_correction(),
        }
    }

    fn sum_range_for_chroma(&self, chroma: F) -> (F, F) {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.sum_range_for_chroma(chroma),
            Self::Secondary(cmy_hue) => cmy_hue.sum_range_for_chroma(chroma),
            Self::Other(sextant_hue) => sextant_hue.sum_range_for_chroma(chroma),
        }
    }

    fn max_chroma_for_sum(&self, sum: F) -> F {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.max_chroma_for_sum(sum),
            Self::Secondary(cmy_hue) => cmy_hue.max_chroma_for_sum(sum),
            Self::Other(sextant_hue) => sextant_hue.max_chroma_for_sum(sum),
        }
    }

    fn max_chroma_rgb(&self) -> RGB<F> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.max_chroma_rgb(),
            Self::Secondary(cmy_hue) => cmy_hue.max_chroma_rgb(),
            Self::Other(sextant_hue) => sextant_hue.max_chroma_rgb(),
        }
    }
}

impl<F: ColourComponent> Hue<F> {
    pub fn ord_index(&self) -> u8 {
        0
    }
}

impl<F: ColourComponent> PartialOrd for Hue<F> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.ord_index().partial_cmp(&other.ord_index())
    }
}

impl<F: ColourComponent> Ord for Hue<F> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[cfg(test)]
mod hue_ng_tests {
    use super::*;
    use num_traits_plus::{assert_approx_eq, float_plus::FloatApproxEq};

    #[test]
    fn hue_from_rgb() {
        for rgb in &[RGB::<f64>::BLACK, RGB::WHITE, RGB::from([0.5, 0.5, 0.5])] {
            assert!(Hue::<f64>::try_from(rgb).is_err());
        }
        for (rgb, hue) in RGB::<f64>::PRIMARIES.iter().zip(Hue::PRIMARIES.iter()) {
            assert_eq!(Hue::<f64>::try_from(rgb), Ok(*hue));
            assert_eq!(Hue::<f64>::try_from(&(*rgb * 0.5)), Ok(*hue));
        }
        for (rgb, hue) in RGB::<f64>::SECONDARIES.iter().zip(Hue::SECONDARIES.iter()) {
            assert_eq!(Hue::<f64>::try_from(rgb), Ok(*hue));
            assert_eq!(Hue::<f64>::try_from(&(*rgb * 0.5)), Ok(*hue));
        }
        for (array, sextant, second) in &[
            ([1.0, 0.5, 0.0], Sextant::RedYellow, 0.5),
            ([0.0, 0.25, 0.5], Sextant::BlueCyan, 0.5),
            ([0.2, 0.0, 0.4], Sextant::BlueMagenta, 0.5),
            ([0.5, 0.0, 1.0], Sextant::BlueMagenta, 0.5),
            ([1.0, 0.0, 0.5], Sextant::RedMagenta, 0.5),
            ([0.5, 1.0, 0.0], Sextant::GreenYellow, 0.5),
            ([0.0, 1.0, 0.5], Sextant::GreenCyan, 0.5),
        ] {
            let rgb = RGB::<f64>::from(array);
            let hue = Hue::Other(SextantHue(*sextant, *second));
            assert_eq!(Hue::<f64>::try_from(&rgb), Ok(hue));
        }
    }

    #[test]
    fn hue_max_chroma_rgb() {
        for (hue, rgb) in Hue::<f64>::PRIMARIES.iter().zip(RGB::PRIMARIES.iter()) {
            assert_eq!(hue.max_chroma_rgb(), *rgb);
        }
        for (hue, rgb) in Hue::<f64>::SECONDARIES.iter().zip(RGB::SECONDARIES.iter()) {
            assert_eq!(hue.max_chroma_rgb(), *rgb);
        }
        for (array, sextant, second) in &[
            ([1.0, 0.5, 0.0], Sextant::RedYellow, 0.5),
            ([0.0, 0.5, 1.0], Sextant::BlueCyan, 0.5),
            ([0.5, 0.0, 1.0], Sextant::BlueMagenta, 0.5),
            ([1.0, 0.0, 0.5], Sextant::RedMagenta, 0.5),
            ([0.5, 1.0, 0.0], Sextant::GreenYellow, 0.5),
            ([0.0, 1.0, 0.5], Sextant::GreenCyan, 0.5),
        ] {
            let rgb = RGB::<f64>::from(array);
            let hue = Hue::Other(SextantHue(*sextant, *second));
            assert_eq!(Hue::<f64>::try_from(&rgb), Ok(hue));
        }
    }

    #[test]
    fn hue_angle() {
        for (hue, angle) in Hue::<f64>::PRIMARIES
            .iter()
            .zip(Degrees::<f64>::PRIMARIES.iter())
        {
            assert_eq!(hue.hue_angle(), *angle);
        }
        for (hue, angle) in Hue::<f64>::SECONDARIES
            .iter()
            .zip(Degrees::<f64>::SECONDARIES.iter())
        {
            assert_eq!(hue.hue_angle(), *angle);
        }
        for (sextant, second, angle) in &[
            (Sextant::RedYellow, 0.5, Degrees::<f64>::DEG_30),
            (Sextant::BlueCyan, 0.5, Degrees::<f64>::NEG_DEG_150),
            (Sextant::BlueMagenta, 0.5, Degrees::<f64>::NEG_DEG_90),
            (Sextant::RedMagenta, 0.5, Degrees::<f64>::NEG_DEG_30),
            (Sextant::GreenYellow, 0.5, Degrees::<f64>::DEG_90),
            (Sextant::GreenCyan, 0.5, Degrees::<f64>::DEG_150),
        ] {
            let hue = Hue::Other(SextantHue(*sextant, *second));
            assert_approx_eq!(hue.hue_angle(), *angle, 0.0000001);
        }
    }
}
