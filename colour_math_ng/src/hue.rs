// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    cmp::Ordering,
    convert::{Into, TryFrom},
    fmt::Debug,
};

use normalised_angles::Degrees;

use crate::{
    proportion::{Chroma, ProportionValidation, SumValidation, UFDFraction},
    rgb::*,
    ChromaOneRGB, Float, HueAngle, HueConstants, LightLevel, RGBConstants,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct SumRange((UFDFraction, UFDFraction, UFDFraction));

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum SumOrdering {
    TooSmall,
    Shade(UFDFraction, UFDFraction),
    Tint(UFDFraction, UFDFraction),
    TooBig,
}

impl SumOrdering {
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

impl SumRange {
    pub fn compare_sum(&self, sum: UFDFraction) -> SumOrdering {
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

    pub fn shade_min(&self) -> UFDFraction {
        self.0 .0
    }

    pub fn shade_max(&self) -> UFDFraction {
        self.0 .1
    }

    pub fn tint_min(&self) -> UFDFraction {
        self.0 .1
    }

    pub fn tint_max(&self) -> UFDFraction {
        self.0 .2
    }
}

pub trait HueIfceTmp {
    fn sum_range_for_chroma(&self, chroma_value: UFDFraction) -> Option<SumRange>;
    fn max_chroma_for_sum(&self, sum: UFDFraction) -> Option<Chroma>;

    fn max_chroma_rgb<T: LightLevel>(&self) -> RGB<T>;
    fn max_chroma_rgb_for_sum<T: LightLevel>(&self, sum: UFDFraction) -> Option<RGB<T>>;
    fn min_sum_rgb_for_chroma<T: LightLevel>(&self, chroma_value: UFDFraction) -> RGB<T>;
    fn max_sum_rgb_for_chroma<T: LightLevel>(&self, chroma_value: UFDFraction) -> RGB<T>;
    fn rgb_for_sum_and_chroma<T: LightLevel>(
        &self,
        sum: UFDFraction,
        chroma_value: UFDFraction,
    ) -> Option<RGB<T>>;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord)]
pub enum RGBHue {
    Red = 5,
    Green = 9,
    Blue = 1,
}

impl RGBHue {
    fn make_rgb<T: LightLevel>(&self, components: (UFDFraction, UFDFraction)) -> RGB<T> {
        use RGBHue::*;
        match self {
            Red => [components.0, components.1, components.1].into(),
            Green => [components.1, components.0, components.1].into(),
            Blue => [components.1, components.1, components.0].into(),
        }
    }
}

impl<T: Float> HueAngle<T> for RGBHue {
    fn hue_angle(&self) -> Degrees<T> {
        match self {
            RGBHue::Red => Degrees::RED,
            RGBHue::Green => Degrees::GREEN,
            RGBHue::Blue => Degrees::BLUE,
        }
    }
}

impl<T: LightLevel> ChromaOneRGB<T> for RGBHue {
    /// RGB wih chroma of 1.0 chroma and with its hue (value may change op or down)
    fn chroma_one_rgb(&self) -> RGB<T> {
        match self {
            RGBHue::Red => RGB::RED,
            RGBHue::Green => RGB::GREEN,
            RGBHue::Blue => RGB::BLUE,
        }
    }
}

impl HueIfceTmp for RGBHue {
    fn sum_range_for_chroma(&self, chroma: UFDFraction) -> Option<SumRange> {
        debug_assert!(chroma.is_vp(), "chroma: {:?}", chroma);
        if chroma == UFDFraction::ZERO {
            None
        } else {
            Some(SumRange((
                chroma.into(),
                UFDFraction::ONE,
                (UFDFraction::THREE - UFDFraction::TWO * chroma).min(UFDFraction::THREE),
            )))
        }
    }

    fn max_chroma_for_sum(&self, sum: UFDFraction) -> Option<Chroma> {
        debug_assert!(sum.is_vs(), "sum: {:?}", sum);
        if sum == UFDFraction::ZERO || sum == UFDFraction::THREE {
            None
        } else if sum < UFDFraction::ONE {
            Some(Chroma::Shade(sum.into()))
        } else if sum > UFDFraction::ONE {
            Some(Chroma::Tint(
                ((UFDFraction::THREE - sum) / UFDFraction::TWO).into(),
            ))
        } else {
            Some(Chroma::ONE)
        }
    }

    fn max_chroma_rgb<T: LightLevel>(&self) -> RGB<T> {
        match self {
            RGBHue::Red => RGB::RED,
            RGBHue::Green => RGB::GREEN,
            RGBHue::Blue => RGB::BLUE,
        }
    }

    fn max_chroma_rgb_for_sum<T: LightLevel>(&self, sum: UFDFraction) -> Option<RGB<T>> {
        debug_assert!(sum.is_vs(), "sum: {:?}", sum);
        if sum == UFDFraction::ZERO || sum == UFDFraction::THREE {
            None
        } else {
            if sum <= UFDFraction::ONE {
                Some(self.make_rgb((sum.into(), UFDFraction::ZERO)))
            } else {
                Some(
                    self.make_rgb((
                        UFDFraction::ONE,
                        ((sum - UFDFraction::ONE) / UFDFraction::TWO)
                            .min(UFDFraction::ONE)
                            .into(),
                    )),
                )
            }
        }
    }

    fn min_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: UFDFraction) -> RGB<T> {
        // TODO: Needs major revision taking into account Shade/Tint
        debug_assert!(chroma.is_vp(), "chroma: {:?}", chroma);
        if chroma == UFDFraction::ZERO {
            RGB::BLACK
        } else if chroma == UFDFraction::ONE {
            self.max_chroma_rgb()
        } else {
            self.make_rgb((chroma, UFDFraction::ZERO))
        }
    }

    fn max_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: UFDFraction) -> RGB<T> {
        // TODO: Needs major revision taking into account Shade/Tint
        debug_assert!(chroma.is_vp(), "chroma: {:?}", chroma);
        if chroma == UFDFraction::ZERO {
            RGB::WHITE
        } else if chroma == UFDFraction::ONE {
            self.max_chroma_rgb()
        } else {
            self.make_rgb((UFDFraction::ONE, UFDFraction::ONE - chroma))
        }
    }

    fn rgb_for_sum_and_chroma<T: LightLevel>(
        &self,
        sum: UFDFraction,
        chroma: UFDFraction,
    ) -> Option<RGB<T>> {
        // TODO: Needs major revision taking into account Shade/Tint
        debug_assert!(sum.is_vs(), "sum: {:?}", sum);
        debug_assert!(chroma.is_vp(), "chroma: {:?}", chroma);
        let sum_range = self.sum_range_for_chroma(chroma)?;
        if sum_range.compare_sum(sum).is_success() {
            let other: UFDFraction = ((sum - chroma) / UFDFraction::THREE).into();
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
    fn make_rgb<T: LightLevel>(&self, components: (UFDFraction, UFDFraction)) -> RGB<T> {
        use CMYHue::*;
        match self {
            Cyan => [components.1, components.0, components.0].into(),
            Magenta => [components.0, components.1, components.0].into(),
            Yellow => [components.0, components.0, components.1].into(),
        }
    }
}

impl<T: Float> HueAngle<T> for CMYHue {
    fn hue_angle(&self) -> Degrees<T> {
        match self {
            CMYHue::Cyan => Degrees::CYAN,
            CMYHue::Magenta => Degrees::MAGENTA,
            CMYHue::Yellow => Degrees::YELLOW,
        }
    }
}

impl HueIfceTmp for CMYHue {
    fn sum_range_for_chroma(&self, chroma: UFDFraction) -> Option<SumRange> {
        debug_assert!(chroma.is_vp(), "chroma: {:?}", chroma);
        if chroma == UFDFraction::ZERO {
            None
        } else {
            let c_sum: UFDFraction = chroma.into();
            Some(SumRange((
                c_sum * UFDFraction::TWO,
                UFDFraction::TWO,
                UFDFraction::THREE - c_sum,
            )))
        }
    }

    fn max_chroma_for_sum(&self, sum: UFDFraction) -> Option<Chroma> {
        debug_assert!(sum.is_vs(), "sum: {:?}", sum);
        if sum == UFDFraction::ZERO || sum == UFDFraction::THREE {
            None
        } else if sum < UFDFraction::TWO {
            Some(Chroma::Shade(
                (sum / UFDFraction::TWO).min(UFDFraction::ONE).into(),
            ))
        } else if sum > UFDFraction::TWO {
            Some(Chroma::Tint(
                (UFDFraction::THREE - sum).min(UFDFraction::ONE).into(),
            ))
        } else {
            Some(Chroma::ONE)
        }
    }

    fn max_chroma_rgb<T: LightLevel>(&self) -> RGB<T> {
        match self {
            CMYHue::Cyan => RGB::CYAN,
            CMYHue::Magenta => RGB::MAGENTA,
            CMYHue::Yellow => RGB::YELLOW,
        }
    }

    fn max_chroma_rgb_for_sum<T: LightLevel>(&self, sum: UFDFraction) -> Option<RGB<T>> {
        debug_assert!(sum.is_vs(), "sum: {:?}", sum);
        if sum == UFDFraction::ZERO || sum == UFDFraction::THREE {
            None
        } else if sum < UFDFraction::TWO {
            Some(self.make_rgb((
                (sum / UFDFraction::TWO).min(UFDFraction::ONE).into(),
                UFDFraction::ZERO,
            )))
        } else if sum > UFDFraction::TWO {
            Some(
                self.make_rgb((
                    UFDFraction::ONE,
                    (sum - UFDFraction::TWO)
                        .max(UFDFraction::ZERO)
                        .min(UFDFraction::ONE)
                        .into(),
                )),
            )
        } else {
            Some(self.max_chroma_rgb())
        }
    }

    fn min_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: UFDFraction) -> RGB<T> {
        // TODO: Needs major revision taking into account Shade/Tint
        debug_assert!(chroma.is_vp(), "chroma: {:?}", chroma);
        if chroma == UFDFraction::ZERO {
            RGB::BLACK
        } else if chroma == UFDFraction::ONE {
            self.max_chroma_rgb()
        } else {
            self.make_rgb((chroma, UFDFraction::ZERO))
        }
    }

    fn max_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: UFDFraction) -> RGB<T> {
        // TODO: Needs major revision taking into account Shade/Tint
        debug_assert!(chroma.is_vp(), "chroma: {:?}", chroma);
        if chroma == UFDFraction::ZERO {
            RGB::WHITE
        } else if chroma == UFDFraction::ONE {
            self.max_chroma_rgb()
        } else {
            self.make_rgb((UFDFraction::ONE, UFDFraction::ONE - chroma))
        }
    }

    fn rgb_for_sum_and_chroma<T: LightLevel>(
        &self,
        sum: UFDFraction,
        chroma: UFDFraction,
    ) -> Option<RGB<T>> {
        debug_assert!(sum.is_vs(), "sum: {:?}", sum);
        debug_assert!(chroma.is_vp());
        let sum_range = self.sum_range_for_chroma(chroma)?;
        if sum_range.compare_sum(sum).is_success() {
            // TODO: reassess this calculation
            Some(self.make_rgb((
                ((sum + chroma) / UFDFraction::THREE).into(),
                ((sum - UFDFraction::TWO * chroma) / UFDFraction::THREE).into(),
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
pub struct SextantHue(Sextant, UFDFraction);

impl Eq for SextantHue {}

impl SextantHue {
    fn make_rgb<T: LightLevel>(
        &self,
        components: (UFDFraction, UFDFraction, UFDFraction),
    ) -> RGB<T> {
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

    pub fn approx_eq(&self, other: &Self, max_diff: Option<f64>) -> bool {
        if self.0 == other.0 {
            self.1.approx_eq(&other.1, max_diff)
        } else {
            false
        }
    }
}

impl<T: LightLevel> From<(Sextant, &RGB<T>)> for SextantHue {
    fn from(arg: (Sextant, &RGB<T>)) -> Self {
        use Sextant::*;
        let [red, green, blue] = <[UFDFraction; 3]>::from(*arg.1);
        match arg.0 {
            RedMagenta => Self(arg.0, (blue - green) / (red - green)),
            RedYellow => Self(arg.0, (green - blue) / (red - blue)),
            GreenYellow => Self(arg.0, (red - blue) / (green - blue)),
            GreenCyan => Self(arg.0, (blue - red) / (green - red)),
            BlueCyan => Self(arg.0, (green - red) / (blue - red)),
            BlueMagenta => Self(arg.0, (red - green) / (blue - green)),
        }
    }
}

impl<T: Float + From<UFDFraction> + Copy> HueAngle<T> for SextantHue {
    fn hue_angle(&self) -> Degrees<T> {
        let second: T = self.1.into();
        let sin = T::SQRT_3 * second / T::TWO / (T::ONE - second + second.powi(2)).sqrt();
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

impl HueIfceTmp for SextantHue {
    fn sum_range_for_chroma(&self, chroma: UFDFraction) -> Option<SumRange> {
        debug_assert!(chroma.is_vp(), "chroma: {:?}", chroma);
        if chroma == UFDFraction::ZERO {
            None
        } else {
            let max_c_sum = (UFDFraction::ONE + self.1).min(UFDFraction::TWO);
            if chroma == UFDFraction::ONE {
                Some(SumRange((max_c_sum, max_c_sum, max_c_sum)))
            } else {
                let min = max_c_sum * chroma;
                let max = UFDFraction::THREE - (UFDFraction::TWO - self.1) * chroma;
                Some(SumRange((min, max_c_sum, max)))
            }
        }
    }

    fn max_chroma_for_sum(&self, sum: UFDFraction) -> Option<Chroma> {
        debug_assert!(sum.is_vs(), "sum: {:?}", sum);
        if sum == UFDFraction::ZERO || sum == UFDFraction::THREE {
            None
        } else {
            match sum.cmp(&(UFDFraction::ONE + self.1)) {
                Ordering::Less => {
                    let temp = sum / (UFDFraction::ONE + self.1);
                    Some(Chroma::Shade(temp))
                }
                Ordering::Greater => {
                    let temp = (UFDFraction::THREE - sum) / (UFDFraction::TWO - self.1);
                    Some(Chroma::Tint(temp))
                }
                Ordering::Equal => Some(Chroma::ONE),
            }
        }
    }

    fn max_chroma_rgb<T: LightLevel>(&self) -> RGB<T> {
        self.make_rgb((UFDFraction::ONE, self.1, UFDFraction::ZERO))
    }

    fn max_chroma_rgb_for_sum<T: LightLevel>(&self, sum: UFDFraction) -> Option<RGB<T>> {
        debug_assert!(sum.is_vs(), "sum: {:?}", sum);
        // TODO: make hue drift an error
        if sum == UFDFraction::ZERO || sum == UFDFraction::THREE {
            None
        } else {
            let max_chroma_sum = UFDFraction::ONE + self.1;
            if sum == max_chroma_sum {
                Some(self.max_chroma_rgb())
            } else {
                let components = if sum < max_chroma_sum {
                    let first: UFDFraction = (sum / max_chroma_sum).min(UFDFraction::ONE).into();
                    (first, first * self.1, UFDFraction::ZERO)
                } else {
                    let temp = sum - UFDFraction::ONE;
                    let second = ((temp + self.1) / UFDFraction::TWO).min(UFDFraction::ONE);
                    (
                        UFDFraction::ONE,
                        second.into(),
                        (temp - second).max(UFDFraction::ZERO).into(),
                    )
                };
                Some(self.make_rgb(components))
            }
        }
    }

    fn min_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: UFDFraction) -> RGB<T> {
        debug_assert!(chroma.is_vp(), "chroma: {:?}", chroma);
        if chroma == UFDFraction::ZERO {
            RGB::BLACK
        } else if chroma == UFDFraction::ONE {
            self.max_chroma_rgb()
        } else {
            self.make_rgb((chroma, self.1 * chroma, UFDFraction::ZERO))
        }
    }

    fn max_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: UFDFraction) -> RGB<T> {
        debug_assert!(chroma.is_vp(), "chroma: {:?}", chroma);
        if chroma == UFDFraction::ZERO {
            RGB::WHITE
        } else if chroma == UFDFraction::ONE {
            self.max_chroma_rgb()
        } else {
            let third = UFDFraction::ONE - chroma;
            let second: UFDFraction = (chroma * self.1 + third).into();
            self.make_rgb((UFDFraction::ONE, second, third))
        }
    }

    fn rgb_for_sum_and_chroma<T: LightLevel>(
        &self,
        sum: UFDFraction,
        chroma: UFDFraction,
    ) -> Option<RGB<T>> {
        debug_assert!(sum.is_vs(), "sum: {:?}", sum);
        debug_assert!(chroma.is_vp());
        let sum_range = self.sum_range_for_chroma(chroma)?;
        if sum_range.compare_sum(sum).is_success() {
            let delta: UFDFraction = (sum - sum_range.shade_min()) / UFDFraction::THREE;
            let first = chroma + delta;
            let second = chroma * self.1 + delta;
            Some(self.make_rgb((first, second, delta)))
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Hue {
    Primary(RGBHue),
    Secondary(CMYHue),
    Sextant(SextantHue),
}

impl Eq for Hue {}

impl HueConstants for Hue {
    const RED: Self = Self::Primary(RGBHue::Red);
    const GREEN: Self = Self::Primary(RGBHue::Green);
    const BLUE: Self = Self::Primary(RGBHue::Blue);

    const CYAN: Self = Self::Secondary(CMYHue::Cyan);
    const MAGENTA: Self = Self::Secondary(CMYHue::Magenta);
    const YELLOW: Self = Self::Secondary(CMYHue::Yellow);
}

impl<T: LightLevel> TryFrom<&RGB<T>> for Hue {
    type Error = &'static str;

    fn try_from(rgb: &RGB<T>) -> Result<Self, Self::Error> {
        use Sextant::*;
        let [red, green, blue] = <[UFDFraction; 3]>::from(*rgb);
        match red.cmp(&green) {
            Ordering::Greater => match green.cmp(&blue) {
                Ordering::Greater => Ok(Hue::Sextant(SextantHue::from((RedYellow, rgb)))),
                Ordering::Less => match red.cmp(&blue) {
                    Ordering::Greater => Ok(Hue::Sextant(SextantHue::from((RedMagenta, rgb)))),
                    Ordering::Less => Ok(Hue::Sextant(SextantHue::from((BlueMagenta, rgb)))),
                    Ordering::Equal => Ok(Hue::Secondary(CMYHue::Magenta)),
                },
                Ordering::Equal => Ok(Hue::Primary(RGBHue::Red)),
            },
            Ordering::Less => match red.cmp(&blue) {
                Ordering::Greater => Ok(Hue::Sextant(SextantHue::from((GreenYellow, rgb)))),
                Ordering::Less => match green.cmp(&blue) {
                    Ordering::Greater => Ok(Hue::Sextant(SextantHue::from((GreenCyan, rgb)))),
                    Ordering::Less => Ok(Hue::Sextant(SextantHue::from((BlueCyan, rgb)))),
                    Ordering::Equal => Ok(Hue::Secondary(CMYHue::Cyan)),
                },
                Ordering::Equal => Ok(Hue::Primary(RGBHue::Green)),
            },
            Ordering::Equal => match red.cmp(&blue) {
                Ordering::Greater => Ok(Hue::Secondary(CMYHue::Yellow)),
                Ordering::Less => Ok(Hue::Primary(RGBHue::Blue)),
                Ordering::Equal => Err("RGB is grey and hs no hue"),
            },
        }
    }
}

impl<T: Float + From<UFDFraction>> HueAngle<T> for Hue {
    fn hue_angle(&self) -> Degrees<T> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.hue_angle(),
            Self::Secondary(cmy_hue) => cmy_hue.hue_angle(),
            Self::Sextant(sextant_hue) => sextant_hue.hue_angle(),
        }
    }
}

impl HueIfceTmp for Hue {
    fn sum_range_for_chroma(&self, chroma: UFDFraction) -> Option<SumRange> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.sum_range_for_chroma(chroma),
            Self::Secondary(cmy_hue) => cmy_hue.sum_range_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.sum_range_for_chroma(chroma),
        }
    }

    fn max_chroma_for_sum(&self, sum: UFDFraction) -> Option<Chroma> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.max_chroma_for_sum(sum),
            Self::Secondary(cmy_hue) => cmy_hue.max_chroma_for_sum(sum),
            Self::Sextant(sextant_hue) => sextant_hue.max_chroma_for_sum(sum),
        }
    }

    fn max_chroma_rgb<T: LightLevel>(&self) -> RGB<T> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.max_chroma_rgb(),
            Self::Secondary(cmy_hue) => cmy_hue.max_chroma_rgb(),
            Self::Sextant(sextant_hue) => sextant_hue.max_chroma_rgb(),
        }
    }

    fn max_chroma_rgb_for_sum<T: LightLevel>(&self, sum: UFDFraction) -> Option<RGB<T>> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.max_chroma_rgb_for_sum(sum),
            Self::Secondary(cmy_hue) => cmy_hue.max_chroma_rgb_for_sum(sum),
            Self::Sextant(sextant_hue) => sextant_hue.max_chroma_rgb_for_sum(sum),
        }
    }

    fn min_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: UFDFraction) -> RGB<T> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.min_sum_rgb_for_chroma(chroma),
            Self::Secondary(cmy_hue) => cmy_hue.min_sum_rgb_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.min_sum_rgb_for_chroma(chroma),
        }
    }

    fn max_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: UFDFraction) -> RGB<T> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.max_sum_rgb_for_chroma(chroma),
            Self::Secondary(cmy_hue) => cmy_hue.max_sum_rgb_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.max_sum_rgb_for_chroma(chroma),
        }
    }

    fn rgb_for_sum_and_chroma<T: LightLevel>(
        &self,
        sum: UFDFraction,
        chroma: UFDFraction,
    ) -> Option<RGB<T>> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.rgb_for_sum_and_chroma(sum, chroma),
            Self::Secondary(cmy_hue) => cmy_hue.rgb_for_sum_and_chroma(sum, chroma),
            Self::Sextant(sextant_hue) => sextant_hue.rgb_for_sum_and_chroma(sum, chroma),
        }
    }
}

impl Hue {
    pub fn ord_index(&self) -> u8 {
        0
    }

    pub fn approx_eq(&self, other: &Self, max_diff: Option<f64>) -> bool {
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

impl PartialOrd for Hue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.ord_index().partial_cmp(&other.ord_index())
    }
}

impl Ord for Hue {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[cfg(test)]
mod hue_ng_tests {
    use super::*;
    use num_traits_plus::{assert_approx_eq, float_plus::FloatApproxEq};

    use crate::{proportion::*, rgb::RGB, CCI};

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

    impl SextantHue {
        fn indices(&self) -> (CCI, CCI, CCI) {
            self.0.indices()
        }
    }

    impl Hue {
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
            RGB::from([0.5_f64, 0.5_f64, 0.5_f64]),
        ] {
            assert!(Hue::try_from(rgb).is_err());
        }
        for (rgb, hue) in RGB::<f64>::PRIMARIES.iter().zip(Hue::PRIMARIES.iter()) {
            assert_eq!(Hue::try_from(rgb), Ok(*hue));
            assert_eq!(Hue::try_from(&(*rgb * UFDFraction::from(0.5))), Ok(*hue));
        }
        for (rgb, hue) in RGB::<f64>::SECONDARIES.iter().zip(Hue::SECONDARIES.iter()) {
            assert_eq!(Hue::try_from(rgb), Ok(*hue));
            assert_eq!(Hue::try_from(&(*rgb * UFDFraction::from(0.5))), Ok(*hue));
        }
        for (array, sextant, second) in &[
            (
                [
                    UFDFraction::ONE,
                    UFDFraction::from(0.5_f64),
                    UFDFraction::ZERO,
                ],
                Sextant::RedYellow,
                UFDFraction::from(0.5),
            ),
            (
                [
                    UFDFraction::ZERO,
                    UFDFraction::from(0.25_f64),
                    UFDFraction::from(0.5_f64),
                ],
                Sextant::BlueCyan,
                UFDFraction::from(0.5),
            ),
            (
                [
                    UFDFraction::from(0.2_f64),
                    UFDFraction::ZERO,
                    UFDFraction::from(0.4_f64),
                ],
                Sextant::BlueMagenta,
                UFDFraction::from(0.5),
            ),
            (
                [
                    UFDFraction::from(0.5_f64),
                    UFDFraction::ZERO,
                    UFDFraction::ONE,
                ],
                Sextant::BlueMagenta,
                UFDFraction::from(0.5),
            ),
            (
                [
                    UFDFraction::ONE,
                    UFDFraction::ZERO,
                    UFDFraction::from(0.5_f64),
                ],
                Sextant::RedMagenta,
                UFDFraction::from(0.5),
            ),
            (
                [
                    UFDFraction::from(0.5_f64),
                    UFDFraction::ONE,
                    UFDFraction::ZERO,
                ],
                Sextant::GreenYellow,
                UFDFraction::from(0.5),
            ),
            (
                [
                    UFDFraction::ZERO,
                    UFDFraction::ONE,
                    UFDFraction::from(0.5_f64),
                ],
                Sextant::GreenCyan,
                UFDFraction::from(0.5),
            ),
        ] {
            let rgb = RGB::<f64>::from([
                UFDFraction::from(array[0]),
                UFDFraction::from(array[1]),
                UFDFraction::from(array[2]),
            ]);
            let hue = Hue::Sextant(SextantHue(*sextant, *second));
            assert_approx_eq!(Hue::try_from(&rgb).unwrap(), hue, 0.000_000_001);
        }
    }

    #[test]
    fn hue_max_chroma_rgb() {
        for (hue, rgb) in Hue::PRIMARIES.iter().zip(RGB::<f64>::PRIMARIES.iter()) {
            assert_eq!(hue.max_chroma_rgb(), *rgb);
        }
        for (hue, rgb) in Hue::SECONDARIES.iter().zip(RGB::<f64>::SECONDARIES.iter()) {
            assert_eq!(hue.max_chroma_rgb(), *rgb);
        }
        for (array, sextant, second) in &[
            (
                [
                    UFDFraction::ONE,
                    UFDFraction::from(0.5_f64),
                    UFDFraction::ZERO,
                ],
                Sextant::RedYellow,
                UFDFraction::from(0.5_f64),
            ),
            (
                [
                    UFDFraction::ZERO,
                    UFDFraction::from(0.5_f64),
                    UFDFraction::ONE,
                ],
                Sextant::BlueCyan,
                UFDFraction::from(0.5_f64),
            ),
            (
                [
                    UFDFraction::from(0.5_f64),
                    UFDFraction::ZERO,
                    UFDFraction::ONE,
                ],
                Sextant::BlueMagenta,
                UFDFraction::from(0.5_f64),
            ),
            (
                [
                    UFDFraction::ONE,
                    UFDFraction::ZERO,
                    UFDFraction::from(0.5_f64),
                ],
                Sextant::RedMagenta,
                UFDFraction::from(0.5_f64),
            ),
            (
                [
                    UFDFraction::from(0.5_f64),
                    UFDFraction::ONE,
                    UFDFraction::ZERO,
                ],
                Sextant::GreenYellow,
                UFDFraction::from(0.5_f64),
            ),
            (
                [
                    UFDFraction::ZERO,
                    UFDFraction::ONE,
                    UFDFraction::from(0.5_f64),
                ],
                Sextant::GreenCyan,
                UFDFraction::from(0.5_f64),
            ),
        ] {
            let rgb = RGB::<f64>::from(*array);
            let hue = Hue::Sextant(SextantHue(*sextant, *second));
            println!("{:?} {:?} {:?} {:?}", rgb, sextant, second, hue);
            assert_eq!(Hue::try_from(&rgb), Ok(hue));
        }
    }

    #[test]
    fn hue_angle() {
        for (hue, angle) in Hue::PRIMARIES.iter().zip(Degrees::<f64>::PRIMARIES.iter()) {
            assert_eq!(hue.hue_angle(), *angle);
        }
        for (hue, angle) in Hue::SECONDARIES
            .iter()
            .zip(Degrees::<f64>::SECONDARIES.iter())
        {
            assert_eq!(hue.hue_angle(), *angle);
        }
        for (sextant, second, angle) in &[
            (
                Sextant::RedYellow,
                UFDFraction::from(0.5_f64),
                Degrees::<f64>::DEG_30,
            ),
            (
                Sextant::BlueCyan,
                UFDFraction::from(0.5_f64),
                Degrees::<f64>::NEG_DEG_150,
            ),
            (
                Sextant::BlueMagenta,
                UFDFraction::from(0.5_f64),
                Degrees::<f64>::NEG_DEG_90,
            ),
            (
                Sextant::RedMagenta,
                UFDFraction::from(0.5_f64),
                Degrees::<f64>::NEG_DEG_30,
            ),
            (
                Sextant::GreenYellow,
                UFDFraction::from(0.5_f64),
                Degrees::<f64>::DEG_90,
            ),
            (
                Sextant::GreenCyan,
                UFDFraction::from(0.5_f64),
                Degrees::<f64>::DEG_150,
            ),
        ] {
            let hue = Hue::Sextant(SextantHue(*sextant, *second));
            assert_approx_eq!(hue.hue_angle(), *angle, 0.0000001);
        }
    }

    // TODO: this test needs to be improved
    #[test]
    fn max_chroma_and_sum_ranges() {
        for hue in &Hue::PRIMARIES {
            assert!(hue
                .sum_range_for_chroma(UFDFraction::from(0.0_f64))
                .is_none());
            assert_eq!(
                hue.sum_range_for_chroma(UFDFraction::from(1.0_f64)),
                Some(SumRange((
                    UFDFraction::from(1.0_f64),
                    UFDFraction::from(1.0_f64),
                    UFDFraction::from(1.0_f64)
                )))
            );
            for item in NON_ZERO_CHROMAS.iter() {
                let chroma = UFDFraction::from(*item);
                let range = hue.sum_range_for_chroma(chroma).unwrap();
                let max_chroma = hue.max_chroma_for_sum(range.shade_min()).unwrap();
                assert_approx_eq!(max_chroma.proportion(), chroma);
                let max_chroma = hue.max_chroma_for_sum(range.tint_max()).unwrap();
                assert_approx_eq!(max_chroma.proportion(), chroma, 0.000_000_000_000_001);
            }
        }
        for hue in &Hue::SECONDARIES {
            assert!(hue
                .sum_range_for_chroma(UFDFraction::from(0.0_f64))
                .is_none());
            assert_eq!(
                hue.sum_range_for_chroma(UFDFraction::from(1.0_f64)),
                Some(SumRange((
                    UFDFraction::TWO,
                    UFDFraction::TWO,
                    UFDFraction::TWO
                )))
            );
            for item in NON_ZERO_CHROMAS.iter() {
                let chroma = UFDFraction::from(*item);
                let range = hue.sum_range_for_chroma(chroma).unwrap();
                let max_chroma = hue.max_chroma_for_sum(range.shade_min()).unwrap();
                assert_approx_eq!(max_chroma.proportion(), chroma);
                let max_chroma = hue.max_chroma_for_sum(range.tint_max()).unwrap();
                assert_approx_eq!(max_chroma.proportion(), chroma, 0.000000000000001);
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
            for item in SECOND_VALUES.iter() {
                let other = UFDFraction::from(*item);
                let hue = Hue::Sextant(SextantHue(*sextant, other));
                assert!(hue
                    .sum_range_for_chroma(UFDFraction::from(0.0_f64))
                    .is_none());
                assert_eq!(
                    hue.sum_range_for_chroma(UFDFraction::from(1.0_f64)),
                    Some(SumRange((
                        UFDFraction::from(1.0_f64) + other,
                        UFDFraction::from(1.0_f64) + other,
                        UFDFraction::from(1.0_f64) + other
                    )))
                );
            }
        }
    }

    #[test]
    fn primary_max_chroma_rgbs() {
        for (hue, expected_rgb) in Hue::PRIMARIES.iter().zip(RGB::<f64>::PRIMARIES.iter()) {
            assert_eq!(
                hue.max_chroma_rgb_for_sum(UFDFraction::from(1.0_f64))
                    .unwrap(),
                *expected_rgb
            );
            assert!(hue
                .max_chroma_rgb_for_sum::<f64>(UFDFraction::from(0.0_f64))
                .is_none());
            assert!(hue
                .max_chroma_rgb_for_sum::<f64>(UFDFraction::from(3.0_f64))
                .is_none());
            for sum in [
                UFDFraction::from(0.0001_f64),
                UFDFraction::from(0.25_f64),
                UFDFraction::from(0.5_f64),
                UFDFraction::from(0.75_f64),
                UFDFraction::from(0.9999_f64),
            ]
            .iter()
            {
                let mut array = [UFDFraction::ZERO, UFDFraction::ZERO, UFDFraction::ZERO];
                array[hue.indices().0 as usize] = (*sum).into();
                let expected: RGB<f64> = array.into();
                assert_eq!(hue.max_chroma_rgb_for_sum::<f64>(*sum).unwrap(), expected);
            }
            for sum in [
                UFDFraction::from(2.0001_f64),
                UFDFraction::from(2.25_f64),
                UFDFraction::from(2.5_f64),
                UFDFraction::from(2.75_f64),
                UFDFraction::from(2.9999_f64),
            ]
            .iter()
            {
                let mut array = [UFDFraction::ONE, UFDFraction::ONE, UFDFraction::ONE];
                array[hue.indices().1 as usize] =
                    ((*sum - UFDFraction::from(1.0_f64)) / UFDFraction::from(2.0_f64)).into();
                array[hue.indices().2 as usize] =
                    ((*sum - UFDFraction::from(1.0_f64)) / UFDFraction::from(2.0_f64)).into();
                let expected: RGB<f64> = array.into();
                assert_eq!(hue.max_chroma_rgb_for_sum::<f64>(*sum).unwrap(), expected);
            }
        }
    }

    #[test]
    fn secondary_max_chroma_rgbs() {
        for (hue, expected_rgb) in Hue::SECONDARIES.iter().zip(RGB::<f64>::SECONDARIES.iter()) {
            assert_eq!(
                hue.max_chroma_rgb_for_sum::<f64>(UFDFraction::from(2.0_f64))
                    .unwrap(),
                *expected_rgb
            );
            assert!(hue
                .max_chroma_rgb_for_sum::<f64>(UFDFraction::from(0.0_f64))
                .is_none());
            assert!(hue
                .max_chroma_rgb_for_sum::<f64>(UFDFraction::from(3.0_f64))
                .is_none());
            for sum in [
                UFDFraction::from(0.0001_f64),
                UFDFraction::from(0.25_f64),
                UFDFraction::from(0.5_f64),
                UFDFraction::from(0.75_f64),
                UFDFraction::from(1.0_f64),
                UFDFraction::from(1.5_f64),
                UFDFraction::from(1.9999_f64),
            ]
            .iter()
            {
                let mut array = [UFDFraction::ZERO, UFDFraction::ZERO, UFDFraction::ZERO];
                array[hue.indices().0 as usize] = (*sum / UFDFraction::from(2.0_f64)).into();
                array[hue.indices().1 as usize] = (*sum / UFDFraction::from(2.0_f64)).into();
                let expected: RGB<f64> = array.into();
                assert_eq!(hue.max_chroma_rgb_for_sum::<f64>(*sum).unwrap(), expected);
            }
            for sum in [
                UFDFraction::from(2.0001_f64),
                UFDFraction::from(2.25_f64),
                UFDFraction::from(2.5_f64),
                UFDFraction::from(2.75_f64),
                UFDFraction::from(2.9999_f64),
            ]
            .iter()
            {
                let mut array = [UFDFraction::ONE, UFDFraction::ONE, UFDFraction::ONE];
                array[hue.indices().2 as usize] = (*sum - UFDFraction::from(2.0_f64)).into();
                let expected: RGB<f64> = array.into();
                assert_eq!(hue.max_chroma_rgb_for_sum::<f64>(*sum).unwrap(), expected);
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
            for item in SECOND_VALUES.iter() {
                let second = UFDFraction::from(*item);
                let sextant_hue = SextantHue(*sextant, second);
                let hue = Hue::Sextant(sextant_hue);
                assert!(hue
                    .max_chroma_rgb_for_sum::<f64>(UFDFraction::from(0.0_f64))
                    .is_none());
                assert!(hue
                    .max_chroma_rgb_for_sum::<f64>(UFDFraction::from(3.0_f64))
                    .is_none());
                println!(
                    "hue: {:?} MAX_CHROMA_RGB: {:?}",
                    hue,
                    hue.max_chroma_rgb::<f64>()
                );
                for item in VALID_OTHER_SUMS.iter() {
                    let sum = UFDFraction::from(*item);
                    let rgb = hue.max_chroma_rgb_for_sum::<f64>(sum).unwrap();
                    //assert_approx_eq!(rgb.sum(), *sum);
                    if sum < UFDFraction::THREE - second {
                        if let Ok(Hue::Sextant(sextant_hue_out)) = Hue::try_from(&rgb) {
                            assert_eq!(sextant_hue.0, sextant_hue_out.0);
                        //assert_approx_eq!(sextant_hue.1, sextant_hue_out.1, 0.000000000001);
                        } else {
                            panic!("\"Sextant\"  Hue variant expected");
                        }
                    } else {
                        // sum is too big for this hue so drifting towards nearest secondary
                        use CMYHue::*;
                        use Sextant::*;
                        let hue_out = Hue::try_from(&rgb).unwrap();
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
        for (hue, expected_rgb) in Hue::PRIMARIES.iter().zip(RGB::<f64>::PRIMARIES.iter()) {
            println!("{:?} : {:?}", hue, expected_rgb);
            assert_eq!(
                hue.min_sum_rgb_for_chroma::<f64>(UFDFraction::from(1.0_f64)),
                *expected_rgb
            );
            assert_eq!(
                hue.max_sum_rgb_for_chroma::<f64>(UFDFraction::from(1.0_f64)),
                *expected_rgb
            );
            let shade = hue.min_sum_rgb_for_chroma(UFDFraction::from(0.5_f64));
            let tint = hue.max_sum_rgb_for_chroma(UFDFraction::from(0.5_f64));
            assert!(shade.value() < tint.value());
            assert_approx_eq!(
                shade.chroma_proportion(),
                UFDFraction::from(0.5_f64),
                0.00000000001
            );
            assert_approx_eq!(
                tint.chroma_proportion(),
                UFDFraction::from(0.5_f64),
                0.00000000001
            );
            assert_approx_eq!(shade.max_chroma_rgb(), tint.max_chroma_rgb(), 0.0000001);
        }
        for (hue, expected_rgb) in Hue::SECONDARIES.iter().zip(RGB::<f64>::SECONDARIES.iter()) {
            println!("{:?} : {:?}", hue, expected_rgb);
            assert_eq!(
                hue.min_sum_rgb_for_chroma(UFDFraction::from(1.0_f64)),
                *expected_rgb
            );
            assert_eq!(
                hue.max_sum_rgb_for_chroma(UFDFraction::from(1.0_f64)),
                *expected_rgb
            );
            let shade = hue.min_sum_rgb_for_chroma(UFDFraction::from(0.5_f64));
            let tint = hue.max_sum_rgb_for_chroma(UFDFraction::from(0.5_f64));
            assert!(shade.value() < tint.value());
            assert_approx_eq!(
                shade.chroma_proportion(),
                UFDFraction::from(0.5_f64),
                0.00000000001
            );
            assert_approx_eq!(
                tint.chroma_proportion(),
                UFDFraction::from(0.5_f64),
                0.00000000001
            );
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
            for item in SECOND_VALUES.iter() {
                let second = UFDFraction::from(*item);
                let hue = Hue::Sextant(SextantHue(*sextant, second));
                assert_eq!(
                    hue.min_sum_rgb_for_chroma::<f64>(UFDFraction::from(0.0_f64)),
                    RGB::BLACK
                );
                assert_eq!(
                    hue.max_sum_rgb_for_chroma::<f64>(UFDFraction::from(0.0_f64)),
                    RGB::WHITE
                );
                for chroma in NON_ZERO_CHROMAS.iter().map(|a| UFDFraction::from(*a)) {
                    let shade = hue.min_sum_rgb_for_chroma(chroma);
                    let tint = hue.max_sum_rgb_for_chroma(chroma);
                    assert!(shade.sum() <= tint.sum());
                    assert_approx_eq!(shade.chroma_proportion(), chroma, 0.00000000001);
                    assert_approx_eq!(tint.chroma_proportion(), chroma, 0.00000000001);
                    assert_approx_eq!(shade.max_chroma_rgb(), tint.max_chroma_rgb(), 0.000_001);
                }
            }
        }
    }

    #[test]
    fn primary_rgb_for_sum_and_chroma() {
        for hue in &Hue::PRIMARIES {
            assert!(hue
                .rgb_for_sum_and_chroma::<f64>(UFDFraction::ZERO, UFDFraction::from(1.0_f64))
                .is_none());
            assert!(hue
                .rgb_for_sum_and_chroma::<f64>(UFDFraction::THREE, UFDFraction::from(1.0_f64))
                .is_none());
            assert!(hue
                .rgb_for_sum_and_chroma::<f64>(UFDFraction::ZERO, UFDFraction::from(0.0_f64))
                .is_none());
            assert!(hue
                .rgb_for_sum_and_chroma::<f64>(UFDFraction::THREE, UFDFraction::from(0.0_f64))
                .is_none());
            for item in &NON_ZERO_CHROMAS {
                let chroma = UFDFraction::from(*item);
                for item in &VALID_OTHER_SUMS {
                    let sum = UFDFraction::from(*item);
                    if let Some(rgb) = hue.rgb_for_sum_and_chroma::<f64>(sum, chroma) {
                        //assert_approx_eq!(rgb.sum(), *sum, 0.000_000_000_1);
                        //assert_approx_eq!(rgb.chroma(), *chroma, 0.000_000_000_1);
                        assert_approx_eq!(Hue::try_from(&rgb).unwrap(), hue);
                    } else {
                        let range = hue.sum_range_for_chroma(chroma).unwrap();
                        println!("{:?}, {:?}, {:?} : {:?}", *hue, sum, chroma, range);
                        assert!(range.compare_sum(sum).is_failure());
                    }
                }
            }
        }
    }

    #[test]
    fn secondary_rgb_for_sum_and_chroma() {
        for hue in &Hue::SECONDARIES {
            assert!(hue
                .rgb_for_sum_and_chroma::<f64>(UFDFraction::ZERO, UFDFraction::from(1.0_f64))
                .is_none());
            assert!(hue
                .rgb_for_sum_and_chroma::<f64>(UFDFraction::THREE, UFDFraction::from(1.0_f64))
                .is_none());
            assert!(hue
                .rgb_for_sum_and_chroma::<f64>(UFDFraction::ZERO, UFDFraction::from(0.0_f64))
                .is_none());
            assert!(hue
                .rgb_for_sum_and_chroma::<f64>(UFDFraction::THREE, UFDFraction::from(0.0_f64))
                .is_none());
            for item in &NON_ZERO_CHROMAS {
                let chroma = UFDFraction::from(*item);
                for item in &VALID_OTHER_SUMS {
                    let sum = UFDFraction::from(*item);
                    if let Some(rgb) = hue.rgb_for_sum_and_chroma::<f64>(sum, chroma) {
                        assert_approx_eq!(rgb.sum(), sum, 0.000_000_1);
                        assert_approx_eq!(rgb.chroma_proportion(), chroma, 0.000_000_1);
                        assert_approx_eq!(Hue::try_from(&rgb).unwrap(), hue);
                    } else {
                        let range = hue.sum_range_for_chroma(chroma).unwrap();
                        assert!(range.compare_sum(sum).is_failure());
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
            for second in SECOND_VALUES.iter().map(|a| UFDFraction::from(*a)) {
                let sextant_hue = SextantHue(*sextant, second);
                let hue = Hue::Sextant(sextant_hue);
                assert!(hue
                    .rgb_for_sum_and_chroma::<f64>(UFDFraction::ZERO, UFDFraction::from(1.0_f64))
                    .is_none());
                assert!(hue
                    .rgb_for_sum_and_chroma::<f64>(UFDFraction::THREE, UFDFraction::from(1.0_f64))
                    .is_none());
                assert!(hue
                    .rgb_for_sum_and_chroma::<f64>(UFDFraction::ZERO, UFDFraction::from(0.0_f64))
                    .is_none());
                assert!(hue
                    .rgb_for_sum_and_chroma::<f64>(UFDFraction::THREE, UFDFraction::from(0.0_f64))
                    .is_none());
                for chroma in NON_ZERO_CHROMAS.iter().map(|a| UFDFraction::from(*a)) {
                    let sum_range = hue.sum_range_for_chroma(chroma).unwrap();
                    for sum in VALID_OTHER_SUMS.iter().map(|a| UFDFraction::from(*a)) {
                        println!(
                            "{:?}, {:?}, {:?} :: {:?}",
                            hue,
                            sum,
                            chroma,
                            hue.sum_range_for_chroma(chroma)
                        );
                        if let Some(rgb) = hue.rgb_for_sum_and_chroma::<f64>(sum, chroma) {
                            use SumOrdering::*;
                            match sum_range.compare_sum(sum) {
                                Shade(_, _) => {
                                    assert_approx_eq!(rgb.sum(), sum, 0.000_000_001);
                                    assert_approx_eq!(rgb.chroma_proportion(), chroma, 0.000_000_1);
                                    // TODO: examine hue drift problem
                                    assert_approx_eq!(Hue::try_from(&rgb).unwrap(), hue, 0.000_001);
                                }
                                Tint(_, _) => {
                                    assert_approx_eq!(rgb.sum(), sum, 0.000_000_001);
                                    // TODO: try harder for creating tints
                                    assert_approx_eq!(rgb.chroma_proportion(), chroma, 0.000_000_1);
                                    assert_approx_eq!(
                                        Hue::try_from(&rgb).unwrap(),
                                        hue,
                                        0.000_000_1
                                    );
                                }
                                _ => (),
                            }
                        } else {
                            let range = hue.sum_range_for_chroma(chroma).unwrap();
                            assert!(range.compare_sum(sum).is_failure());
                        }
                    }
                }
            }
        }
    }
}
