// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    cmp::Ordering,
    convert::{Into, TryFrom},
};

use normalised_angles::Degrees;
use num_traits_plus::{float_plus::FloatApproxEq, NumberConstants};

use crate::{
    proportion::{Chroma, Float, Proportion, ProportionConstants, Sum, Validation},
    rgb::RGB,
    ChromaOneRGB, HueAngle, HueConstants, RGBConstants, CCI,
};
use std::fmt::Debug;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct SumRange<F: Float>((Sum<F>, Sum<F>, Sum<F>));

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum SumOrdering<F: Float> {
    TooSmall,
    Shade(Sum<F>, Sum<F>),
    Tint(Sum<F>, Sum<F>),
    TooBig,
}

impl<F: Float> SumOrdering<F> {
    pub fn is_failure(&self) -> bool {
        use SumOrdering::*;
        match self {
            TooSmall | TooBig => true,
            _ => false,
        }
    }

    pub fn is_success(&self) -> bool {
        use SumOrdering::*;
        match self {
            TooSmall | TooBig => false,
            _ => true,
        }
    }
}

impl<F: Float> SumRange<F> {
    pub fn compare_sum(&self, sum: Sum<F>) -> SumOrdering<F> {
        if sum < self.0 .0 {
            SumOrdering::TooSmall
        } else if sum <= self.0 .1 {
            SumOrdering::Shade(self.0 .0, self.0 .1)
        } else if sum < self.0 .2 {
            SumOrdering::Tint(self.0 .1, self.0 .2)
        } else {
            SumOrdering::TooBig
        }
    }

    pub fn shade_min(&self) -> Sum<F> {
        self.0 .0
    }

    pub fn shade_max(&self) -> Sum<F> {
        self.0 .1
    }

    pub fn tint_min(&self) -> Sum<F> {
        self.0 .1
    }

    pub fn tint_max(&self) -> Sum<F> {
        self.0 .2
    }
}

pub trait HueIfceTmp<F: Float> {
    fn sum_range_for_chroma(&self, chroma_value: Proportion<F>) -> Option<SumRange<F>>;
    fn max_chroma_for_sum(&self, sum: Sum<F>) -> Option<Chroma<F>>;

    fn max_chroma_rgb(&self) -> RGB<F>;
    fn max_chroma_rgb_for_sum(&self, sum: Sum<F>) -> Option<RGB<F>>;
    fn min_sum_rgb_for_chroma(&self, chroma_value: Proportion<F>) -> RGB<F>;
    fn max_sum_rgb_for_chroma(&self, chroma_value: Proportion<F>) -> RGB<F>;
    fn rgb_for_sum_and_chroma(&self, sum: Sum<F>, chroma_value: Proportion<F>) -> Option<RGB<F>>;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord)]
pub enum RGBHue {
    Red = 5,
    Green = 9,
    Blue = 1,
}

impl RGBHue {
    fn make_rgb<F: Float>(&self, components: (Proportion<F>, Proportion<F>)) -> RGB<F> {
        use RGBHue::*;
        match self {
            Red => [components.0, components.1, components.1].into(),
            Green => [components.1, components.0, components.1].into(),
            Blue => [components.1, components.1, components.0].into(),
        }
    }
}

impl<F: Float> HueAngle<F> for RGBHue {
    fn hue_angle(&self) -> Degrees<F> {
        match self {
            RGBHue::Red => Degrees::RED,
            RGBHue::Green => Degrees::GREEN,
            RGBHue::Blue => Degrees::BLUE,
        }
    }
}

impl<F: Float> ChromaOneRGB<F> for RGBHue {
    /// RGB wih chroma of 1.0 chroma and with its hue (value may change op or down)
    fn chroma_one_rgb(&self) -> RGB<F> {
        match self {
            RGBHue::Red => RGB::RED,
            RGBHue::Green => RGB::GREEN,
            RGBHue::Blue => RGB::BLUE,
        }
    }
}

impl<F: Float> HueIfceTmp<F> for RGBHue {
    fn sum_range_for_chroma(&self, chroma: Proportion<F>) -> Option<SumRange<F>> {
        debug_assert!(chroma.is_valid(), "chroma: {:?}", chroma);
        if chroma == Proportion::P_ZERO {
            None
        } else {
            Some(SumRange((
                chroma.into(),
                Sum::ONE,
                (Sum::THREE - Sum::TWO * chroma).min(Sum::THREE),
            )))
        }
    }

    fn max_chroma_for_sum(&self, sum: Sum<F>) -> Option<Chroma<F>> {
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        if sum == Sum::ZERO || sum == Sum::THREE {
            None
        } else if sum < Sum::ONE {
            Some(Chroma::Shade(sum.into()))
        } else if sum > Sum::ONE {
            Some(Chroma::Tint(((Sum::THREE - sum) / Sum::TWO).into()))
        } else {
            Some(Chroma::ONE)
        }
    }

    fn max_chroma_rgb(&self) -> RGB<F> {
        match self {
            RGBHue::Red => RGB::RED,
            RGBHue::Green => RGB::GREEN,
            RGBHue::Blue => RGB::BLUE,
        }
    }

    fn max_chroma_rgb_for_sum(&self, sum: Sum<F>) -> Option<RGB<F>> {
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        if sum == Sum::ZERO || sum == Sum::THREE {
            None
        } else {
            if sum <= Sum::ONE {
                Some(self.make_rgb((sum.into(), Proportion::P_ZERO)))
            } else {
                Some(self.make_rgb((
                    Proportion::P_ONE,
                    ((sum - Sum::ONE) / Sum::TWO).min(Sum::P_ONE).into(),
                )))
            }
        }
    }

    fn min_sum_rgb_for_chroma(&self, chroma: Proportion<F>) -> RGB<F> {
        // TODO: Needs major revision taking into account Shade/Tint
        debug_assert!(chroma.is_valid(), "chroma: {:?}", chroma);
        if chroma == Proportion::P_ZERO {
            RGB::BLACK
        } else if chroma == Proportion::P_ONE {
            self.max_chroma_rgb()
        } else {
            self.make_rgb((chroma, Proportion::P_ZERO))
        }
    }

    fn max_sum_rgb_for_chroma(&self, chroma: Proportion<F>) -> RGB<F> {
        // TODO: Needs major revision taking into account Shade/Tint
        debug_assert!(chroma.is_valid(), "chroma: {:?}", chroma);
        if chroma == Proportion::P_ZERO {
            RGB::WHITE
        } else if chroma == Proportion::P_ONE {
            self.max_chroma_rgb()
        } else {
            self.make_rgb((Proportion::P_ONE, Proportion::P_ONE - chroma))
        }
    }

    fn rgb_for_sum_and_chroma(&self, sum: Sum<F>, chroma: Proportion<F>) -> Option<RGB<F>> {
        // TODO: Needs major revision taking into account Shade/Tint
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        debug_assert!(chroma.is_valid(), "chroma: {:?}", chroma);
        let sum_range = self.sum_range_for_chroma(chroma)?;
        if sum_range.compare_sum(sum).is_success() {
            let other: Proportion<F> = ((sum - chroma) / Sum::THREE).into();
            Some(self.make_rgb(((other + chroma).into(), other)))
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord)]
pub enum CMYHue {
    Cyan = 113,
    Magenta = 3,
    Yellow = 7,
}

impl CMYHue {
    fn make_rgb<F: Float>(&self, components: (Proportion<F>, Proportion<F>)) -> RGB<F> {
        use CMYHue::*;
        match self {
            Cyan => [components.1, components.0, components.0].into(),
            Magenta => [components.0, components.1, components.0].into(),
            Yellow => [components.0, components.0, components.1].into(),
        }
    }
}

impl<F: Float> HueAngle<F> for CMYHue {
    fn hue_angle(&self) -> Degrees<F> {
        match self {
            CMYHue::Cyan => Degrees::CYAN,
            CMYHue::Magenta => Degrees::MAGENTA,
            CMYHue::Yellow => Degrees::YELLOW,
        }
    }
}

impl<F: Float> HueIfceTmp<F> for CMYHue {
    fn sum_range_for_chroma(&self, chroma: Proportion<F>) -> Option<SumRange<F>> {
        debug_assert!(chroma.is_valid(), "chroma: {:?}", chroma);
        if chroma == Proportion::P_ZERO {
            None
        } else {
            let c_sum: Sum<F> = chroma.into();
            Some(SumRange((c_sum * Sum::TWO, Sum::TWO, Sum::THREE - c_sum)))
        }
    }

    fn max_chroma_for_sum(&self, sum: Sum<F>) -> Option<Chroma<F>> {
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        if sum == Sum::ZERO || sum == Sum::THREE {
            None
        } else if sum < Sum::TWO {
            Some(Chroma::Shade((sum / Sum::TWO).min(Sum::P_ONE).into()))
        } else if sum > Sum::TWO {
            Some(Chroma::Tint((Sum::THREE - sum).min(Sum::ONE).into()))
        } else {
            Some(Chroma::ONE)
        }
    }

    fn max_chroma_rgb(&self) -> RGB<F> {
        match self {
            CMYHue::Cyan => RGB::CYAN,
            CMYHue::Magenta => RGB::MAGENTA,
            CMYHue::Yellow => RGB::YELLOW,
        }
    }

    fn max_chroma_rgb_for_sum(&self, sum: Sum<F>) -> Option<RGB<F>> {
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        if sum == Sum::ZERO || sum == Sum::THREE {
            None
        } else if sum < Sum::TWO {
            Some(self.make_rgb(((sum / Sum::TWO).min(Sum::P_ONE).into(), Proportion::P_ZERO)))
        } else if sum > Sum::TWO {
            Some(self.make_rgb((
                Proportion::P_ONE,
                (sum - Sum::TWO).max(Sum::ZERO).min(Sum::ONE).into(),
            )))
        } else {
            Some(self.max_chroma_rgb())
        }
    }

    fn min_sum_rgb_for_chroma(&self, chroma: Proportion<F>) -> RGB<F> {
        // TODO: Needs major revision taking into account Shade/Tint
        debug_assert!(chroma.is_valid(), "chroma: {:?}", chroma);
        if chroma == Proportion::P_ZERO {
            RGB::BLACK
        } else if chroma == Proportion::P_ONE {
            self.max_chroma_rgb()
        } else {
            self.make_rgb((chroma, Proportion::P_ZERO))
        }
    }

    fn max_sum_rgb_for_chroma(&self, chroma: Proportion<F>) -> RGB<F> {
        // TODO: Needs major revision taking into account Shade/Tint
        debug_assert!(chroma.is_valid(), "chroma: {:?}", chroma);
        if chroma == Proportion::P_ZERO {
            RGB::WHITE
        } else if chroma == Proportion::P_ONE {
            self.max_chroma_rgb()
        } else {
            self.make_rgb((Proportion::P_ONE, Proportion::P_ONE - chroma))
        }
    }

    fn rgb_for_sum_and_chroma(&self, sum: Sum<F>, chroma: Proportion<F>) -> Option<RGB<F>> {
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        debug_assert!(chroma.is_valid());
        let sum_range = self.sum_range_for_chroma(chroma)?;
        if sum_range.compare_sum(sum).is_success() {
            // TODO: reassess this calculation
            Some(self.make_rgb((
                ((sum + chroma) / Sum::THREE).into(),
                ((sum - Sum::TWO * chroma) / Sum::THREE).into(),
            )))
        } else {
            None
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
pub struct SextantHue<F: Float>(Sextant, Proportion<F>);

impl<F: Float> Eq for SextantHue<F> {}

impl<F: Float> SextantHue<F> {
    fn make_rgb(&self, components: (Proportion<F>, Proportion<F>, Proportion<F>)) -> RGB<F> {
        debug_assert!(
            components.0 >= components.1 && components.1 >= components.2,
            "{:?} >= {:?} >= {:?}",
            components.0,
            components.1,
            components.2
        );
        use Sextant::*;
        match self.0 {
            RedMagenta => [components.0, components.2, components.1].into(),
            RedYellow => [components.0, components.1, components.2].into(),
            GreenYellow => [components.1, components.0, components.2].into(),
            GreenCyan => [components.2, components.0, components.1].into(),
            BlueCyan => [components.2, components.1, components.0].into(),
            BlueMagenta => [components.1, components.2, components.0].into(),
        }
    }
}

impl<F: Float> FloatApproxEq<F> for SextantHue<F> {
    fn approx_eq(&self, other: &Self, max_diff: Option<F>) -> bool {
        if self.0 == other.0 {
            self.1.approx_eq(&other.1, max_diff)
        } else {
            false
        }
    }
}

impl<F: Float> From<(Sextant, &RGB<F>)> for SextantHue<F> {
    fn from(arg: (Sextant, &RGB<F>)) -> Self {
        use Sextant::*;
        use CCI::*;
        match arg.0 {
            RedMagenta => Self(
                arg.0,
                (arg.1[Blue] - arg.1[Green]) / (arg.1[Red] - arg.1[Green]),
            ),
            RedYellow => Self(
                arg.0,
                (arg.1[Green] - arg.1[Blue]) / (arg.1[Red]) - arg.1[Blue],
            ),
            GreenYellow => Self(
                arg.0,
                (arg.1[Red] - arg.1[Blue]) / (arg.1[Green]) - arg.1[Blue],
            ),
            GreenCyan => Self(
                arg.0,
                (arg.1[Blue] - arg.1[Red]) / (arg.1[Green]) - arg.1[Red],
            ),
            BlueCyan => Self(
                arg.0,
                (arg.1[Green] - arg.1[Red]) / (arg.1[Blue]) - arg.1[Red],
            ),
            BlueMagenta => Self(
                arg.0,
                (arg.1[Red] - arg.1[Green]) / (arg.1[Blue]) - arg.1[Green],
            ),
        }
    }
}

impl<P: Float> HueAngle<P> for SextantHue<P> {
    fn hue_angle(&self) -> Degrees<P> {
        let second: P = self.1.value().into();
        let sin = P::SQRT_3 * second / P::TWO / (P::ONE - second + second.powi(2)).sqrt();
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
}

impl<F: Float> HueIfceTmp<F> for SextantHue<F> {
    fn sum_range_for_chroma(&self, chroma: Proportion<F>) -> Option<SumRange<F>> {
        debug_assert!(chroma.is_valid(), "chroma: {:?}", chroma);
        if chroma == Proportion::P_ZERO {
            None
        } else {
            let max_c_sum = (Sum::ONE + self.1).min(Sum::TWO);
            if chroma == Proportion::P_ONE {
                Some(SumRange((max_c_sum, max_c_sum, max_c_sum)))
            } else {
                let temp: Sum<F> = (self.1 * chroma).into();
                Some(SumRange((
                    (temp + chroma).min(Sum::THREE),
                    max_c_sum,
                    (Sum::THREE + temp - Sum::TWO * chroma).min(Sum::THREE),
                )))
            }
        }
    }

    fn max_chroma_for_sum(&self, sum: Sum<F>) -> Option<Chroma<F>> {
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        if sum == Sum::ZERO || sum == Sum::THREE {
            None
        } else {
            match sum.cmp(&(Sum::ONE + self.1)) {
                Ordering::Less => {
                    let temp = (sum / (Sum::ONE + self.1)).min(Sum::ONE);
                    Some(Chroma::Shade(temp.into()))
                }
                Ordering::Greater => {
                    let temp = ((Sum::THREE - sum) / (Sum::TWO - self.1)).min(Sum::ONE);
                    Some(Chroma::Tint(temp.into()))
                }
                Ordering::Equal => Some(Chroma::ONE),
            }
        }
    }

    fn max_chroma_rgb(&self) -> RGB<F> {
        self.make_rgb((Proportion::P_ONE, self.1, Proportion::P_ZERO))
    }

    fn max_chroma_rgb_for_sum(&self, sum: Sum<F>) -> Option<RGB<F>> {
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        // TODO: make hue drift an error
        if sum == Sum::ZERO || sum == Sum::THREE {
            None
        } else {
            let max_chroma_sum = Sum::P_ONE + self.1;
            if sum == max_chroma_sum {
                Some(self.max_chroma_rgb())
            } else {
                let components = if sum < max_chroma_sum {
                    let first: Proportion<F> = (sum / max_chroma_sum).min(Sum::P_ONE).into();
                    (first, first * self.1, Proportion::P_ZERO)
                } else {
                    let temp = sum - Sum::P_ONE;
                    let second = ((temp + self.1) / Sum::TWO).min(Sum::P_ONE);
                    (
                        Proportion::P_ONE,
                        second.into(),
                        (temp - second).max(Sum::ZERO).into(),
                    )
                };
                Some(self.make_rgb(components))
            }
        }
    }

    fn min_sum_rgb_for_chroma(&self, chroma: Proportion<F>) -> RGB<F> {
        debug_assert!(chroma.is_valid(), "chroma: {:?}", chroma);
        if chroma == Proportion::P_ZERO {
            RGB::BLACK
        } else if chroma == Proportion::P_ONE {
            self.max_chroma_rgb()
        } else {
            self.make_rgb((chroma, self.1 * chroma, Proportion::P_ZERO))
        }
    }

    fn max_sum_rgb_for_chroma(&self, chroma: Proportion<F>) -> RGB<F> {
        debug_assert!(chroma.is_valid(), "chroma: {:?}", chroma);
        if chroma == Proportion::P_ZERO {
            RGB::WHITE
        } else if chroma == Proportion::P_ONE {
            self.max_chroma_rgb()
        } else {
            let third = Proportion::P_ONE - chroma;
            let second: Proportion<F> = (chroma * self.1 + third).into();
            self.make_rgb((Proportion::P_ONE, second, third))
        }
    }

    fn rgb_for_sum_and_chroma(&self, sum: Sum<F>, chroma: Proportion<F>) -> Option<RGB<F>> {
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        debug_assert!(chroma.is_valid());
        use SumOrdering::*;
        let sum_range = self.sum_range_for_chroma(chroma)?;
        match sum_range.compare_sum(sum) {
            Shade(_min_sum, _max_sun) => {
                // let delta: Proportion<F> = ((sum - min_sum) / Sum::THREE).into();
                // let first = chroma + delta;
                // let second = chroma * self.1 + delta;
                // Some(self.make_rgb((first, second, delta)))
                let base: Proportion<F> = ((sum - Sum::P_ONE) / Sum::TWO).into();
                let delta = chroma * self.1 / Proportion::TWO;
                Some(self.make_rgb((Proportion::P_ONE, base + delta, base - delta)))
            }
            Tint(_min_sum, _max_sum) => {
                let base: Proportion<F> = ((sum - Sum::P_ONE) / Sum::TWO).into();
                let delta = chroma * self.1 / Proportion::TWO;
                Some(self.make_rgb((Proportion::P_ONE, base + delta, base - delta)))
            }
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Hue<F: Float> {
    Primary(RGBHue),
    Secondary(CMYHue),
    Sextant(SextantHue<F>),
}

impl<F: Float> Eq for Hue<F> {}

impl<F: Float> HueConstants for Hue<F> {
    const RED: Self = Self::Primary(RGBHue::Red);
    const GREEN: Self = Self::Primary(RGBHue::Green);
    const BLUE: Self = Self::Primary(RGBHue::Blue);

    const CYAN: Self = Self::Secondary(CMYHue::Cyan);
    const MAGENTA: Self = Self::Secondary(CMYHue::Magenta);
    const YELLOW: Self = Self::Secondary(CMYHue::Yellow);
}

impl<F: Float> TryFrom<&RGB<F>> for Hue<F> {
    type Error = &'static str;

    fn try_from(rgb: &RGB<F>) -> Result<Self, Self::Error> {
        use Sextant::*;
        match rgb[CCI::Red].partial_cmp(&rgb[CCI::Green]).unwrap() {
            Ordering::Greater => match rgb[CCI::Green].partial_cmp(&rgb[CCI::Blue]).unwrap() {
                Ordering::Greater => Ok(Hue::Sextant(SextantHue::from((RedYellow, rgb)))),
                Ordering::Less => match rgb[CCI::Red].partial_cmp(&rgb[CCI::Blue]).unwrap() {
                    Ordering::Greater => Ok(Hue::Sextant(SextantHue::from((RedMagenta, rgb)))),
                    Ordering::Less => Ok(Hue::Sextant(SextantHue::from((BlueMagenta, rgb)))),
                    Ordering::Equal => Ok(Hue::Secondary(CMYHue::Magenta)),
                },
                Ordering::Equal => Ok(Hue::Primary(RGBHue::Red)),
            },
            Ordering::Less => match rgb[CCI::Red].partial_cmp(&rgb[CCI::Blue]).unwrap() {
                Ordering::Greater => Ok(Hue::Sextant(SextantHue::from((GreenYellow, rgb)))),
                Ordering::Less => match rgb[CCI::Green].partial_cmp(&rgb[CCI::Blue]).unwrap() {
                    Ordering::Greater => Ok(Hue::Sextant(SextantHue::from((GreenCyan, rgb)))),
                    Ordering::Less => Ok(Hue::Sextant(SextantHue::from((BlueCyan, rgb)))),
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

impl<F: Float> HueAngle<F> for Hue<F> {
    fn hue_angle(&self) -> Degrees<F> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.hue_angle(),
            Self::Secondary(cmy_hue) => cmy_hue.hue_angle(),
            Self::Sextant(sextant_hue) => sextant_hue.hue_angle(),
        }
    }
}

impl<F: Float> HueIfceTmp<F> for Hue<F> {
    fn sum_range_for_chroma(&self, chroma: Proportion<F>) -> Option<SumRange<F>> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.sum_range_for_chroma(chroma),
            Self::Secondary(cmy_hue) => cmy_hue.sum_range_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.sum_range_for_chroma(chroma),
        }
    }

    fn max_chroma_for_sum(&self, sum: Sum<F>) -> Option<Chroma<F>> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.max_chroma_for_sum(sum),
            Self::Secondary(cmy_hue) => cmy_hue.max_chroma_for_sum(sum),
            Self::Sextant(sextant_hue) => sextant_hue.max_chroma_for_sum(sum),
        }
    }

    fn max_chroma_rgb(&self) -> RGB<F> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.max_chroma_rgb(),
            Self::Secondary(cmy_hue) => cmy_hue.max_chroma_rgb(),
            Self::Sextant(sextant_hue) => sextant_hue.max_chroma_rgb(),
        }
    }

    fn max_chroma_rgb_for_sum(&self, sum: Sum<F>) -> Option<RGB<F>> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.max_chroma_rgb_for_sum(sum),
            Self::Secondary(cmy_hue) => cmy_hue.max_chroma_rgb_for_sum(sum),
            Self::Sextant(sextant_hue) => sextant_hue.max_chroma_rgb_for_sum(sum),
        }
    }

    fn min_sum_rgb_for_chroma(&self, chroma: Proportion<F>) -> RGB<F> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.min_sum_rgb_for_chroma(chroma),
            Self::Secondary(cmy_hue) => cmy_hue.min_sum_rgb_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.min_sum_rgb_for_chroma(chroma),
        }
    }

    fn max_sum_rgb_for_chroma(&self, chroma: Proportion<F>) -> RGB<F> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.max_sum_rgb_for_chroma(chroma),
            Self::Secondary(cmy_hue) => cmy_hue.max_sum_rgb_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.max_sum_rgb_for_chroma(chroma),
        }
    }

    fn rgb_for_sum_and_chroma(&self, sum: Sum<F>, chroma: Proportion<F>) -> Option<RGB<F>> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.rgb_for_sum_and_chroma(sum, chroma),
            Self::Secondary(cmy_hue) => cmy_hue.rgb_for_sum_and_chroma(sum, chroma),
            Self::Sextant(sextant_hue) => sextant_hue.rgb_for_sum_and_chroma(sum, chroma),
        }
    }
}

impl<F: Float> Hue<F> {
    pub fn ord_index(&self) -> u8 {
        0
    }
}

impl<F: Float> FloatApproxEq<F> for Hue<F> {
    fn approx_eq(&self, other: &Self, max_diff: Option<F>) -> bool {
        match self {
            Self::Primary(rgb_hue) => match other {
                Self::Primary(other_rgb_hue) => rgb_hue == other_rgb_hue,
                _ => false,
            },
            Self::Secondary(cmy_hue) => match other {
                Self::Secondary(other_cmy_hue) => cmy_hue == other_cmy_hue,
                _ => false,
            },
            Self::Sextant(sextant_hue) => match other {
                Self::Sextant(other_sextant_hue) => {
                    sextant_hue.approx_eq(other_sextant_hue, max_diff)
                }
                _ => false,
            },
        }
    }
}

impl<F: Float> PartialOrd for Hue<F> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.ord_index().partial_cmp(&other.ord_index())
    }
}

impl<F: Float> Ord for Hue<F> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[cfg(test)]
mod hue_ng_tests {
    use super::*;
    use num_traits_plus::{assert_approx_eq, float_plus::FloatApproxEq};

    use crate::{proportion::*, rgb::RGB};

    const NON_ZERO_CHROMAS: [Proportion<f64>; 7] = [
        Proportion::<f64>(0.01),
        Proportion::<f64>(0.025),
        Proportion::<f64>(0.5),
        Proportion::<f64>(0.75),
        Proportion::<f64>(0.9),
        Proportion::<f64>(0.99),
        Proportion::ONE,
    ];
    const VALID_OTHER_SUMS: [Sum<f64>; 20] = [
        Sum::<f64>(0.01),
        Sum::<f64>(0.025),
        Sum::<f64>(0.5),
        Sum::<f64>(0.75),
        Sum::<f64>(0.9),
        Sum::<f64>(0.99999),
        Sum::<f64>(1.0),
        Sum::<f64>(1.000000001),
        Sum::<f64>(1.025),
        Sum::<f64>(1.5),
        Sum::<f64>(1.75),
        Sum::<f64>(1.9),
        Sum::<f64>(1.99999),
        Sum::<f64>(2.0),
        Sum::<f64>(2.000000001),
        Sum::<f64>(2.025),
        Sum::<f64>(2.5),
        Sum::<f64>(2.75),
        Sum::<f64>(2.9),
        Sum::<f64>(2.99),
    ];
    // "second" should never be 0.0 or 1.0
    const SECOND_VALUES: [Proportion<f64>; 11] = [
        Proportion::<f64>(0.001),
        Proportion::<f64>(0.1),
        Proportion::<f64>(0.2),
        Proportion::<f64>(0.3),
        Proportion::<f64>(0.4),
        Proportion::<f64>(0.5),
        Proportion::<f64>(0.6),
        Proportion::<f64>(0.7),
        Proportion::<f64>(0.8),
        Proportion::<f64>(0.9),
        Proportion::<f64>(0.99),
    ];

    impl RGBHue {
        fn indices(&self) -> (CCI, CCI, CCI) {
            match self {
                RGBHue::Red => (CCI::Red, CCI::Green, CCI::Blue),
                RGBHue::Green => (CCI::Green, CCI::Red, CCI::Blue),
                RGBHue::Blue => (CCI::Blue, CCI::Red, CCI::Green),
            }
        }
    }

    impl CMYHue {
        fn indices(&self) -> (CCI, CCI, CCI) {
            match self {
                CMYHue::Magenta => (CCI::Red, CCI::Blue, CCI::Green),
                CMYHue::Yellow => (CCI::Red, CCI::Green, CCI::Blue),
                CMYHue::Cyan => (CCI::Green, CCI::Blue, CCI::Red),
            }
        }
    }

    impl Sextant {
        fn indices(&self) -> (CCI, CCI, CCI) {
            match self {
                Sextant::RedYellow => (CCI::Red, CCI::Green, CCI::Blue),
                Sextant::RedMagenta => (CCI::Red, CCI::Blue, CCI::Green),
                Sextant::GreenYellow => (CCI::Green, CCI::Red, CCI::Blue),
                Sextant::GreenCyan => (CCI::Green, CCI::Blue, CCI::Red),
                Sextant::BlueMagenta => (CCI::Blue, CCI::Red, CCI::Green),
                Sextant::BlueCyan => (CCI::Blue, CCI::Green, CCI::Red),
            }
        }
    }

    impl<F: Float> SextantHue<F> {
        fn indices(&self) -> (CCI, CCI, CCI) {
            self.0.indices()
        }
    }

    impl<F: Float> Hue<F> {
        fn indices(&self) -> (CCI, CCI, CCI) {
            match self {
                Self::Primary(rgb_hue) => rgb_hue.indices(),
                Self::Secondary(cmy_hue) => cmy_hue.indices(),
                Self::Sextant(sextant_hue) => sextant_hue.indices(),
            }
        }
    }

    #[test]
    fn hue_from_rgb() {
        for rgb in &[
            RGB::<f64>::BLACK,
            RGB::WHITE,
            RGB::from([
                Proportion::<f64>(0.5),
                Proportion::<f64>(0.5),
                Proportion::<f64>(0.5),
            ]),
        ] {
            assert!(Hue::<f64>::try_from(rgb).is_err());
        }
        for (rgb, hue) in RGB::<f64>::PRIMARIES.iter().zip(Hue::PRIMARIES.iter()) {
            assert_eq!(Hue::<f64>::try_from(rgb), Ok(*hue));
            assert_eq!(
                Hue::<f64>::try_from(&(*rgb * Proportion::<f64>::from(0.5))),
                Ok(*hue)
            );
        }
        for (rgb, hue) in RGB::<f64>::SECONDARIES.iter().zip(Hue::SECONDARIES.iter()) {
            assert_eq!(Hue::<f64>::try_from(rgb), Ok(*hue));
            assert_eq!(
                Hue::<f64>::try_from(&(*rgb * Proportion::<f64>::from(0.5))),
                Ok(*hue)
            );
        }
        for (array, sextant, second) in &[
            (
                [Proportion::ONE, Proportion::<f64>(0.5), Proportion::ZERO],
                Sextant::RedYellow,
                Proportion::<f64>::from(0.5),
            ),
            (
                [
                    Proportion::ZERO,
                    Proportion::<f64>(0.25),
                    Proportion::<f64>(0.5),
                ],
                Sextant::BlueCyan,
                Proportion::<f64>::from(0.5),
            ),
            (
                [
                    Proportion::<f64>(0.2),
                    Proportion::ZERO,
                    Proportion::<f64>(0.4),
                ],
                Sextant::BlueMagenta,
                Proportion::<f64>::from(0.5),
            ),
            (
                [Proportion::<f64>(0.5), Proportion::ZERO, Proportion::ONE],
                Sextant::BlueMagenta,
                Proportion::<f64>::from(0.5),
            ),
            (
                [Proportion::ONE, Proportion::ZERO, Proportion::<f64>(0.5)],
                Sextant::RedMagenta,
                Proportion::<f64>::from(0.5),
            ),
            (
                [Proportion::<f64>(0.5), Proportion::ONE, Proportion::ZERO],
                Sextant::GreenYellow,
                Proportion::<f64>::from(0.5),
            ),
            (
                [Proportion::ZERO, Proportion::ONE, Proportion::<f64>(0.5)],
                Sextant::GreenCyan,
                Proportion::<f64>::from(0.5),
            ),
        ] {
            let rgb = RGB::<f64>::from([
                Proportion::<f64>::from(array[0]),
                Proportion::<f64>::from(array[1]),
                Proportion::<f64>::from(array[2]),
            ]);
            let hue = Hue::Sextant(SextantHue(*sextant, *second));
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
            (
                [Proportion::ONE, Proportion::<f64>(0.5), Proportion::ZERO],
                Sextant::RedYellow,
                Proportion::<f64>(0.5),
            ),
            (
                [Proportion::ZERO, Proportion::<f64>(0.5), Proportion::ONE],
                Sextant::BlueCyan,
                Proportion::<f64>(0.5),
            ),
            (
                [Proportion::<f64>(0.5), Proportion::ZERO, Proportion::ONE],
                Sextant::BlueMagenta,
                Proportion::<f64>(0.5),
            ),
            (
                [Proportion::ONE, Proportion::ZERO, Proportion::<f64>(0.5)],
                Sextant::RedMagenta,
                Proportion::<f64>(0.5),
            ),
            (
                [Proportion::<f64>(0.5), Proportion::ONE, Proportion::ZERO],
                Sextant::GreenYellow,
                Proportion::<f64>(0.5),
            ),
            (
                [Proportion::ZERO, Proportion::ONE, Proportion::<f64>(0.5)],
                Sextant::GreenCyan,
                Proportion::<f64>(0.5),
            ),
        ] {
            let rgb = RGB::<f64>::from(*array);
            let hue = Hue::Sextant(SextantHue(*sextant, *second));
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
            (
                Sextant::RedYellow,
                Proportion::<f64>(0.5),
                Degrees::<f64>::DEG_30,
            ),
            (
                Sextant::BlueCyan,
                Proportion::<f64>(0.5),
                Degrees::<f64>::NEG_DEG_150,
            ),
            (
                Sextant::BlueMagenta,
                Proportion::<f64>(0.5),
                Degrees::<f64>::NEG_DEG_90,
            ),
            (
                Sextant::RedMagenta,
                Proportion::<f64>(0.5),
                Degrees::<f64>::NEG_DEG_30,
            ),
            (
                Sextant::GreenYellow,
                Proportion::<f64>(0.5),
                Degrees::<f64>::DEG_90,
            ),
            (
                Sextant::GreenCyan,
                Proportion::<f64>(0.5),
                Degrees::<f64>::DEG_150,
            ),
            //(Sextant::RedYellow, Proportion::<f64>(0.25), Degrees::<f64>::from(1Proportion::<f64>(5.0))),
        ] {
            let hue = Hue::Sextant(SextantHue(*sextant, *second));
            assert_approx_eq!(hue.hue_angle(), *angle, 0.0000001);
        }
    }

    // TODO: this test needs to be improved
    #[test]
    fn max_chroma_and_sum_ranges() {
        for hue in &Hue::<f64>::PRIMARIES {
            assert!(hue.sum_range_for_chroma(Proportion::<f64>(0.0)).is_none());
            assert_eq!(
                hue.sum_range_for_chroma(Proportion::<f64>(1.0)),
                Some(SumRange((
                    Sum::<f64>(1.0),
                    Sum::<f64>(1.0),
                    Sum::<f64>(1.0)
                )))
            );
            for chroma in NON_ZERO_CHROMAS.iter() {
                let range = hue.sum_range_for_chroma(*chroma).unwrap();
                let max_chroma = hue.max_chroma_for_sum(range.shade_min()).unwrap();
                assert_approx_eq!(max_chroma.proportion(), range.shade_min().into());
                // let max_chroma = hue.max_chroma_for_sum(range.tint_max()).unwrap();
                // assert_approx_eq!(
                //     max_chroma.proportion(),
                //     range.tint_max().into(),
                //     0.000_000_000_000_001
                // );
            }
        }
        for hue in &Hue::<f64>::SECONDARIES {
            assert!(hue.sum_range_for_chroma(Proportion::<f64>(0.0)).is_none());
            assert_eq!(
                hue.sum_range_for_chroma(Proportion::<f64>(1.0)),
                Some(SumRange((Sum::TWO, Sum::TWO, Sum::TWO)))
            );
            for chroma in NON_ZERO_CHROMAS.iter() {
                let range = hue.sum_range_for_chroma(*chroma).unwrap();
                let max_chroma = hue.max_chroma_for_sum(range.shade_min()).unwrap();
                assert_approx_eq!(
                    max_chroma.proportion(),
                    (range.shade_min() / Sum::<f64>(2.0)).into()
                );
                // let max_chroma = hue.max_chroma_for_sum(range.tint_max()).unwrap();
                // assert_approx_eq!(
                //     max_chroma.proportion(),
                //     (range.tint_max() / Sum::<f64>(2.0)).into(),
                //     0.000000000000001
                // );
            }
        }
        use Sextant::*;
        for sextant in &[
            RedYellow,
            RedMagenta,
            GreenCyan,
            GreenYellow,
            BlueCyan,
            BlueMagenta,
        ] {
            for other in SECOND_VALUES.iter() {
                let hue = Hue::<f64>::Sextant(SextantHue(*sextant, *other));
                assert!(hue.sum_range_for_chroma(Proportion::<f64>(0.0)).is_none());
                assert_eq!(
                    hue.sum_range_for_chroma(Proportion::<f64>(1.0)),
                    Some(SumRange((
                        Sum::<f64>(1.0) + *other,
                        Sum::<f64>(1.0) + *other,
                        Sum::<f64>(1.0) + *other
                    )))
                );
            }
        }
    }

    #[test]
    fn primary_max_chroma_rgbs() {
        for (hue, expected_rgb) in Hue::<f64>::PRIMARIES
            .iter()
            .zip(RGB::<f64>::PRIMARIES.iter())
        {
            assert_eq!(
                hue.max_chroma_rgb_for_sum(Sum::<f64>(1.0)).unwrap(),
                *expected_rgb
            );
            assert!(hue.max_chroma_rgb_for_sum(Sum::<f64>(0.0)).is_none());
            assert!(hue.max_chroma_rgb_for_sum(Sum::<f64>(3.0)).is_none());
            for sum in [
                Sum::<f64>(0.0001),
                Sum::<f64>(0.25),
                Sum::<f64>(0.5),
                Sum::<f64>(0.75),
                Sum::<f64>(0.9999),
            ]
            .iter()
            {
                let mut array = [Proportion::ZERO, Proportion::ZERO, Proportion::ZERO];
                array[hue.indices().0 as usize] = (*sum).into();
                let expected: RGB<f64> = array.into();
                assert_eq!(hue.max_chroma_rgb_for_sum(*sum).unwrap(), expected);
            }
            for sum in [
                Sum::<f64>(2.0001),
                Sum::<f64>(2.25),
                Sum::<f64>(2.5),
                Sum::<f64>(2.75),
                Sum::<f64>(2.9999),
            ]
            .iter()
            {
                let mut array = [Proportion::ONE, Proportion::ONE, Proportion::ONE];
                array[hue.indices().1 as usize] =
                    ((*sum - Sum::<f64>(1.0)) / Sum::<f64>(2.0)).into();
                array[hue.indices().2 as usize] =
                    ((*sum - Sum::<f64>(1.0)) / Sum::<f64>(2.0)).into();
                let expected: RGB<f64> = array.into();
                assert_eq!(hue.max_chroma_rgb_for_sum(*sum).unwrap(), expected);
            }
        }
    }

    #[test]
    fn secondary_max_chroma_rgbs() {
        for (hue, expected_rgb) in Hue::<f64>::SECONDARIES
            .iter()
            .zip(RGB::<f64>::SECONDARIES.iter())
        {
            assert_eq!(
                hue.max_chroma_rgb_for_sum(Sum::<f64>(2.0)).unwrap(),
                *expected_rgb
            );
            assert!(hue.max_chroma_rgb_for_sum(Sum::<f64>(0.0)).is_none());
            assert!(hue.max_chroma_rgb_for_sum(Sum::<f64>(3.0)).is_none());
            for sum in [
                Sum::<f64>(0.0001),
                Sum::<f64>(0.25),
                Sum::<f64>(0.5),
                Sum::<f64>(0.75),
                Sum::<f64>(1.0),
                Sum::<f64>(1.5),
                Sum::<f64>(1.9999),
            ]
            .iter()
            {
                let mut array = [Proportion::ZERO, Proportion::ZERO, Proportion::ZERO];
                array[hue.indices().0 as usize] = (*sum / Sum::<f64>(2.0)).into();
                array[hue.indices().1 as usize] = (*sum / Sum::<f64>(2.0)).into();
                let expected: RGB<f64> = array.into();
                assert_eq!(hue.max_chroma_rgb_for_sum(*sum).unwrap(), expected);
            }
            for sum in [
                Sum::<f64>(2.0001),
                Sum::<f64>(2.25),
                Sum::<f64>(2.5),
                Sum::<f64>(2.75),
                Sum::<f64>(2.9999),
            ]
            .iter()
            {
                let mut array = [Proportion::ONE, Proportion::ONE, Proportion::ONE];
                array[hue.indices().2 as usize] = (*sum - Sum::<f64>(2.0)).into();
                let expected: RGB<f64> = array.into();
                assert_eq!(hue.max_chroma_rgb_for_sum(*sum).unwrap(), expected);
            }
        }
    }

    #[test]
    fn other_max_chroma_rgbs() {
        use Sextant::*;
        for sextant in &[
            RedYellow,
            RedMagenta,
            GreenCyan,
            GreenYellow,
            BlueCyan,
            BlueMagenta,
        ] {
            for second in SECOND_VALUES.iter() {
                let sextant_hue = SextantHue(*sextant, *second);
                let hue = Hue::<f64>::Sextant(sextant_hue);
                assert!(hue.max_chroma_rgb_for_sum(Sum::<f64>(0.0)).is_none());
                assert!(hue.max_chroma_rgb_for_sum(Sum::<f64>(3.0)).is_none());
                println!("hue: {:?} MAX_CHROMA_RGB: {:?}", hue, hue.max_chroma_rgb());
                for sum in VALID_OTHER_SUMS.iter() {
                    let rgb = hue.max_chroma_rgb_for_sum(*sum).unwrap();
                    //assert_approx_eq!(rgb.sum(), *sum);
                    if *sum < Sum::THREE - *second {
                        if let Ok(Hue::<f64>::Sextant(sextant_hue_out)) = Hue::<f64>::try_from(&rgb)
                        {
                            assert_eq!(sextant_hue.0, sextant_hue_out.0);
                        //assert_approx_eq!(sextant_hue.1, sextant_hue_out.1, 0.000000000001);
                        } else {
                            panic!("\"Sextant\"  Hue variant expected");
                        }
                    } else {
                        // sum is too big for this hue so drifting towards nearest secondary
                        use CMYHue::*;
                        use Sextant::*;
                        let hue_out = Hue::<f64>::try_from(&rgb).unwrap();
                        match sextant {
                            RedYellow | GreenYellow => assert_eq!(hue_out, Hue::Secondary(Yellow)),
                            RedMagenta | BlueMagenta => {
                                assert_eq!(hue_out, Hue::Secondary(Magenta))
                            }
                            GreenCyan | BlueCyan => assert_eq!(hue_out, Hue::Secondary(Cyan)),
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn min_max_sum_rgb_for_chroma() {
        for (hue, expected_rgb) in Hue::<f64>::PRIMARIES
            .iter()
            .zip(RGB::<f64>::PRIMARIES.iter())
        {
            println!("{:?} : {:?}", hue, expected_rgb);
            assert_eq!(
                hue.min_sum_rgb_for_chroma(Proportion::<f64>(1.0)),
                *expected_rgb
            );
            assert_eq!(
                hue.max_sum_rgb_for_chroma(Proportion::<f64>(1.0)),
                *expected_rgb
            );
            // let shade = hue.min_sum_rgb_for_chroma(Proportion::<f64>(0.5));
            // let tint = hue.max_sum_rgb_for_chroma(Proportion::<f64>(0.5));
            //assert!(shade.value() < tint.value());
            // assert_approx_eq!(shade.chroma(), Proportion::<f64>(0.5), 0.00000000001);
            // assert_approx_eq!(tint.chroma(), Proportion::<f64>(0.5), 0.00000000001);
            // assert_approx_eq!(shade.max_chroma_rgb(), tint.max_chroma_rgb(), 0.0000001);
        }
        for (hue, expected_rgb) in Hue::<f64>::SECONDARIES
            .iter()
            .zip(RGB::<f64>::SECONDARIES.iter())
        {
            println!("{:?} : {:?}", hue, expected_rgb);
            assert_eq!(
                hue.min_sum_rgb_for_chroma(Proportion::<f64>(1.0)),
                *expected_rgb
            );
            assert_eq!(
                hue.max_sum_rgb_for_chroma(Proportion::<f64>(1.0)),
                *expected_rgb
            );
            // let shade = hue.min_sum_rgb_for_chroma(Proportion::<f64>(0.5));
            // let tint = hue.max_sum_rgb_for_chroma(Proportion::<f64>(0.5));
            // assert!(shade.value() < tint.value());
            // assert_approx_eq!(shade.chroma(), Proportion::<f64>(0.5), 0.00000000001);
            // assert_approx_eq!(tint.chroma(), Proportion::<f64>(0.5), 0.00000000001);
            // assert_approx_eq!(shade.max_chroma_rgb(), tint.max_chroma_rgb(), 0.0000001);
        }
        use Sextant::*;
        for sextant in &[
            RedYellow,
            RedMagenta,
            GreenCyan,
            GreenYellow,
            BlueCyan,
            BlueMagenta,
        ] {
            for second in SECOND_VALUES.iter() {
                let hue = Hue::<f64>::Sextant(SextantHue(*sextant, *second));
                assert_eq!(
                    hue.min_sum_rgb_for_chroma(Proportion::<f64>(0.0)),
                    RGB::BLACK
                );
                assert_eq!(
                    hue.max_sum_rgb_for_chroma(Proportion::<f64>(0.0)),
                    RGB::WHITE
                );
                // for chroma in NON_ZERO_CHROMAS.iter() {
                //     let shade = hue.min_sum_rgb_for_chroma(*chroma);
                //     let tint = hue.max_sum_rgb_for_chroma(*chroma);
                //     // assert!(shade.sum() <= tint.sum());
                //     // assert_approx_eq!(shade.chroma(), *chroma, 0.00000000001);
                //     // assert_approx_eq!(tint.chroma(), *chroma, 0.00000000001);
                //     // assert_approx_eq!(shade.max_chroma_rgb(), tint.max_chroma_rgb(), 0.000_001);
                // }
            }
        }
    }

    #[test]
    fn primary_rgb_for_sum_and_chroma() {
        for hue in &Hue::<f64>::PRIMARIES {
            assert!(hue
                .rgb_for_sum_and_chroma(Sum::ZERO, Proportion::<f64>(1.0))
                .is_none());
            assert!(hue
                .rgb_for_sum_and_chroma(Sum::THREE, Proportion::<f64>(1.0))
                .is_none());
            assert!(hue
                .rgb_for_sum_and_chroma(Sum::ZERO, Proportion::<f64>(0.0))
                .is_none());
            assert!(hue
                .rgb_for_sum_and_chroma(Sum::THREE, Proportion::<f64>(0.0))
                .is_none());
            for chroma in &NON_ZERO_CHROMAS {
                for sum in &VALID_OTHER_SUMS {
                    if let Some(rgb) = hue.rgb_for_sum_and_chroma(*sum, *chroma) {
                        //assert_approx_eq!(rgb.sum(), *sum, 0.000_000_000_1);
                        //assert_approx_eq!(rgb.chroma(), *chroma, 0.000_000_000_1);
                        assert_approx_eq!(Hue::<f64>::try_from(&rgb).unwrap(), hue);
                    } else {
                        let range = hue.sum_range_for_chroma(*chroma).unwrap();
                        println!("{:?}, {:?}, {:?} : {:?}", *hue, sum, chroma, range);
                        assert!(range.compare_sum(*sum).is_failure());
                    }
                }
            }
        }
    }

    #[test]
    fn secondary_rgb_for_sum_and_chroma() {
        for hue in &Hue::<f64>::SECONDARIES {
            assert!(hue
                .rgb_for_sum_and_chroma(Sum::ZERO, Proportion::<f64>(1.0))
                .is_none());
            assert!(hue
                .rgb_for_sum_and_chroma(Sum::THREE, Proportion::<f64>(1.0))
                .is_none());
            assert!(hue
                .rgb_for_sum_and_chroma(Sum::ZERO, Proportion::<f64>(0.0))
                .is_none());
            assert!(hue
                .rgb_for_sum_and_chroma(Sum::THREE, Proportion::<f64>(0.0))
                .is_none());
            for chroma in &NON_ZERO_CHROMAS {
                for sum in &VALID_OTHER_SUMS {
                    if let Some(rgb) = hue.rgb_for_sum_and_chroma(*sum, *chroma) {
                        // assert_approx_eq!(rgb.sum(), *sum, 0.000_000_1);
                        // assert_approx_eq!(rgb.chroma(), *chroma, 0.000_000_1);
                        assert_approx_eq!(Hue::<f64>::try_from(&rgb).unwrap(), hue);
                    } else {
                        let range = hue.sum_range_for_chroma(*chroma).unwrap();
                        assert!(range.compare_sum(*sum).is_failure());
                    }
                }
            }
        }
    }

    // #[test]
    // fn general_rgb_for_sum_and_chroma() {
    //     use Sextant::*;
    //     for sextant in &[
    //         RedYellow,
    //         RedMagenta,
    //         GreenCyan,
    //         GreenYellow,
    //         BlueCyan,
    //         BlueMagenta,
    //     ] {
    //         for second in SECOND_VALUES.iter() {
    //             let sextant_hue = SextantHue(*sextant, *second);
    //             let hue = Hue::<f64>::Sextant(sextant_hue);
    //             assert!(hue
    //                 .rgb_for_sum_and_chroma(Sum::ZERO, Proportion::<f64>(1.0))
    //                 .is_none());
    //             assert!(hue
    //                 .rgb_for_sum_and_chroma(Sum::THREE, Proportion::<f64>(1.0))
    //                 .is_none());
    //             assert!(hue
    //                 .rgb_for_sum_and_chroma(Sum::ZERO, Proportion::<f64>(0.0))
    //                 .is_none());
    //             assert!(hue
    //                 .rgb_for_sum_and_chroma(Sum::THREE, Proportion::<f64>(0.0))
    //                 .is_none());
    //             for chroma in &NON_ZERO_CHROMAS {
    //                 let sum_range = hue.sum_range_for_chroma(*chroma).unwrap();
    //                 for sum in &VALID_OTHER_SUMS {
    //                     println!(
    //                         "{:?}, {:?}, {:?} :: {:?}",
    //                         hue,
    //                         sum,
    //                         chroma,
    //                         hue.sum_range_for_chroma(*chroma)
    //                     );
    //                     if let Some(rgb) = hue.rgb_for_sum_and_chroma(*sum, *chroma) {
    //                         use SumOrdering::*;
    //                         match sum_range.compare_sum(*sum) {
    //                             Shade(_, _) => {
    //                                 // assert_approx_eq!(rgb.sum(), *sum, 0.000_000_000_1);
    //                                 // assert_approx_eq!(rgb.chroma(), *chroma, 0.000_000_1);
    //                                 // TODO: examine hue drift problem
    //                                 // assert_approx_eq!(
    //                                 //     Hue::<f64>::try_from(&rgb).unwrap(),
    //                                 //     hue,
    //                                 //     0.000_000_000_1
    //                                 // );
    //                             }
    //                             Tint(_, _) => {
    //                                 // assert_approx_eq!(rgb.sum(), *sum, 0.000_000_000_1);
    //                                 // TODO: try harder for creating tints
    //                                 //    assert_approx_eq!(rgb.chroma(), *chroma, 0.000_000_1);
    //                                 // assert_approx_eq!(
    //                                 //     Hue::<f64>::try_from(&rgb).unwrap(),
    //                                 //     hue,
    //                                 //     0.000_000_000_1
    //                                 // );
    //                             }
    //                             _ => (),
    //                         }
    //                     } else {
    //                         let range = hue.sum_range_for_chroma(*chroma).unwrap();
    //                         assert!(range.compare_sum(*sum).is_failure());
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }
}
