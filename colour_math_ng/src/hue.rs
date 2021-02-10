// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    cmp::Ordering,
    convert::{Into, TryFrom},
    fmt::Debug,
};

use normalised_angles::Degrees;

use crate::{
    Chroma, ChromaOneRGB, Float, HueAngle, HueConstants, LightLevel, Prop, RGBConstants, Sum, RGB,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct SumRange((Sum, Sum, Sum));

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum SumOrdering {
    TooSmall,
    Shade(Sum, Sum),
    Tint(Sum, Sum),
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
    pub fn compare_sum(&self, sum: Sum) -> SumOrdering {
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

    pub fn min(&self) -> Sum {
        self.0 .0
    }

    pub fn shade_min(&self) -> Sum {
        self.0 .0
    }

    pub fn shade_max(&self) -> Sum {
        self.0 .1
    }

    pub fn crossover(&self) -> Sum {
        self.0 .1
    }

    pub fn tint_min(&self) -> Sum {
        self.0 .1
    }

    pub fn tint_max(&self) -> Sum {
        self.0 .2
    }

    pub fn max(&self) -> Sum {
        self.0 .2
    }
}

pub trait HueIfceTmp {
    fn sum_range_for_chroma(&self, chroma_value: Prop) -> Option<SumRange>;
    fn max_chroma_for_sum(&self, sum: Sum) -> Option<Chroma>;

    fn max_chroma_rgb<T: LightLevel>(&self) -> RGB<T>;
    fn max_chroma_rgb_for_sum<T: LightLevel>(&self, sum: Sum) -> Option<RGB<T>>;
    fn min_sum_rgb_for_chroma<T: LightLevel>(&self, chroma_value: Prop) -> RGB<T>;
    fn max_sum_rgb_for_chroma<T: LightLevel>(&self, chroma_value: Prop) -> RGB<T>;
    fn rgb_for_sum_and_chroma<T: LightLevel>(&self, sum: Sum, chroma_value: Prop)
        -> Option<RGB<T>>;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord)]
pub enum RGBHue {
    Red = 5,
    Green = 9,
    Blue = 1,
}

impl RGBHue {
    fn make_rgb<T: LightLevel>(&self, components: (Prop, Prop)) -> RGB<T> {
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
    fn sum_range_for_chroma(&self, chroma: Prop) -> Option<SumRange> {
        if chroma == Prop::ZERO {
            None
        } else {
            Some(SumRange((
                chroma.into(),
                Sum::ONE,
                (Sum::THREE - chroma * 2),
            )))
        }
    }

    fn max_chroma_for_sum(&self, sum: Sum) -> Option<Chroma> {
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        if sum == Sum::ZERO || sum == Sum::THREE {
            None
        } else if sum < Sum::ONE {
            Some(Chroma::Shade(sum.into()))
        } else if sum > Sum::ONE {
            Some(Chroma::Tint((Sum::THREE - sum) / 2))
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

    fn max_chroma_rgb_for_sum<T: LightLevel>(&self, sum: Sum) -> Option<RGB<T>> {
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        if sum == Sum::ZERO || sum == Sum::THREE {
            None
        } else {
            if sum <= Sum::ONE {
                Some(self.make_rgb((sum.into(), Prop::ZERO)))
            } else {
                Some(self.make_rgb((Prop::ONE, ((sum - Sum::ONE) / 2))))
            }
        }
    }

    fn min_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Prop) -> RGB<T> {
        // TODO: Needs major revision taking into account Shade/Tint
        if chroma == Prop::ZERO {
            RGB::BLACK
        } else if chroma == Prop::ONE {
            self.max_chroma_rgb()
        } else {
            self.make_rgb((chroma, Prop::ZERO))
        }
    }

    fn max_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Prop) -> RGB<T> {
        // TODO: Needs major revision taking into account Shade/Tint
        if chroma == Prop::ZERO {
            RGB::WHITE
        } else if chroma == Prop::ONE {
            self.max_chroma_rgb()
        } else {
            self.make_rgb((Prop::ONE, Prop::ONE - chroma))
        }
    }

    fn rgb_for_sum_and_chroma<T: LightLevel>(&self, sum: Sum, chroma: Prop) -> Option<RGB<T>> {
        // TODO: Needs major revision taking into account Shade/Tint
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        let sum_range = self.sum_range_for_chroma(chroma)?;
        if sum_range.compare_sum(sum).is_success() {
            let other = (sum - chroma) / 3;
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
    fn make_rgb<T: LightLevel>(&self, components: (Prop, Prop)) -> RGB<T> {
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
    fn sum_range_for_chroma(&self, chroma: Prop) -> Option<SumRange> {
        if chroma == Prop::ZERO {
            None
        } else {
            Some(SumRange((chroma * 2, Sum::TWO, Sum::THREE - chroma)))
        }
    }

    fn max_chroma_for_sum(&self, sum: Sum) -> Option<Chroma> {
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        if sum == Sum::ZERO || sum == Sum::THREE {
            None
        } else if sum < Sum::TWO {
            Some(Chroma::Shade(sum / 2))
        } else if sum > Sum::TWO {
            Some(Chroma::Tint((Sum::THREE - sum).into()))
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

    fn max_chroma_rgb_for_sum<T: LightLevel>(&self, sum: Sum) -> Option<RGB<T>> {
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        if sum == Sum::ZERO || sum == Sum::THREE {
            None
        } else if sum < Sum::TWO {
            Some(self.make_rgb(((sum / 2), Prop::ZERO)))
        } else if sum > Sum::TWO {
            Some(self.make_rgb((Prop::ONE, (sum - Sum::TWO).into())))
        } else {
            Some(self.max_chroma_rgb())
        }
    }

    fn min_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Prop) -> RGB<T> {
        // TODO: Needs major revision taking into account Shade/Tint

        if chroma == Prop::ZERO {
            RGB::BLACK
        } else if chroma == Prop::ONE {
            self.max_chroma_rgb()
        } else {
            self.make_rgb((chroma, Prop::ZERO))
        }
    }

    fn max_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Prop) -> RGB<T> {
        // TODO: Needs major revision taking into account Shade/Tint

        if chroma == Prop::ZERO {
            RGB::WHITE
        } else if chroma == Prop::ONE {
            self.max_chroma_rgb()
        } else {
            self.make_rgb((Prop::ONE, Prop::ONE - chroma))
        }
    }

    fn rgb_for_sum_and_chroma<T: LightLevel>(&self, sum: Sum, chroma: Prop) -> Option<RGB<T>> {
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        let sum_range = self.sum_range_for_chroma(chroma)?;
        if sum_range.compare_sum(sum).is_success() {
            // TODO: reassess this calculation
            Some(self.make_rgb(((sum + chroma) / 3, (sum - chroma * 2) / 3)))
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
pub struct SextantHue(Sextant, Prop);

impl Eq for SextantHue {}

impl SextantHue {
    fn make_rgb<T: LightLevel>(&self, components: (Prop, Prop, Prop)) -> RGB<T> {
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

#[cfg(test)]
impl SextantHue {
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
        let [red, green, blue] = <[Prop; 3]>::from(*arg.1);
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

impl<T: Float + From<Prop> + Copy> HueAngle<T> for SextantHue {
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
    fn sum_range_for_chroma(&self, chroma: Prop) -> Option<SumRange> {
        if chroma == Prop::ZERO {
            None
        } else {
            let max_c_sum = (Prop::ONE + self.1).min(Sum::TWO);
            if chroma == Prop::ONE {
                Some(SumRange((max_c_sum, max_c_sum, max_c_sum)))
            } else {
                let min = max_c_sum * chroma;
                let max = Sum::THREE - (Sum::TWO - self.1) * chroma;
                Some(SumRange((min, max_c_sum, max)))
            }
        }
    }

    fn max_chroma_for_sum(&self, sum: Sum) -> Option<Chroma> {
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        if sum == Sum::ZERO || sum == Sum::THREE {
            None
        } else {
            match sum.cmp(&(Prop::ONE + self.1)) {
                Ordering::Less => {
                    let temp = sum / (Prop::ONE + self.1);
                    Some(Chroma::Shade(temp))
                }
                Ordering::Greater => {
                    let temp = (Sum::THREE - sum) / (Sum::TWO - self.1);
                    Some(Chroma::Tint(temp))
                }
                Ordering::Equal => Some(Chroma::ONE),
            }
        }
    }

    fn max_chroma_rgb<T: LightLevel>(&self) -> RGB<T> {
        self.make_rgb((Prop::ONE, self.1, Prop::ZERO))
    }

    fn max_chroma_rgb_for_sum<T: LightLevel>(&self, sum: Sum) -> Option<RGB<T>> {
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        // TODO: make hue drift an error
        if sum == Sum::ZERO || sum == Sum::THREE {
            None
        } else {
            let max_chroma_sum = Sum::ONE + self.1;
            if sum == max_chroma_sum {
                Some(self.max_chroma_rgb())
            } else {
                let components = if sum < max_chroma_sum {
                    let first = sum / max_chroma_sum;
                    (first, first * self.1, Prop::ZERO)
                } else {
                    let temp = sum - Prop::ONE;
                    let second = (temp + self.1) / 2;
                    (Prop::ONE, second, (temp - second).into())
                };
                Some(self.make_rgb(components))
            }
        }
    }

    fn min_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Prop) -> RGB<T> {
        if chroma == Prop::ZERO {
            RGB::BLACK
        } else if chroma == Prop::ONE {
            self.max_chroma_rgb()
        } else {
            self.make_rgb((chroma, self.1 * chroma, Prop::ZERO))
        }
    }

    fn max_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Prop) -> RGB<T> {
        if chroma == Prop::ZERO {
            RGB::WHITE
        } else if chroma == Prop::ONE {
            self.max_chroma_rgb()
        } else {
            let third = Prop::ONE - chroma;
            let second = chroma * self.1 + third;
            self.make_rgb((Prop::ONE, second.into(), third))
        }
    }

    fn rgb_for_sum_and_chroma<T: LightLevel>(&self, sum: Sum, chroma: Prop) -> Option<RGB<T>> {
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);

        let sum_range = self.sum_range_for_chroma(chroma)?;
        if sum_range.compare_sum(sum).is_success() {
            let delta = (sum - sum_range.shade_min()) / 3;
            let first = chroma + delta;
            let second = chroma * self.1 + delta;
            Some(self.make_rgb((first.into(), second.into(), delta)))
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
        let [red, green, blue] = <[Prop; 3]>::from(*rgb);
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

impl<T: Float + From<Prop>> HueAngle<T> for Hue {
    fn hue_angle(&self) -> Degrees<T> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.hue_angle(),
            Self::Secondary(cmy_hue) => cmy_hue.hue_angle(),
            Self::Sextant(sextant_hue) => sextant_hue.hue_angle(),
        }
    }
}

impl HueIfceTmp for Hue {
    fn sum_range_for_chroma(&self, chroma: Prop) -> Option<SumRange> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.sum_range_for_chroma(chroma),
            Self::Secondary(cmy_hue) => cmy_hue.sum_range_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.sum_range_for_chroma(chroma),
        }
    }

    fn max_chroma_for_sum(&self, sum: Sum) -> Option<Chroma> {
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

    fn max_chroma_rgb_for_sum<T: LightLevel>(&self, sum: Sum) -> Option<RGB<T>> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.max_chroma_rgb_for_sum(sum),
            Self::Secondary(cmy_hue) => cmy_hue.max_chroma_rgb_for_sum(sum),
            Self::Sextant(sextant_hue) => sextant_hue.max_chroma_rgb_for_sum(sum),
        }
    }

    fn min_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Prop) -> RGB<T> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.min_sum_rgb_for_chroma(chroma),
            Self::Secondary(cmy_hue) => cmy_hue.min_sum_rgb_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.min_sum_rgb_for_chroma(chroma),
        }
    }

    fn max_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Prop) -> RGB<T> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.max_sum_rgb_for_chroma(chroma),
            Self::Secondary(cmy_hue) => cmy_hue.max_sum_rgb_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.max_sum_rgb_for_chroma(chroma),
        }
    }

    fn rgb_for_sum_and_chroma<T: LightLevel>(&self, sum: Sum, chroma: Prop) -> Option<RGB<T>> {
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
}

#[cfg(test)]
impl Hue {
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
            assert_eq!(Hue::try_from(&(*rgb * Prop::from(0.5))), Ok(*hue));
        }
        for (rgb, hue) in RGB::<f64>::SECONDARIES.iter().zip(Hue::SECONDARIES.iter()) {
            assert_eq!(Hue::try_from(rgb), Ok(*hue));
            assert_eq!(Hue::try_from(&(*rgb * Prop::from(0.5))), Ok(*hue));
        }
        for (array, sextant, second) in &[
            (
                [Prop::ONE, Prop::from(0.5_f64), Prop::ZERO],
                Sextant::RedYellow,
                Prop::from(0.5),
            ),
            (
                [Prop::ZERO, Prop::from(0.25_f64), Prop::from(0.5_f64)],
                Sextant::BlueCyan,
                Prop::from(0.5),
            ),
            (
                [Prop::from(0.2_f64), Prop::ZERO, Prop::from(0.4_f64)],
                Sextant::BlueMagenta,
                Prop::from(0.5),
            ),
            (
                [Prop::from(0.5_f64), Prop::ZERO, Prop::ONE],
                Sextant::BlueMagenta,
                Prop::from(0.5),
            ),
            (
                [Prop::ONE, Prop::ZERO, Prop::from(0.5_f64)],
                Sextant::RedMagenta,
                Prop::from(0.5),
            ),
            (
                [Prop::from(0.5_f64), Prop::ONE, Prop::ZERO],
                Sextant::GreenYellow,
                Prop::from(0.5),
            ),
            (
                [Prop::ZERO, Prop::ONE, Prop::from(0.5_f64)],
                Sextant::GreenCyan,
                Prop::from(0.5),
            ),
        ] {
            let rgb = RGB::<f64>::from([
                Prop::from(array[0]),
                Prop::from(array[1]),
                Prop::from(array[2]),
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
                [Prop::ONE, Prop::from(0.5_f64), Prop::ZERO],
                Sextant::RedYellow,
                Prop::from(0.5_f64),
            ),
            (
                [Prop::ZERO, Prop::from(0.5_f64), Prop::ONE],
                Sextant::BlueCyan,
                Prop::from(0.5_f64),
            ),
            (
                [Prop::from(0.5_f64), Prop::ZERO, Prop::ONE],
                Sextant::BlueMagenta,
                Prop::from(0.5_f64),
            ),
            (
                [Prop::ONE, Prop::ZERO, Prop::from(0.5_f64)],
                Sextant::RedMagenta,
                Prop::from(0.5_f64),
            ),
            (
                [Prop::from(0.5_f64), Prop::ONE, Prop::ZERO],
                Sextant::GreenYellow,
                Prop::from(0.5_f64),
            ),
            (
                [Prop::ZERO, Prop::ONE, Prop::from(0.5_f64)],
                Sextant::GreenCyan,
                Prop::from(0.5_f64),
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
                Prop::from(0.5_f64),
                Degrees::<f64>::DEG_30,
            ),
            (
                Sextant::BlueCyan,
                Prop::from(0.5_f64),
                Degrees::<f64>::NEG_DEG_150,
            ),
            (
                Sextant::BlueMagenta,
                Prop::from(0.5_f64),
                Degrees::<f64>::NEG_DEG_90,
            ),
            (
                Sextant::RedMagenta,
                Prop::from(0.5_f64),
                Degrees::<f64>::NEG_DEG_30,
            ),
            (
                Sextant::GreenYellow,
                Prop::from(0.5_f64),
                Degrees::<f64>::DEG_90,
            ),
            (
                Sextant::GreenCyan,
                Prop::from(0.5_f64),
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
            assert!(hue.sum_range_for_chroma(Prop::ZERO).is_none());
            assert_eq!(
                hue.sum_range_for_chroma(Prop::ONE),
                Some(SumRange((Sum::ONE, Sum::ONE, Sum::ONE)))
            );
            for item in NON_ZERO_CHROMAS.iter() {
                let chroma = Prop::from(*item);
                let range = hue.sum_range_for_chroma(chroma).unwrap();
                let max_chroma = hue.max_chroma_for_sum(range.shade_min()).unwrap();
                assert_approx_eq!(max_chroma.proportion(), chroma);
                let max_chroma = hue.max_chroma_for_sum(range.tint_max()).unwrap();
                assert_approx_eq!(max_chroma.proportion(), chroma, 0.000_000_000_000_001);
            }
        }
        for hue in &Hue::SECONDARIES {
            assert!(hue.sum_range_for_chroma(Prop::ZERO).is_none());
            assert_eq!(
                hue.sum_range_for_chroma(Prop::ONE),
                Some(SumRange((Sum::TWO, Sum::TWO, Sum::TWO)))
            );
            for item in NON_ZERO_CHROMAS.iter() {
                let chroma = Prop::from(*item);
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
                let other = Prop::from(*item);
                let hue = Hue::Sextant(SextantHue(*sextant, other));
                assert!(hue.sum_range_for_chroma(Prop::ZERO).is_none());
                assert_eq!(
                    hue.sum_range_for_chroma(Prop::ONE),
                    Some(SumRange((
                        Sum::ONE + other,
                        Sum::ONE + other,
                        Sum::ONE + other
                    )))
                );
            }
        }
    }

    #[test]
    fn primary_max_chroma_rgbs() {
        for (hue, expected_rgb) in Hue::PRIMARIES.iter().zip(RGB::<f64>::PRIMARIES.iter()) {
            assert_eq!(hue.max_chroma_rgb_for_sum(Sum::ONE).unwrap(), *expected_rgb);
            assert!(hue.max_chroma_rgb_for_sum::<f64>(Sum::ZERO).is_none());
            assert!(hue.max_chroma_rgb_for_sum::<f64>(Sum::THREE).is_none());
            for sum in [
                Sum::from(0.0001_f64),
                Sum::from(0.25_f64),
                Sum::from(0.5_f64),
                Sum::from(0.75_f64),
                Sum::from(0.9999_f64),
            ]
            .iter()
            {
                let mut array = [Prop::ZERO, Prop::ZERO, Prop::ZERO];
                array[hue.indices().0 as usize] = (*sum).into();
                let expected: RGB<f64> = array.into();
                assert_eq!(hue.max_chroma_rgb_for_sum::<f64>(*sum).unwrap(), expected);
            }
            for sum in [
                Sum::from(2.0001_f64),
                Sum::from(2.25_f64),
                Sum::from(2.5_f64),
                Sum::from(2.75_f64),
                Sum::from(2.9999_f64),
            ]
            .iter()
            {
                let mut array = [Prop::ONE, Prop::ONE, Prop::ONE];
                array[hue.indices().1 as usize] = ((*sum - Sum::ONE) / 2).into();
                array[hue.indices().2 as usize] = ((*sum - Sum::ONE) / 2).into();
                let expected: RGB<f64> = array.into();
                assert_eq!(hue.max_chroma_rgb_for_sum::<f64>(*sum).unwrap(), expected);
            }
        }
    }

    #[test]
    fn secondary_max_chroma_rgbs() {
        for (hue, expected_rgb) in Hue::SECONDARIES.iter().zip(RGB::<f64>::SECONDARIES.iter()) {
            assert_approx_eq!(
                hue.max_chroma_rgb_for_sum::<f64>(Sum::from(2.0_f64))
                    .unwrap(),
                *expected_rgb
            );
            assert!(hue.max_chroma_rgb_for_sum::<f64>(Sum::ZERO).is_none());
            assert!(hue.max_chroma_rgb_for_sum::<f64>(Sum::THREE).is_none());
            for sum in [
                Sum::from(0.0001_f64),
                Sum::from(0.25_f64),
                Sum::from(0.5_f64),
                Sum::from(0.75_f64),
                Sum::ONE,
                Sum::from(1.5_f64),
                Sum::from(1.9999_f64),
            ]
            .iter()
            {
                let mut array = [Prop::ZERO, Prop::ZERO, Prop::ZERO];
                array[hue.indices().0 as usize] = (*sum / 2).into();
                array[hue.indices().1 as usize] = (*sum / 2).into();
                let expected: RGB<f64> = array.into();
                assert_eq!(hue.max_chroma_rgb_for_sum::<f64>(*sum).unwrap(), expected);
            }
            for sum in [
                Sum::from(2.0001_f64),
                Sum::from(2.25_f64),
                Sum::from(2.5_f64),
                Sum::from(2.75_f64),
                Sum::from(2.9999_f64),
            ]
            .iter()
            {
                let mut array = [Prop::ONE, Prop::ONE, Prop::ONE];
                array[hue.indices().2 as usize] = (*sum - Sum::from(2.0_f64)).into();
                let expected: RGB<f64> = array.into();
                assert_approx_eq!(hue.max_chroma_rgb_for_sum::<f64>(*sum).unwrap(), expected);
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
                let second = Prop::from(*item);
                let sextant_hue = SextantHue(*sextant, second);
                let hue = Hue::Sextant(sextant_hue);
                assert!(hue.max_chroma_rgb_for_sum::<f64>(Sum::ZERO).is_none());
                assert!(hue.max_chroma_rgb_for_sum::<f64>(Sum::THREE).is_none());
                for item in VALID_OTHER_SUMS.iter() {
                    println!(
                        "hue: {:?} MAX_CHROMA_RGB: {:?} sum: {:?} second: {:?}",
                        hue,
                        hue.max_chroma_rgb::<f64>(),
                        item,
                        f64::from(second)
                    );
                    let sum = Sum::from(*item);
                    let rgb = hue.max_chroma_rgb_for_sum::<f64>(sum).unwrap();
                    //assert_approx_eq!(rgb.sum(), *sum);
                    if sum < Sum::THREE - second {
                        if let Ok(Hue::Sextant(sextant_hue_out)) = Hue::try_from(&rgb) {
                            assert_eq!(sextant_hue.0, sextant_hue_out.0);
                        //assert_approx_eq!(sextant_hue.1, sextant_hue_out.1, 0.000000000001);
                        } else {
                            assert!(rgb.is_grey());
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
            assert_eq!(hue.min_sum_rgb_for_chroma::<f64>(Prop::ONE), *expected_rgb);
            assert_eq!(hue.max_sum_rgb_for_chroma::<f64>(Prop::ONE), *expected_rgb);
            let shade = hue.min_sum_rgb_for_chroma(Prop::from(0.5_f64));
            let tint = hue.max_sum_rgb_for_chroma(Prop::from(0.5_f64));
            assert!(shade.value() < tint.value());
            assert_approx_eq!(
                shade.chroma_proportion(),
                Prop::from(0.5_f64),
                0.00000000001
            );
            assert_approx_eq!(tint.chroma_proportion(), Prop::from(0.5_f64), 0.00000000001);
            assert_approx_eq!(shade.max_chroma_rgb(), tint.max_chroma_rgb(), 0.0000001);
        }
        for (hue, expected_rgb) in Hue::SECONDARIES.iter().zip(RGB::<f64>::SECONDARIES.iter()) {
            println!("{:?} : {:?}", hue, expected_rgb);
            assert_eq!(hue.min_sum_rgb_for_chroma(Prop::ONE), *expected_rgb);
            assert_eq!(hue.max_sum_rgb_for_chroma(Prop::ONE), *expected_rgb);
            let shade = hue.min_sum_rgb_for_chroma(Prop::from(0.5_f64));
            let tint = hue.max_sum_rgb_for_chroma(Prop::from(0.5_f64));
            assert!(shade.value() < tint.value());
            assert_approx_eq!(
                shade.chroma_proportion(),
                Prop::from(0.5_f64),
                0.00000000001
            );
            assert_approx_eq!(tint.chroma_proportion(), Prop::from(0.5_f64), 0.00000000001);
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
                let second = Prop::from(*item);
                let hue = Hue::Sextant(SextantHue(*sextant, second));
                assert_eq!(hue.min_sum_rgb_for_chroma::<f64>(Prop::ZERO), RGB::BLACK);
                assert_eq!(hue.max_sum_rgb_for_chroma::<f64>(Prop::ZERO), RGB::WHITE);
                for chroma in NON_ZERO_CHROMAS.iter().map(|a| Prop::from(*a)) {
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
                .rgb_for_sum_and_chroma::<f64>(Sum::ZERO, Prop::ONE)
                .is_none());
            assert!(hue
                .rgb_for_sum_and_chroma::<f64>(Sum::THREE, Prop::ONE)
                .is_none());
            assert!(hue
                .rgb_for_sum_and_chroma::<f64>(Sum::ZERO, Prop::ZERO)
                .is_none());
            assert!(hue
                .rgb_for_sum_and_chroma::<f64>(Sum::THREE, Prop::ZERO)
                .is_none());
            for item in &NON_ZERO_CHROMAS {
                let chroma = Prop::from(*item);
                for item in &VALID_OTHER_SUMS {
                    let sum = Sum::from(*item);
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
                .rgb_for_sum_and_chroma::<f64>(Sum::ZERO, Prop::ONE)
                .is_none());
            assert!(hue
                .rgb_for_sum_and_chroma::<f64>(Sum::THREE, Prop::ONE)
                .is_none());
            assert!(hue
                .rgb_for_sum_and_chroma::<f64>(Sum::ZERO, Prop::ZERO)
                .is_none());
            assert!(hue
                .rgb_for_sum_and_chroma::<f64>(Sum::THREE, Prop::ZERO)
                .is_none());
            for item in &NON_ZERO_CHROMAS {
                let chroma = Prop::from(*item);
                for item in &VALID_OTHER_SUMS {
                    let sum = Sum::from(*item);
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
            for second in SECOND_VALUES.iter().map(|a| Prop::from(*a)) {
                let sextant_hue = SextantHue(*sextant, second);
                let hue = Hue::Sextant(sextant_hue);
                assert!(hue
                    .rgb_for_sum_and_chroma::<f64>(Sum::ZERO, Prop::ONE)
                    .is_none());
                assert!(hue
                    .rgb_for_sum_and_chroma::<f64>(Sum::THREE, Prop::ONE)
                    .is_none());
                assert!(hue
                    .rgb_for_sum_and_chroma::<f64>(Sum::ZERO, Prop::ZERO)
                    .is_none());
                assert!(hue
                    .rgb_for_sum_and_chroma::<f64>(Sum::THREE, Prop::ZERO)
                    .is_none());
                for chroma in NON_ZERO_CHROMAS.iter().map(|a| Prop::from(*a)) {
                    let sum_range = hue.sum_range_for_chroma(chroma).unwrap();
                    for sum in VALID_OTHER_SUMS.iter().map(|a| Sum::from(*a)) {
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
