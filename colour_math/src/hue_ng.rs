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
use num_traits_plus::float_plus::FloatApproxEq;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord)]
pub struct SumRange<F: ColourComponent>((F, F, F));

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord)]
pub enum SumRangeComparisonResult<F: ColourComponent> {
    TooSmall,
    Shade(F, F),
    Tint(F, F),
    TooBig,
}

impl<F: ColourComponent> SumRange<F> {
    pub fn compare_sum(&self, sum: F) -> SumRangeComparisonResult<F> {
        if sum <= self.0 .0 {
            SumRangeComparisonResult::TooSmall
        } else if sum <= self.0 .1 {
            SumRangeComparisonResult::Shade(self.0 .0, self.0 .1)
        } else if sum < self.0 .2 {
            SumRangeComparisonResult::Tint(self.0 .1, self.0 .2)
        } else {
            SumRangeComparisonResult::TooBig
        }
    }

    pub fn shade_min(&self) -> F {
        self.0 .0
    }

    pub fn shade_max(&self) -> F {
        self.0 .1
    }

    pub fn tint_min(&self) -> F {
        self.0 .1
    }

    pub fn tint_max(&self) -> F {
        self.0 .2
    }
}

pub trait HueIfceTmp<F: ColourComponent> {
    fn hue_angle(&self) -> Degrees<F>;
    fn chroma_correction(&self) -> F;
    fn sum_range_for_chroma(&self, chroma: F) -> SumRange<F>;
    fn max_chroma_for_sum(&self, sum: F) -> F;

    fn max_chroma_rgb(&self) -> RGB<F>;
    fn max_chroma_rgb_for_sum(&self, sum: F) -> RGB<F>;
    fn min_sum_rgb_for_chroma(&self, chroma: F) -> RGB<F>;
    fn max_sum_rgb_for_chroma(&self, chroma: F) -> RGB<F>;
    fn rgb_for_sum_and_chroma(&self, sum: F, chroma: F) -> Option<RGB<F>>;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord)]
pub enum RGBHue {
    Red = 5,
    Green = 9,
    Blue = 1,
}

impl RGBHue {
    fn make_rgb<F: ColourComponent>(&self, components: (F, F)) -> RGB<F> {
        use RGBHue::*;
        match self {
            Red => [components.0, components.1, components.1].into(),
            Green => [components.1, components.0, components.1].into(),
            Blue => [components.1, components.1, components.0].into(),
        }
    }
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

    fn sum_range_for_chroma(&self, chroma: F) -> SumRange<F> {
        debug_assert!(chroma.is_proportion(), "chroma: {:?}", chroma);
        SumRange((chroma, (F::THREE - F::TWO * chroma).min(F::THREE), F::ONE))
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

    fn max_chroma_rgb_for_sum(&self, sum: F) -> RGB<F> {
        debug_assert!(sum >= F::ZERO && sum <= F::THREE, "sum: {:?}", sum);
        if sum == F::ZERO {
            RGB::BLACK
        } else if sum == F::THREE {
            RGB::WHITE
        } else {
            if sum <= F::ONE {
                self.make_rgb((sum, F::ZERO))
            } else {
                self.make_rgb((F::ONE, ((sum - F::ONE) / F::TWO).min(F::ONE)))
            }
        }
    }

    fn min_sum_rgb_for_chroma(&self, chroma: F) -> RGB<F> {
        debug_assert!(chroma.is_proportion(), "chroma: {:?}", chroma);
        if chroma == F::ZERO {
            RGB::BLACK
        } else if chroma == F::ONE {
            self.max_chroma_rgb()
        } else {
            self.make_rgb((chroma, F::ZERO))
        }
    }

    fn max_sum_rgb_for_chroma(&self, chroma: F) -> RGB<F> {
        debug_assert!(chroma.is_proportion(), "chroma: {:?}", chroma);
        if chroma == F::ZERO {
            RGB::WHITE
        } else if chroma == F::ONE {
            self.max_chroma_rgb()
        } else {
            self.make_rgb((F::ONE, F::ONE - chroma))
        }
    }

    fn rgb_for_sum_and_chroma(&self, sum: F, chroma: F) -> Option<RGB<F>> {
        debug_assert!(sum >= F::ZERO && sum <= F::THREE, "sum: {:?}", sum);
        debug_assert!(chroma.is_proportion());
        if chroma > F::ZERO && sum > chroma && sum < F::THREE - chroma * F::TWO {
            let other = (sum - chroma) / F::THREE;
            Some(self.make_rgb(((chroma + other), other)))
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
    fn make_rgb<F: ColourComponent>(&self, components: (F, F)) -> RGB<F> {
        use CMYHue::*;
        match self {
            Cyan => [components.1, components.0, components.0].into(),
            Magenta => [components.0, components.1, components.0].into(),
            Yellow => [components.0, components.0, components.1].into(),
        }
    }
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

    fn sum_range_for_chroma(&self, chroma: F) -> SumRange<F> {
        debug_assert!(chroma.is_proportion(), "chroma: {:?}", chroma);
        SumRange((chroma * F::TWO, F::THREE - chroma, F::TWO))
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

    fn max_chroma_rgb_for_sum(&self, sum: F) -> RGB<F> {
        debug_assert!(sum >= F::ZERO && sum <= F::THREE, "sum: {:?}", sum);
        if sum == F::ZERO {
            RGB::BLACK
        } else if sum == F::THREE {
            RGB::WHITE
        } else {
            if sum <= F::TWO {
                self.make_rgb(((sum / F::TWO).min(F::ONE), F::ZERO))
            } else {
                self.make_rgb((F::ONE, (sum - F::TWO).max(F::ZERO).min(F::ONE)))
            }
        }
    }

    fn min_sum_rgb_for_chroma(&self, chroma: F) -> RGB<F> {
        debug_assert!(chroma.is_proportion(), "chroma: {:?}", chroma);
        if chroma == F::ZERO {
            RGB::BLACK
        } else if chroma == F::ONE {
            self.max_chroma_rgb()
        } else {
            self.make_rgb((chroma, F::ZERO))
        }
    }

    fn max_sum_rgb_for_chroma(&self, chroma: F) -> RGB<F> {
        debug_assert!(chroma.is_proportion(), "chroma: {:?}", chroma);
        if chroma == F::ZERO {
            RGB::WHITE
        } else if chroma == F::ONE {
            self.max_chroma_rgb()
        } else {
            self.make_rgb((F::ONE, F::ONE - chroma))
        }
    }

    fn rgb_for_sum_and_chroma(&self, sum: F, chroma: F) -> Option<RGB<F>> {
        debug_assert!(sum >= F::ZERO && sum <= F::THREE, "sum: {:?}", sum);
        debug_assert!(chroma.is_proportion());
        if chroma > F::ZERO && sum >= chroma * F::TWO && sum <= F::THREE - chroma {
            Some(self.make_rgb((
                (sum + chroma) / F::THREE,
                (sum - chroma * F::TWO) / F::THREE,
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
pub struct SextantHue<F: ColourComponent>(Sextant, F);

impl<F: ColourComponent> Eq for SextantHue<F> {}

impl<F: ColourComponent> SextantHue<F> {
    fn make_rgb(&self, components: (F, F, F)) -> RGB<F> {
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

    fn chroma_xy(&self, chroma: F) -> (F, F) {
        (chroma * (F::ONE - self.1), chroma * self.1 * F::SIN_120)
    }
}

impl<F: ColourComponent> FloatApproxEq<F> for SextantHue<F> {
    fn approx_eq(&self, other: &Self, max_diff: Option<F>) -> bool {
        if self.0 == other.0 {
            self.1.approx_eq(&other.1, max_diff)
        } else {
            false
        }
    }
}

impl<F: ColourComponent> From<(Sextant, &RGB<F>)> for SextantHue<F> {
    fn from(arg: (Sextant, &RGB<F>)) -> Self {
        use Sextant::*;
        use CCI::*;
        match arg.0 {
            RedMagenta => Self(arg.0, (arg.1[Blue] - arg.1[Green]) / arg.1[Red]),
            RedYellow => Self(arg.0, (arg.1[Green] - arg.1[Blue]) / arg.1[Red]),
            GreenYellow => Self(arg.0, (arg.1[Red] - arg.1[Blue]) / arg.1[Green]),
            GreenCyan => Self(arg.0, (arg.1[Blue] - arg.1[Red]) / arg.1[Green]),
            BlueCyan => Self(arg.0, (arg.1[Green] - arg.1[Red]) / arg.1[Blue]),
            BlueMagenta => Self(arg.0, (arg.1[Red] - arg.1[Green]) / arg.1[Blue]),
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
        (F::ONE + self.1 * self.1 - F::TWO * self.1)
            .sqrt()
            .min(F::ONE)
            .recip()
    }

    fn sum_range_for_chroma(&self, chroma: F) -> SumRange<F> {
        debug_assert!(chroma.is_proportion(), "chroma: {:?}", chroma);
        let temp = self.1 * chroma;
        SumRange((
            (chroma + temp).min(F::THREE),
            (F::THREE + temp - F::TWO * chroma).min(F::THREE),
            temp,
        ))
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
        self.make_rgb((F::ONE, self.1, F::ZERO))
    }

    fn max_chroma_rgb_for_sum(&self, sum: F) -> RGB<F> {
        debug_assert!(sum >= F::ZERO && sum <= F::THREE, "sum: {:?}", sum);
        // TODO: make hue drift an error
        if sum == F::ZERO {
            RGB::BLACK
        } else if sum == F::THREE {
            RGB::WHITE
        } else {
            let max_chroma_sum = self.1 + F::ONE;
            if sum == max_chroma_sum {
                self.max_chroma_rgb()
            } else {
                let components = if sum < max_chroma_sum {
                    let first = (sum / max_chroma_sum).min(F::ONE);
                    (first, first * self.1, F::ZERO)
                } else {
                    let temp = sum - F::ONE;
                    let second = ((temp + self.1) / F::TWO).min(F::ONE);
                    (F::ONE, second, (temp - second).max(F::ZERO))
                };
                self.make_rgb(components)
            }
        }
    }

    fn min_sum_rgb_for_chroma(&self, chroma: F) -> RGB<F> {
        debug_assert!(chroma.is_proportion(), "chroma: {:?}", chroma);
        if chroma == F::ZERO {
            RGB::BLACK
        } else if chroma == F::ONE {
            self.max_chroma_rgb()
        } else {
            self.make_rgb((chroma, self.1 * chroma, F::ZERO))
        }
    }

    fn max_sum_rgb_for_chroma(&self, chroma: F) -> RGB<F> {
        debug_assert!(chroma.is_proportion(), "chroma: {:?}", chroma);
        if chroma == F::ZERO {
            RGB::WHITE
        } else if chroma == F::ONE {
            self.max_chroma_rgb()
        } else {
            let third = F::ONE - chroma;
            self.make_rgb((F::ONE, chroma * self.1 + third, third))
        }
    }

    fn rgb_for_sum_and_chroma(&self, sum: F, chroma: F) -> Option<RGB<F>> {
        debug_assert!(sum >= F::ZERO && sum <= F::THREE, "sum: {:?}", sum);
        debug_assert!(chroma.is_proportion());
        if chroma > F::ZERO && chroma < F::ONE {
            let (chroma_x, chroma_y) = self.chroma_xy(chroma);
            println!(
                "SUM: {:?} <= {:?} <= {:?}",
                chroma * (self.1 + F::ONE),
                sum,
                F::THREE - F::TWO * chroma_x
            );
            if sum > chroma * (self.1 + F::ONE) && sum < F::THREE - F::TWO * chroma_x {
                let max_chroma_sum = F::ONE + self.1;
                if sum <= max_chroma_sum {
                    let first = (sum / max_chroma_sum).min(F::ONE);
                    println!(
                        "DARK: {:?} {:?} ({:?}, {:?}, {:?})",
                        sum,
                        chroma,
                        first,
                        first * self.1,
                        F::ZERO
                    );
                    Some(self.make_rgb((first, first * self.1, F::ZERO)))
                } else {
                    let first = (sum + chroma_x * F::TWO) / F::THREE;
                    let delta = chroma_y / F::SIN_120;
                    let remainder = sum - first;
                    println!(
                        "LIGHT: {:?} {:?} ({:?}, {:?}, {:?}): delta = {:?} : x = {:?}, y = {:?}",
                        sum,
                        chroma,
                        first,
                        (remainder + delta) / F::TWO,
                        (remainder - delta) / F::TWO,
                        delta,
                        chroma_x,
                        chroma_y
                    );
                    Some(self.make_rgb((
                        first,
                        (remainder + delta) / F::TWO,
                        (remainder - delta) / F::TWO,
                    )))
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Hue<F: ColourComponent> {
    Primary(RGBHue),
    Secondary(CMYHue),
    Sextant(SextantHue<F>),
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

impl<F: ColourComponent> HueIfceTmp<F> for Hue<F> {
    fn hue_angle(&self) -> Degrees<F> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.hue_angle(),
            Self::Secondary(cmy_hue) => cmy_hue.hue_angle(),
            Self::Sextant(sextant_hue) => sextant_hue.hue_angle(),
        }
    }

    fn chroma_correction(&self) -> F {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.chroma_correction(),
            Self::Secondary(cmy_hue) => cmy_hue.chroma_correction(),
            Self::Sextant(sextant_hue) => sextant_hue.chroma_correction(),
        }
    }

    fn sum_range_for_chroma(&self, chroma: F) -> SumRange<F> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.sum_range_for_chroma(chroma),
            Self::Secondary(cmy_hue) => cmy_hue.sum_range_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.sum_range_for_chroma(chroma),
        }
    }

    fn max_chroma_for_sum(&self, sum: F) -> F {
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

    fn max_chroma_rgb_for_sum(&self, sum: F) -> RGB<F> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.max_chroma_rgb_for_sum(sum),
            Self::Secondary(cmy_hue) => cmy_hue.max_chroma_rgb_for_sum(sum),
            Self::Sextant(sextant_hue) => sextant_hue.max_chroma_rgb_for_sum(sum),
        }
    }

    fn min_sum_rgb_for_chroma(&self, chroma: F) -> RGB<F> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.min_sum_rgb_for_chroma(chroma),
            Self::Secondary(cmy_hue) => cmy_hue.min_sum_rgb_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.min_sum_rgb_for_chroma(chroma),
        }
    }

    fn max_sum_rgb_for_chroma(&self, chroma: F) -> RGB<F> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.max_sum_rgb_for_chroma(chroma),
            Self::Secondary(cmy_hue) => cmy_hue.max_sum_rgb_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.max_sum_rgb_for_chroma(chroma),
        }
    }

    fn rgb_for_sum_and_chroma(&self, sum: F, chroma: F) -> Option<RGB<F>> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.rgb_for_sum_and_chroma(sum, chroma),
            Self::Secondary(cmy_hue) => cmy_hue.rgb_for_sum_and_chroma(sum, chroma),
            Self::Sextant(sextant_hue) => sextant_hue.rgb_for_sum_and_chroma(sum, chroma),
        }
    }
}

impl<F: ColourComponent> Hue<F> {
    pub fn ord_index(&self) -> u8 {
        0
    }
}

impl<F: ColourComponent> FloatApproxEq<F> for Hue<F> {
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

    const NON_ZERO_CHROMAS: [f64; 7] = [0.01, 0.025, 0.5, 0.75, 0.9, 0.99, 1.0];
    const VALID_OTHER_SUMS: [f64; 20] = [
        0.01,
        0.025,
        0.5,
        0.75,
        0.9,
        0.99999,
        1.0,
        1.000000001,
        1.025,
        1.5,
        1.75,
        1.9,
        1.99999,
        2.0,
        2.000000001,
        2.025,
        2.5,
        2.75,
        2.9,
        2.99,
    ];
    // "second" should never be 0.0 or 1.0
    const SECOND_VALUES: [f64; 11] = [0.001, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 0.99];

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

    impl<F: ColourComponent> SextantHue<F> {
        fn indices(&self) -> (CCI, CCI, CCI) {
            self.0.indices()
        }
    }

    impl<F: ColourComponent> Hue<F> {
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
            ([1.0, 0.5, 0.0], Sextant::RedYellow, 0.5),
            ([0.0, 0.5, 1.0], Sextant::BlueCyan, 0.5),
            ([0.5, 0.0, 1.0], Sextant::BlueMagenta, 0.5),
            ([1.0, 0.0, 0.5], Sextant::RedMagenta, 0.5),
            ([0.5, 1.0, 0.0], Sextant::GreenYellow, 0.5),
            ([0.0, 1.0, 0.5], Sextant::GreenCyan, 0.5),
        ] {
            let rgb = RGB::<f64>::from(array);
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
            (Sextant::RedYellow, 0.5, Degrees::<f64>::DEG_30),
            (Sextant::BlueCyan, 0.5, Degrees::<f64>::NEG_DEG_150),
            (Sextant::BlueMagenta, 0.5, Degrees::<f64>::NEG_DEG_90),
            (Sextant::RedMagenta, 0.5, Degrees::<f64>::NEG_DEG_30),
            (Sextant::GreenYellow, 0.5, Degrees::<f64>::DEG_90),
            (Sextant::GreenCyan, 0.5, Degrees::<f64>::DEG_150),
            //(Sextant::RedYellow, 0.25, Degrees::<f64>::from(15.0)),
        ] {
            let hue = Hue::Sextant(SextantHue(*sextant, *second));
            assert_approx_eq!(hue.hue_angle(), *angle, 0.0000001);
        }
    }

    #[test]
    fn max_chroma_and_sum_ranges() {
        for hue in &Hue::<f64>::PRIMARIES {
            assert_eq!(hue.sum_range_for_chroma(0.0), SumRange((0.0, 3.0, 1.0)));
            assert_eq!(hue.sum_range_for_chroma(1.0), SumRange((1.0, 1.0, 1.0)));
            for chroma in NON_ZERO_CHROMAS.iter() {
                let range = hue.sum_range_for_chroma(*chroma);
                let max_chroma = hue.max_chroma_for_sum(range.shade_min());
                assert_approx_eq!(max_chroma, *chroma);
                let max_chroma = hue.max_chroma_for_sum(range.tint_max());
                assert_approx_eq!(max_chroma, *chroma, 0.000000000000001);
            }
        }
        for hue in &Hue::<f64>::SECONDARIES {
            assert_eq!(hue.sum_range_for_chroma(0.0), SumRange((0.0, 3.0, 2.0)));
            assert_eq!(hue.sum_range_for_chroma(1.0), SumRange((2.0, 2.0, 2.0)));
            for chroma in NON_ZERO_CHROMAS.iter() {
                let range = hue.sum_range_for_chroma(*chroma);
                let max_chroma = hue.max_chroma_for_sum(range.shade_min());
                assert_approx_eq!(max_chroma, *chroma);
                let max_chroma = hue.max_chroma_for_sum(range.tint_max());
                assert_approx_eq!(max_chroma, *chroma, 0.000000000000001);
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
                assert_eq!(
                    hue.sum_range_for_chroma(0.0),
                    SumRange((0.0, 3.0, 1.0 + other))
                );
                assert_eq!(
                    hue.sum_range_for_chroma(1.0),
                    SumRange((1.0 + *other, 1.0 + *other, 1.0 + other))
                );
                for chroma in NON_ZERO_CHROMAS.iter() {
                    let range = hue.sum_range_for_chroma(*chroma);
                    let max_chroma = hue.max_chroma_for_sum(range.shade_min());
                    assert_approx_eq!(max_chroma, *chroma);
                    let max_chroma = hue.max_chroma_for_sum(range.tint_max());
                    assert_approx_eq!(max_chroma, *chroma, 0.000000000000001);
                }
            }
        }
    }

    #[test]
    fn primary_max_chroma_rgbs() {
        for (hue, expected_rgb) in Hue::<f64>::PRIMARIES
            .iter()
            .zip(RGB::<f64>::PRIMARIES.iter())
        {
            assert_eq!(hue.max_chroma_rgb_for_sum(1.0), *expected_rgb);
            assert_eq!(hue.max_chroma_rgb_for_sum(0.0), RGB::BLACK);
            assert_eq!(hue.max_chroma_rgb_for_sum(3.0), RGB::WHITE);
            for sum in [0.0001, 0.25, 0.5, 0.75, 0.9999].iter() {
                let mut array = [0.0_f64, 0.0, 0.0];
                array[hue.indices().0 as usize] = *sum;
                let expected: RGB<f64> = array.into();
                assert_eq!(hue.max_chroma_rgb_for_sum(*sum), expected);
            }
            for sum in [2.0001, 2.25, 2.5, 2.75, 2.9999].iter() {
                let mut array = [1.0_f64, 1.0, 1.0];
                array[hue.indices().1 as usize] = (sum - 1.0) / 2.0;
                array[hue.indices().2 as usize] = (sum - 1.0) / 2.0;
                let expected: RGB<f64> = array.into();
                assert_eq!(hue.max_chroma_rgb_for_sum(*sum), expected);
            }
        }
    }

    #[test]
    fn secondary_max_chroma_rgbs() {
        for (hue, expected_rgb) in Hue::<f64>::SECONDARIES
            .iter()
            .zip(RGB::<f64>::SECONDARIES.iter())
        {
            assert_eq!(hue.max_chroma_rgb_for_sum(2.0), *expected_rgb);
            assert_eq!(hue.max_chroma_rgb_for_sum(0.0), RGB::BLACK);
            assert_eq!(hue.max_chroma_rgb_for_sum(3.0), RGB::WHITE);
            for sum in [0.0001, 0.25, 0.5, 0.75, 1.0, 1.5, 1.9999].iter() {
                let mut array = [0.0_f64, 0.0, 0.0];
                array[hue.indices().0 as usize] = sum / 2.0;
                array[hue.indices().1 as usize] = sum / 2.0;
                let expected: RGB<f64> = array.into();
                assert_eq!(hue.max_chroma_rgb_for_sum(*sum), expected);
            }
            for sum in [2.0001, 2.25, 2.5, 2.75, 2.9999].iter() {
                let mut array = [1.0_f64, 1.0, 1.0];
                array[hue.indices().2 as usize] = sum - 2.0;
                let expected: RGB<f64> = array.into();
                assert_eq!(hue.max_chroma_rgb_for_sum(*sum), expected);
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
                assert_eq!(hue.max_chroma_rgb_for_sum(0.0), RGB::BLACK);
                assert_eq!(hue.max_chroma_rgb_for_sum(3.0), RGB::WHITE);
                println!("hue: {:?} MAX_CHROMA_RGB: {:?}", hue, hue.max_chroma_rgb());
                for sum in VALID_OTHER_SUMS.iter() {
                    let rgb = hue.max_chroma_rgb_for_sum(*sum);
                    assert_approx_eq!(rgb.sum(), *sum);
                    if *sum < 3.0 - *second {
                        if let Hue::<f64>::Sextant(sextant_hue_out) =
                            Hue::<f64>::try_from(&rgb).unwrap()
                        {
                            assert_eq!(sextant_hue.0, sextant_hue_out.0);
                            assert_approx_eq!(sextant_hue.1, sextant_hue_out.1, 0.000000000001);
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
            assert_eq!(hue.min_sum_rgb_for_chroma(1.0), *expected_rgb);
            assert_eq!(hue.max_sum_rgb_for_chroma(1.0), *expected_rgb);
            let shade = hue.min_sum_rgb_for_chroma(0.5);
            let tint = hue.max_sum_rgb_for_chroma(0.5);
            assert!(shade.value() < tint.value());
            assert_approx_eq!(shade.chroma(), 0.5, 0.00000000001);
            assert_approx_eq!(tint.chroma(), 0.5, 0.00000000001);
            assert_approx_eq!(shade.max_chroma_rgb(), tint.max_chroma_rgb(), 0.0000001);
        }
        for (hue, expected_rgb) in Hue::<f64>::SECONDARIES
            .iter()
            .zip(RGB::<f64>::SECONDARIES.iter())
        {
            println!("{:?} : {:?}", hue, expected_rgb);
            assert_eq!(hue.min_sum_rgb_for_chroma(1.0), *expected_rgb);
            assert_eq!(hue.max_sum_rgb_for_chroma(1.0), *expected_rgb);
            let shade = hue.min_sum_rgb_for_chroma(0.5);
            let tint = hue.max_sum_rgb_for_chroma(0.5);
            assert!(shade.value() < tint.value());
            assert_approx_eq!(shade.chroma(), 0.5, 0.00000000001);
            assert_approx_eq!(tint.chroma(), 0.5, 0.00000000001);
            assert_approx_eq!(shade.max_chroma_rgb(), tint.max_chroma_rgb(), 0.0000001);
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
                assert_eq!(hue.min_sum_rgb_for_chroma(0.0), RGB::BLACK);
                assert_eq!(hue.max_sum_rgb_for_chroma(0.0), RGB::WHITE);
                for chroma in NON_ZERO_CHROMAS.iter() {
                    let shade = hue.min_sum_rgb_for_chroma(*chroma);
                    let tint = hue.max_sum_rgb_for_chroma(*chroma);
                    assert!(shade.sum() <= tint.sum());
                    assert_approx_eq!(shade.chroma(), *chroma, 0.00000000001);
                    assert_approx_eq!(tint.chroma(), *chroma, 0.00000000001);
                    assert_approx_eq!(shade.max_chroma_rgb(), tint.max_chroma_rgb(), 0.000_001);
                }
            }
        }
    }

    #[test]
    fn primary_rgb_for_sum_and_chroma() {
        for hue in &Hue::<f64>::PRIMARIES {
            assert!(hue.rgb_for_sum_and_chroma(0.0, 1.0).is_none());
            assert!(hue.rgb_for_sum_and_chroma(3.0, 1.0).is_none());
            assert!(hue.rgb_for_sum_and_chroma(0.0, 0.0).is_none());
            assert!(hue.rgb_for_sum_and_chroma(3.0, 0.0).is_none());
            for chroma in &NON_ZERO_CHROMAS {
                for sum in &VALID_OTHER_SUMS {
                    if let Some(rgb) = hue.rgb_for_sum_and_chroma(*sum, *chroma) {
                        assert_approx_eq!(rgb.sum(), *sum, 0.000_000_000_1);
                        assert_approx_eq!(rgb.chroma(), *chroma, 0.000_000_000_1);
                        assert_approx_eq!(Hue::<f64>::try_from(&rgb).unwrap(), hue);
                    } else {
                        use SumRangeComparisonResult::*;
                        let range = hue.sum_range_for_chroma(*chroma);
                        assert!([TooSmall, TooBig].contains(&range.compare_sum(*sum)));
                    }
                }
            }
        }
    }

    #[test]
    fn secondary_rgb_for_sum_and_chroma() {
        for hue in &Hue::<f64>::SECONDARIES {
            assert!(hue.rgb_for_sum_and_chroma(0.0, 1.0).is_none());
            assert!(hue.rgb_for_sum_and_chroma(3.0, 1.0).is_none());
            assert!(hue.rgb_for_sum_and_chroma(0.0, 0.0).is_none());
            assert!(hue.rgb_for_sum_and_chroma(3.0, 0.0).is_none());
            for chroma in &NON_ZERO_CHROMAS {
                for sum in &VALID_OTHER_SUMS {
                    if let Some(rgb) = hue.rgb_for_sum_and_chroma(*sum, *chroma) {
                        assert_approx_eq!(rgb.sum(), *sum, 0.000_000_1);
                        assert_approx_eq!(rgb.chroma(), *chroma, 0.000_000_1);
                        assert_approx_eq!(Hue::<f64>::try_from(&rgb).unwrap(), hue);
                    } else {
                        use SumRangeComparisonResult::*;
                        let range = hue.sum_range_for_chroma(*chroma);
                        assert!([TooSmall, TooBig].contains(&range.compare_sum(*sum)));
                    }
                }
            }
        }
    }

    #[test]
    fn general_rgb_for_sum_and_chroma() {
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
                assert!(hue.rgb_for_sum_and_chroma(0.0, 1.0).is_none());
                assert!(hue.rgb_for_sum_and_chroma(3.0, 1.0).is_none());
                assert!(hue.rgb_for_sum_and_chroma(0.0, 0.0).is_none());
                assert!(hue.rgb_for_sum_and_chroma(3.0, 0.0).is_none());
                for chroma in &NON_ZERO_CHROMAS {
                    for sum in &VALID_OTHER_SUMS {
                        println!(
                            "{:?}, {:?}, {:?} :: {:?}",
                            hue,
                            sum,
                            chroma,
                            hue.sum_range_for_chroma(*chroma)
                        );
                        if let Some(rgb) = hue.rgb_for_sum_and_chroma(*sum, *chroma) {
                            assert_approx_eq!(rgb.sum(), *sum);
                        // assert_approx_eq!(rgb.chroma(), *chroma, 0.000_000_1);
                        // assert_approx_eq!(
                        //     Hue::<f64>::try_from(&rgb).unwrap(),
                        //     hue,
                        //     0.000_000_000_1
                        // );
                        } else {
                            use SumRangeComparisonResult::*;
                            let range = hue.sum_range_for_chroma(*chroma);
                            assert!([TooSmall, TooBig].contains(&range.compare_sum(*sum)));
                        }
                    }
                }
            }
        }
    }
}
