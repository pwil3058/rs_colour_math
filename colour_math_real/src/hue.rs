// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    cmp::Ordering,
    convert::{From, Into, TryFrom},
    fmt::Debug,
    ops::{Add, Sub},
};

use num_traits_plus::{debug_assert_approx_eq, float_plus::FloatPlus};

pub mod angle;

use crate::{
    attributes::{Chroma, Warmth},
    debug::{AbsDiff, ApproxEq, PropDiff},
    //hcv::HCV,
    hue::angle::Angle,
    real::{IntoProp, Prop, Real},
    rgb::RGB,
    ColourBasics,
    HueConstants,
    LightLevel,
    RGBConstants,
};

pub(crate) trait HueBasics: Copy + Debug + Sized + Into<Hue> {
    fn sum_for_max_chroma(&self) -> Real;

    fn min_sum_for_chroma_prop(&self, c_prop: Prop) -> Option<Real> {
        match c_prop {
            Prop::ZERO => None,
            c_prop => match self.sum_for_max_chroma() * c_prop {
                Real::ZERO => None,
                sum => Some(sum),
            },
        }
    }

    fn max_sum_for_chroma_prop(&self, c_prop: Prop) -> Option<Real> {
        match c_prop {
            Prop::ZERO => None,
            c_prop => match Real::THREE - (Real::THREE - self.sum_for_max_chroma()) * c_prop {
                Real::ZERO => None,
                sum => Some(sum),
            },
        }
    }

    fn sum_range_for_chroma_prop(&self, c_prop: Prop) -> Option<(Real, Real)> {
        let min = self.min_sum_for_chroma_prop(c_prop)?;
        let max = self.max_sum_for_chroma_prop(c_prop)?;
        Some((min, max))
    }

    fn sum_range_for_chroma(&self, chroma: Chroma) -> Option<(Real, Real)> {
        debug_assert!(chroma.is_valid());
        let (min_sum, max_sum) = self.sum_range_for_chroma_prop(chroma.into_prop())?;
        match chroma {
            Chroma::ZERO => None,
            Chroma::Neither(_) => Some((self.sum_for_max_chroma(), self.sum_for_max_chroma())),
            Chroma::Shade(_) => Some((min_sum, self.sum_for_max_chroma() - Real(f64::EPSILON))),
            Chroma::Tint(_) => Some((self.sum_for_max_chroma() + Real(f64::EPSILON), max_sum)),
        }
    }

    fn max_chroma_prop_for_sum(&self, sum: Real) -> Option<Prop> {
        Some(self.max_chroma_for_sum(sum)?.into_prop())
    }

    fn max_chroma_for_sum(&self, sum: Real) -> Option<Chroma> {
        debug_assert!(sum.is_valid_sum(), "sum: {:?}", sum);
        if sum.is_hue_valid() {
            match sum.cmp(&self.sum_for_max_chroma()) {
                Ordering::Equal => Some(Chroma::ONE),
                Ordering::Less => {
                    let temp = sum / self.sum_for_max_chroma();
                    Some(Chroma::Shade(temp.into()))
                }
                Ordering::Greater => {
                    // NB: make sure it doesn't round to one
                    let c_prop = ((Real::THREE - sum) / (Real::THREE - self.sum_for_max_chroma()))
                        .into_prop()
                        .min(Prop::ALMOST_ONE);
                    Some(Chroma::Tint(c_prop))
                }
            }
        } else {
            None
        }
    }
}

pub(crate) trait SumChromaCompatibility: HueBasics {
    fn sum_and_chroma_prop_are_compatible(&self, sum: Real, c_prop: Prop) -> bool {
        if let Some((min_sum, max_sum)) = self.sum_range_for_chroma_prop(c_prop) {
            sum >= min_sum
                && sum <= max_sum
                && (sum - self.sum_for_max_chroma() * c_prop) % Real(3.0) == Real::ZERO
        } else {
            false
        }
    }

    fn sum_and_chroma_are_compatible(&self, sum: Real, chroma: Chroma) -> bool {
        debug_assert!(chroma.is_valid());
        if let Some((min_sum, max_sum)) = self.sum_range_for_chroma(chroma) {
            sum >= min_sum
                && sum <= max_sum
                && (sum - self.sum_for_max_chroma() * chroma.into_prop()) % Real(3.0) == Real::ZERO
        } else {
            false
        }
    }
}

pub(crate) trait OrderedTriplets: HueBasics + SumChromaCompatibility {
    fn has_valid_value_order(&self, triplet: &[Prop; 3]) -> bool;
    fn has_valid_rgb_order(&self, triplet: &[Prop; 3]) -> bool;
    fn triplet_to_rgb_order(&self, triplet: &[Prop; 3]) -> [Prop; 3];

    fn rgb_ordered_triplet(&self, sum: Real, c_prop: Prop) -> Option<[Prop; 3]> {
        let triplet = self.ordered_triplet(sum, c_prop)?;
        debug_assert!(self.has_valid_value_order(&triplet));
        Some(self.triplet_to_rgb_order(&triplet))
    }

    fn ordered_triplet(&self, sum: Real, c_prop: Prop) -> Option<[Prop; 3]> {
        if sum >= self.sum_for_max_chroma() * c_prop {
            debug_assert!(self.sum_and_chroma_prop_are_compatible(sum, c_prop));
            let third = (sum - self.sum_for_max_chroma() * c_prop) / Real(3.0);
            let first = third + c_prop;
            if first.is_proportion() {
                let second = sum - first - third;
                debug_assert_eq!(first + second + third, sum);
                debug_assert_eq!(first - third, c_prop.into());
                let triplet = [first.into_prop(), second.into_prop(), third.into_prop()];
                debug_assert!(self.has_valid_value_order(&triplet));
                Some(triplet)
            } else {
                None
            }
        } else {
            None
        }
    }

    // fn ordered_triplet_to_hcv(&self, triplet: &[Prop; 3]) -> HCV {
    //     debug_assert!(self.has_valid_value_order(triplet));
    //     let sum = triplet[0] + triplet[1] + triplet[2];
    //     let c_prop = triplet[0] - triplet[2];
    //     let chroma = match sum.cmp(&self.sum_for_max_chroma()) {
    //         Ordering::Less => Chroma::Shade(c_prop),
    //         Ordering::Equal => Chroma::Neither(c_prop),
    //         Ordering::Greater => Chroma::Tint(c_prop),
    //     };
    //     HCV {
    //         hue: Some((*self).into()),
    //         chroma,
    //         sum,
    //     }
    // }
}

pub(crate) trait HueIfce:
    HueBasics + OrderedTriplets + ColourModificationHelpers + SumChromaCompatibility
{
    fn angle(&self) -> Angle;

    fn warmth_for_chroma(&self, chroma: Chroma) -> Warmth;

    fn min_sum_for_chroma(&self, chroma: Chroma) -> Option<Real> {
        debug_assert!(chroma.is_valid());
        match chroma {
            Chroma::ZERO => None,
            Chroma::Neither(_) => Some(self.sum_for_max_chroma()),
            Chroma::Shade(c_prop) => self.min_sum_for_chroma_prop(c_prop),
            Chroma::Tint(c_prop) => {
                let mut sum = self.sum_for_max_chroma() + Real(f64::EPSILON);
                while !self.sum_and_chroma_prop_are_compatible(sum, c_prop) {
                    sum = sum + Real(f64::EPSILON);
                }
                Some(sum)
            }
        }
    }

    fn max_sum_for_chroma(&self, chroma: Chroma) -> Option<Real> {
        debug_assert!(chroma.is_valid());
        match chroma {
            Chroma::ZERO => None,
            Chroma::Neither(_) => Some(self.sum_for_max_chroma()),
            Chroma::Shade(c_prop) => {
                let mut sum = self.sum_for_max_chroma() - Real(f64::EPSILON);
                while !self.sum_and_chroma_prop_are_compatible(sum, c_prop) {
                    sum = sum - Real(f64::EPSILON);
                }
                Some(sum)
            }
            Chroma::Tint(c_prop) => self.max_sum_for_chroma_prop(c_prop),
        }
    }

    fn max_chroma_rgb<T: LightLevel>(&self) -> RGB<T> {
        RGB::<T>::MEDIUM_GREY
        //self.max_chroma_hcv().rgb::<T>()
    }

    // fn max_chroma_hcv(&self) -> HCV {
    //     HCV {
    //         hue: Some((*self).into()),
    //         chroma: Chroma::ONE,
    //         sum: self.sum_for_max_chroma(),
    //     }
    // }

    fn max_chroma_rgb_for_sum<T: LightLevel>(&self, sum: Real) -> Option<RGB<T>> {
        debug_assert!(sum.is_valid_sum(), "sum: {:?}", sum);
        match sum {
            Real::ZERO | Real::THREE => None,
            sum => {
                let max_chroma = self.max_chroma_for_sum(sum)?;
                let (chroma, sum) = self.adjusted_favouring_sum(sum, max_chroma)?;
                let triplet = self.rgb_ordered_triplet(sum, chroma.into_prop())?;
                Some(RGB::<T>::from(triplet))
            }
        }
    }

    // fn darkest_hcv_for_chroma(&self, chroma: Chroma) -> Option<HCV> {
    //     debug_assert!(chroma.is_valid());
    //     let sum = self.min_sum_for_chroma(chroma)?;
    //     debug_assert!(self.sum_and_chroma_are_compatible(sum, chroma));
    //     Some(HCV {
    //         hue: Some((*self).into()),
    //         chroma,
    //         sum,
    //     })
    // }

    // fn lightest_hcv_for_chroma(&self, chroma: Chroma) -> Option<HCV> {
    //     debug_assert!(chroma.is_valid());
    //     let sum = self.max_sum_for_chroma(chroma)?;
    //     debug_assert!(self.sum_and_chroma_are_compatible(sum, chroma));
    //     Some(HCV {
    //         hue: Some((*self).into()),
    //         chroma,
    //         sum,
    //     })
    // }

    fn darkest_rgb_for_chroma<T: LightLevel>(&self, chroma: Chroma) -> Option<RGB<T>> {
        debug_assert!(chroma.is_valid());
        let sum = self.min_sum_for_chroma(chroma)?;
        debug_assert!(self.sum_and_chroma_are_compatible(sum, chroma));
        let triplet = self.rgb_ordered_triplet(sum, chroma.into_prop())?;
        Some(RGB::<T>::from(triplet))
    }

    fn lightest_rgb_for_chroma<T: LightLevel>(&self, chroma: Chroma) -> Option<RGB<T>> {
        debug_assert!(chroma.is_valid());
        let sum = self.max_sum_for_chroma(chroma)?;
        debug_assert!(self.sum_and_chroma_are_compatible(sum, chroma));
        let triplet = self.rgb_ordered_triplet(sum, chroma.into_prop())?;
        Some(RGB::<T>::from(triplet))
    }

    fn rgb_for_sum_and_chroma<T: LightLevel>(&self, sum: Real, chroma: Chroma) -> Option<RGB<T>> {
        debug_assert!(chroma.is_valid() && sum.is_valid_sum());
        let (min_sum, max_sum) = self.sum_range_for_chroma(chroma)?;
        match sum {
            sum if sum < min_sum || sum > max_sum => None,
            sum => match chroma.into_prop() {
                Prop::ZERO => None,
                c_prop => {
                    // TODO: Do we need this
                    let (chroma, sum) = self.trim_overs(sum, c_prop)?;
                    let triplet = self.rgb_ordered_triplet(sum, chroma.into_prop())?;
                    Some(RGB::<T>::from(triplet))
                }
            },
        }
    }
}

pub(crate) trait ColourModificationHelpers: HueBasics + Debug + Sized {
    fn trim_overs(&self, sum: Real, c_prop: Prop) -> Option<(Chroma, Real)> {
        debug_assert!(sum.is_valid_sum());
        match self.sum_for_max_chroma() * c_prop {
            Real::ZERO => None,
            min_sum if sum < min_sum => Some((Chroma::Shade(c_prop), min_sum)),
            min_sum => match sum - (sum - min_sum) % Real(3.0) {
                Real::ZERO => None,
                sum => match sum.cmp(&self.sum_for_max_chroma()) {
                    Ordering::Equal => Some((Chroma::Neither(c_prop), sum)),
                    Ordering::Less => Some((Chroma::Shade(c_prop), sum)),
                    Ordering::Greater => Some((Chroma::Tint(c_prop), sum)),
                },
            },
        }
    }

    fn adjusted_favouring_chroma(&self, sum: Real, chroma: Chroma) -> Option<(Chroma, Real)> {
        debug_assert!(chroma.is_valid() && sum.is_valid_sum());
        match chroma {
            Chroma::ZERO => None,
            Chroma::ONE => Some((Chroma::ONE, self.sum_for_max_chroma())),
            Chroma::Neither(c_prop) | Chroma::Tint(c_prop) | Chroma::Shade(c_prop) => {
                let (chroma, sum) = self.trim_overs(sum, c_prop)?;
                match sum.cmp(&self.sum_for_max_chroma()) {
                    Ordering::Equal | Ordering::Less => Some((chroma, sum)),
                    Ordering::Greater => {
                        let max_sum = self
                            .max_sum_for_chroma_prop(chroma.into_prop())
                            .expect("chroma > 0");
                        if sum > max_sum {
                            Some(self.trim_overs(max_sum, c_prop)?)
                        } else {
                            Some((chroma, sum))
                        }
                    }
                }
            }
        }
    }

    fn adjusted_favouring_sum(&self, sum: Real, chroma: Chroma) -> Option<(Chroma, Real)> {
        debug_assert!(chroma.is_valid() && sum.is_valid_sum());
        match chroma {
            Chroma::ZERO => None,
            Chroma::Neither(c_prop) | Chroma::Tint(c_prop) | Chroma::Shade(c_prop) => {
                match sum.cmp(&self.sum_for_max_chroma()) {
                    Ordering::Equal => Some(self.trim_overs(sum, c_prop)?),
                    Ordering::Less => {
                        let min_sum = self.min_sum_for_chroma_prop(c_prop).expect("chroma > 0");
                        if sum < min_sum {
                            let max_chroma = self.max_chroma_prop_for_sum(sum)?;
                            Some(self.trim_overs(sum, max_chroma)?)
                        } else {
                            Some(self.trim_overs(sum, c_prop)?)
                        }
                    }
                    Ordering::Greater => {
                        let max_chroma = self.max_chroma_prop_for_sum(sum)?;
                        if c_prop > max_chroma {
                            Some(self.trim_overs(sum, max_chroma)?)
                        } else {
                            Some(self.trim_overs(sum, c_prop)?)
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord)]
pub enum RGBHue {
    Red = 5,
    Green = 9,
    Blue = 1,
}

impl From<RGBHue> for Hue {
    fn from(rgb_hue: RGBHue) -> Self {
        Hue::Primary(rgb_hue)
    }
}

impl PropDiff for RGBHue {
    fn prop_diff(&self, other: &Self) -> Option<Prop> {
        if self == other {
            Some(Prop::ZERO)
        } else {
            None
        }
    }
}

impl ApproxEq for RGBHue {}

impl HueBasics for RGBHue {
    fn sum_for_max_chroma(&self) -> Real {
        Real::ONE
    }
}

impl OrderedTriplets for RGBHue {
    fn has_valid_value_order(&self, triplet: &[Prop; 3]) -> bool {
        triplet[0] > triplet[1] && triplet[1] == triplet[2]
    }

    fn has_valid_rgb_order(&self, triplet: &[Prop; 3]) -> bool {
        use RGBHue::*;
        match self {
            Red => triplet[0] > triplet[1] && triplet[1] == triplet[2],
            Green => triplet[1] > triplet[0] && triplet[0] == triplet[2],
            Blue => triplet[2] > triplet[0] && triplet[0] == triplet[1],
        }
    }

    fn triplet_to_rgb_order(&self, triplet: &[Prop; 3]) -> [Prop; 3] {
        debug_assert!(self.has_valid_value_order(&triplet));
        use RGBHue::*;
        match self {
            Red => *triplet,
            Green => [triplet[1], triplet[0], triplet[2]],
            Blue => [triplet[2], triplet[1], triplet[0]],
        }
    }
}

impl ColourModificationHelpers for RGBHue {}

impl SumChromaCompatibility for RGBHue {}

impl HueIfce for RGBHue {
    fn angle(&self) -> Angle {
        match self {
            RGBHue::Red => Angle::RED,
            RGBHue::Green => Angle::GREEN,
            RGBHue::Blue => Angle::BLUE,
        }
    }

    fn warmth_for_chroma(&self, chroma: Chroma) -> Warmth {
        debug_assert!(chroma.is_valid());
        let x_dash = match self {
            RGBHue::Red => ((Real::ONE + chroma.into_prop()) / Real(2.0)).into(),
            RGBHue::Green | RGBHue::Blue => ((Real::TWO - chroma.into_prop()) / Real(4.0)).into(),
        };
        Warmth::calculate(chroma, x_dash)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord)]
pub enum CMYHue {
    Cyan = 113,
    Magenta = 3,
    Yellow = 7,
}

impl From<CMYHue> for Hue {
    fn from(cmy_hue: CMYHue) -> Self {
        Hue::Secondary(cmy_hue)
    }
}

impl PropDiff for CMYHue {
    fn prop_diff(&self, other: &Self) -> Option<Prop> {
        if self == other {
            Some(Prop::ZERO)
        } else {
            None
        }
    }
}

impl ApproxEq for CMYHue {}

impl HueBasics for CMYHue {
    fn sum_for_max_chroma(&self) -> Real {
        Real::TWO
    }
}

impl OrderedTriplets for CMYHue {
    fn has_valid_value_order(&self, triplet: &[Prop; 3]) -> bool {
        triplet[0] == triplet[1] && triplet[1] > triplet[2]
    }

    fn has_valid_rgb_order(&self, triplet: &[Prop; 3]) -> bool {
        use CMYHue::*;
        match self {
            Cyan => triplet[1] == triplet[2] && triplet[1] > triplet[0],
            Magenta => triplet[0] == triplet[2] && triplet[0] > triplet[1],
            Yellow => triplet[0] == triplet[1] && triplet[0] > triplet[2],
        }
    }

    fn triplet_to_rgb_order(&self, triplet: &[Prop; 3]) -> [Prop; 3] {
        debug_assert!(self.has_valid_value_order(&triplet));
        use CMYHue::*;
        match self {
            Cyan => [triplet[2], triplet[0], triplet[1]],
            Magenta => [triplet[0], triplet[2], triplet[1]],
            Yellow => *triplet,
        }
    }
}

impl ColourModificationHelpers for CMYHue {}

impl SumChromaCompatibility for CMYHue {}

impl HueIfce for CMYHue {
    fn angle(&self) -> Angle {
        match self {
            CMYHue::Cyan => Angle::CYAN,
            CMYHue::Magenta => Angle::MAGENTA,
            CMYHue::Yellow => Angle::YELLOW,
        }
    }

    fn warmth_for_chroma(&self, chroma: Chroma) -> Warmth {
        debug_assert!(chroma.is_valid());
        let x_dash = match self {
            CMYHue::Cyan => (Real::ONE - chroma.into_prop()) / Real(2.0),
            CMYHue::Magenta | CMYHue::Yellow => (Real::TWO + chroma.into_prop()) / Real(4.0),
        };
        Warmth::calculate(chroma, x_dash.into())
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

impl From<SextantHue> for Hue {
    fn from(sextant_hue: SextantHue) -> Self {
        Hue::Sextant(sextant_hue)
    }
}

impl PropDiff for SextantHue {
    fn prop_diff(&self, other: &Self) -> Option<Prop> {
        if self.0 == other.0 {
            self.1.prop_diff(&other.1)
        } else {
            None
        }
    }
}

impl ApproxEq for SextantHue {}

impl Eq for SextantHue {}

impl HueBasics for SextantHue {
    fn sum_for_max_chroma(&self) -> Real {
        Real::ONE + self.1
    }
}

impl OrderedTriplets for SextantHue {
    fn has_valid_value_order(&self, triplet: &[Prop; 3]) -> bool {
        triplet[0] > triplet[1] && triplet[1] > triplet[2]
    }

    fn has_valid_rgb_order(&self, triplet: &[Prop; 3]) -> bool {
        use Sextant::*;
        match self.0 {
            RedMagenta => triplet[0] > triplet[2] && triplet[2] > triplet[1],
            RedYellow => triplet[0] > triplet[1] && triplet[1] > triplet[2],
            GreenYellow => triplet[1] > triplet[0] && triplet[0] > triplet[2],
            GreenCyan => triplet[1] > triplet[2] && triplet[2] > triplet[0],
            BlueCyan => triplet[2] > triplet[1] && triplet[1] > triplet[0],
            BlueMagenta => triplet[2] > triplet[0] && triplet[0] > triplet[1],
        }
    }

    fn triplet_to_rgb_order(&self, triplet: &[Prop; 3]) -> [Prop; 3] {
        debug_assert!(self.has_valid_value_order(&triplet));
        use Sextant::*;
        match self.0 {
            RedMagenta => [triplet[0], triplet[2], triplet[1]],
            RedYellow => *triplet,
            GreenYellow => [triplet[1], triplet[0], triplet[2]],
            GreenCyan => [triplet[2], triplet[0], triplet[1]],
            BlueCyan => [triplet[2], triplet[1], triplet[0]],
            BlueMagenta => [triplet[1], triplet[2], triplet[0]],
        }
    }
}

impl ColourModificationHelpers for SextantHue {}

impl SumChromaCompatibility for SextantHue {
    fn sum_and_chroma_prop_are_compatible(&self, sum: Real, c_prop: Prop) -> bool {
        if let Some((min_sum, max_sum)) = self.sum_range_for_chroma_prop(c_prop) {
            sum >= min_sum && sum <= max_sum
        } else {
            false
        }
    }

    fn sum_and_chroma_are_compatible(&self, sum: Real, chroma: Chroma) -> bool {
        debug_assert!(chroma.is_valid());
        if let Some((min_sum, max_sum)) = self.sum_range_for_chroma(chroma) {
            sum >= min_sum && sum <= max_sum
        } else {
            false
        }
    }
}

impl SextantHue {
    pub fn abs_diff(&self, other: &Self) -> Prop {
        if self.0 == other.0 {
            self.1.abs_diff(&other.1)
        } else {
            Prop::ONE
        }
    }

    pub fn sextant(&self) -> Sextant {
        self.0
    }

    pub fn prop(&self) -> Prop {
        self.1
    }
}

#[cfg(test)]
impl SextantHue {
    pub fn approx_eq(&self, other: &Self, acceptable_rounding_error: Option<Prop>) -> bool {
        if self.0 == other.0 {
            self.1.approx_eq(&other.1, acceptable_rounding_error)
        } else {
            false
        }
    }
}

impl<T: LightLevel> From<(Sextant, &RGB<T>)> for SextantHue {
    fn from(arg: (Sextant, &RGB<T>)) -> Self {
        Self::from((arg.0, <[Prop; 3]>::from(*arg.1)))
    }
}

impl From<(Sextant, [Prop; 3])> for SextantHue {
    fn from(arg: (Sextant, [Prop; 3])) -> Self {
        let [first, second, third] = arg.1;
        debug_assert!(first > second && second > third);
        let mut sum_for_max_chroma = Real::ONE + ((second - third) / (first - third));
        debug_assert!(sum_for_max_chroma > Real::ONE && sum_for_max_chroma < Real::TWO);
        // Handle possible (fatal) rounding error
        while sum_for_max_chroma > Real::ONE + Real(f64::EPSILON)
            && sum_for_max_chroma * (first - third) > first + second + third
        {
            sum_for_max_chroma = sum_for_max_chroma - Real(f64::EPSILON);
        }
        debug_assert!(sum_for_max_chroma > Real::ONE && sum_for_max_chroma < Real::TWO);
        debug_assert_eq!(
            (first + second + third - sum_for_max_chroma * (first - third)) / Real(3.0),
            third.into()
        );
        Self(arg.0, (sum_for_max_chroma - Real::ONE).into())
    }
}

impl From<(Sextant, &[Prop; 3])> for SextantHue {
    fn from(arg: (Sextant, &[Prop; 3])) -> Self {
        Self::from((arg.0, *arg.1))
    }
}

impl HueIfce for SextantHue {
    fn angle(&self) -> Angle {
        match self {
            SextantHue(Sextant::BlueCyan, Prop::HALF) => Angle::BLUE_CYAN,
            SextantHue(Sextant::BlueMagenta, Prop::HALF) => Angle::BLUE_MAGENTA,
            SextantHue(Sextant::RedMagenta, Prop::HALF) => Angle::RED_MAGENTA,
            SextantHue(Sextant::RedYellow, Prop::HALF) => Angle::RED_YELLOW,
            SextantHue(Sextant::GreenYellow, Prop::HALF) => Angle::GREEN_YELLOW,
            SextantHue(Sextant::GreenCyan, Prop::HALF) => Angle::GREEN_CYAN,
            _ => {
                let second: f64 = self.1.into();
                let sin = f64::SQRT_3 * second / 2.0 / (1.0 - second + second.powi(2)).sqrt();
                let angle = Angle::asin(Real::from(sin));
                match self.0 {
                    Sextant::RedMagenta => -angle,
                    Sextant::RedYellow => angle,
                    Sextant::GreenYellow => Angle::GREEN - angle,
                    Sextant::GreenCyan => Angle::GREEN + angle,
                    Sextant::BlueCyan => Angle::BLUE - angle,
                    Sextant::BlueMagenta => Angle::BLUE + angle,
                }
            }
        }
    }

    fn warmth_for_chroma(&self, chroma: Chroma) -> Warmth {
        debug_assert!(chroma.is_valid());
        let kc = chroma.into_prop() * self.1;
        let x_dash = match self.0 {
            // TODO: take tint and shade into account
            Sextant::RedYellow | Sextant::RedMagenta => {
                (Real::TWO + chroma.into_prop() * Real(2.0) - kc) / Real(4.0)
            }
            Sextant::GreenYellow | Sextant::BlueMagenta => {
                (Real::TWO + kc * Real(2.0) - chroma.into_prop()) / Real(4.0)
            }
            Sextant::GreenCyan | Sextant::BlueCyan => {
                (Real::TWO - kc - chroma.into_prop()) / Real(4.0)
            }
        };
        Warmth::calculate(chroma, x_dash.into())
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

    const BLUE_CYAN: Self = Self::Sextant(SextantHue(Sextant::BlueCyan, Prop::HALF));
    const BLUE_MAGENTA: Self = Self::Sextant(SextantHue(Sextant::BlueMagenta, Prop::HALF));
    const RED_MAGENTA: Self = Self::Sextant(SextantHue(Sextant::RedMagenta, Prop::HALF));
    const RED_YELLOW: Self = Self::Sextant(SextantHue(Sextant::RedYellow, Prop::HALF));
    const GREEN_YELLOW: Self = Self::Sextant(SextantHue(Sextant::GreenYellow, Prop::HALF));
    const GREEN_CYAN: Self = Self::Sextant(SextantHue(Sextant::GreenCyan, Prop::HALF));
}

impl Default for Hue {
    fn default() -> Self {
        Self::RED
    }
}

impl TryFrom<[Prop; 3]> for Hue {
    type Error = &'static str;

    fn try_from(arrayx: [Prop; 3]) -> Result<Self, Self::Error> {
        use Sextant::*;
        let [red, green, blue] = arrayx;
        match red.cmp(&green) {
            Ordering::Greater => match green.cmp(&blue) {
                Ordering::Greater => Ok(Hue::Sextant(SextantHue::from((
                    RedYellow,
                    [red, green, blue],
                )))),
                Ordering::Less => match red.cmp(&blue) {
                    Ordering::Greater => Ok(Hue::Sextant(SextantHue::from((
                        RedMagenta,
                        [red, blue, green],
                    )))),
                    Ordering::Less => Ok(Hue::Sextant(SextantHue::from((
                        BlueMagenta,
                        [blue, red, green],
                    )))),
                    Ordering::Equal => Ok(Hue::Secondary(CMYHue::Magenta)),
                },
                Ordering::Equal => Ok(Hue::Primary(RGBHue::Red)),
            },
            Ordering::Less => match red.cmp(&blue) {
                Ordering::Greater => Ok(Hue::Sextant(SextantHue::from((
                    GreenYellow,
                    [green, red, blue],
                )))),
                Ordering::Less => match green.cmp(&blue) {
                    Ordering::Greater => Ok(Hue::Sextant(SextantHue::from((
                        GreenCyan,
                        [green, blue, red],
                    )))),
                    Ordering::Less => Ok(Hue::Sextant(SextantHue::from((
                        BlueCyan,
                        [blue, green, red],
                    )))),
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

impl TryFrom<&[Prop; 3]> for Hue {
    type Error = &'static str;

    fn try_from(array: &[Prop; 3]) -> Result<Self, Self::Error> {
        Self::try_from(*array)
    }
}

impl<T: LightLevel> TryFrom<&RGB<T>> for Hue {
    type Error = &'static str;

    fn try_from(rgb: &RGB<T>) -> Result<Self, Self::Error> {
        Self::try_from(<[Prop; 3]>::from(*rgb))
    }
}

impl<T: LightLevel> TryFrom<RGB<T>> for Hue {
    type Error = &'static str;

    fn try_from(rgb: RGB<T>) -> Result<Self, Self::Error> {
        Self::try_from(<[Prop; 3]>::from(rgb))
    }
}

impl From<Angle> for Hue {
    fn from(angle: Angle) -> Self {
        match angle {
            Angle::RED => Hue::RED,
            Angle::GREEN => Hue::GREEN,
            Angle::BLUE => Hue::BLUE,
            Angle::CYAN => Hue::CYAN,
            Angle::MAGENTA => Hue::MAGENTA,
            Angle::YELLOW => Hue::YELLOW,
            Angle::BLUE_CYAN => Hue::BLUE_CYAN,
            Angle::BLUE_MAGENTA => Hue::BLUE_MAGENTA,
            Angle::RED_MAGENTA => Hue::RED_MAGENTA,
            Angle::RED_YELLOW => Hue::RED_YELLOW,
            Angle::GREEN_YELLOW => Hue::GREEN_YELLOW,
            Angle::GREEN_CYAN => Hue::GREEN_CYAN,
            _ => {
                fn f(angle: Angle) -> Prop {
                    (angle.sin() / (Angle::GREEN - angle).sin()).into()
                };
                if angle >= Angle::RED {
                    if angle < Angle::YELLOW {
                        Hue::Sextant(SextantHue(Sextant::RedYellow, f(angle)))
                    } else if angle < Angle::GREEN {
                        Hue::Sextant(SextantHue(Sextant::GreenYellow, f(Angle::GREEN - angle)))
                    } else {
                        Hue::Sextant(SextantHue(Sextant::GreenCyan, f(angle - Angle::GREEN)))
                    }
                } else if angle > Angle::MAGENTA {
                    Hue::Sextant(SextantHue(Sextant::RedMagenta, f(-angle)))
                } else if angle > Angle::BLUE {
                    Hue::Sextant(SextantHue(Sextant::BlueMagenta, f(Angle::GREEN + angle)))
                } else {
                    Hue::Sextant(SextantHue(Sextant::BlueCyan, f(-angle - Angle::GREEN)))
                }
            }
        }
    }
}

impl From<Hue> for Angle {
    fn from(hue: Hue) -> Self {
        hue.angle()
    }
}

impl HueBasics for Hue {
    fn sum_for_max_chroma(&self) -> Real {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.sum_for_max_chroma(),
            Self::Secondary(cmy_hue) => cmy_hue.sum_for_max_chroma(),
            Self::Sextant(sextant_hue) => sextant_hue.sum_for_max_chroma(),
        }
    }

    fn min_sum_for_chroma_prop(&self, c_prop: Prop) -> Option<Real> {
        match self {
            Self::Primary(primary_hue) => primary_hue.min_sum_for_chroma_prop(c_prop),
            Self::Secondary(secondary_hue) => secondary_hue.min_sum_for_chroma_prop(c_prop),
            Self::Sextant(sextant_hue) => sextant_hue.min_sum_for_chroma_prop(c_prop),
        }
    }

    fn max_sum_for_chroma_prop(&self, c_prop: Prop) -> Option<Real> {
        match self {
            Self::Primary(primary_hue) => primary_hue.max_sum_for_chroma_prop(c_prop),
            Self::Secondary(secondary_hue) => secondary_hue.max_sum_for_chroma_prop(c_prop),
            Self::Sextant(sextant_hue) => sextant_hue.max_sum_for_chroma_prop(c_prop),
        }
    }

    fn sum_range_for_chroma_prop(&self, c_prop: Prop) -> Option<(Real, Real)> {
        match self {
            Self::Primary(primary_hue) => primary_hue.sum_range_for_chroma_prop(c_prop),
            Self::Secondary(secondary_hue) => secondary_hue.sum_range_for_chroma_prop(c_prop),
            Self::Sextant(sextant_hue) => sextant_hue.sum_range_for_chroma_prop(c_prop),
        }
    }

    fn sum_range_for_chroma(&self, chroma: Chroma) -> Option<(Real, Real)> {
        match self {
            Self::Primary(primary_hue) => primary_hue.sum_range_for_chroma(chroma),
            Self::Secondary(secondary_hue) => secondary_hue.sum_range_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.sum_range_for_chroma(chroma),
        }
    }

    fn max_chroma_prop_for_sum(&self, sum: Real) -> Option<Prop> {
        match self {
            Self::Primary(primary_hue) => primary_hue.max_chroma_prop_for_sum(sum),
            Self::Secondary(secondary_hue) => secondary_hue.max_chroma_prop_for_sum(sum),
            Self::Sextant(sextant_hue) => sextant_hue.max_chroma_prop_for_sum(sum),
        }
    }

    fn max_chroma_for_sum(&self, sum: Real) -> Option<Chroma> {
        match self {
            Self::Primary(primary_hue) => primary_hue.max_chroma_for_sum(sum),
            Self::Secondary(secondary_hue) => secondary_hue.max_chroma_for_sum(sum),
            Self::Sextant(sextant_hue) => sextant_hue.max_chroma_for_sum(sum),
        }
    }
}

impl SumChromaCompatibility for Hue {
    fn sum_and_chroma_prop_are_compatible(&self, sum: Real, c_prop: Prop) -> bool {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.sum_and_chroma_prop_are_compatible(sum, c_prop),
            Self::Secondary(cmy_hue) => cmy_hue.sum_and_chroma_prop_are_compatible(sum, c_prop),
            Self::Sextant(sextant_hue) => {
                sextant_hue.sum_and_chroma_prop_are_compatible(sum, c_prop)
            }
        }
    }

    fn sum_and_chroma_are_compatible(&self, sum: Real, chroma: Chroma) -> bool {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.sum_and_chroma_are_compatible(sum, chroma),
            Self::Secondary(cmy_hue) => cmy_hue.sum_and_chroma_are_compatible(sum, chroma),
            Self::Sextant(sextant_hue) => sextant_hue.sum_and_chroma_are_compatible(sum, chroma),
        }
    }
}

impl OrderedTriplets for Hue {
    fn has_valid_value_order(&self, triplet: &[Prop; 3]) -> bool {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.has_valid_value_order(triplet),
            Self::Secondary(cmy_hue) => cmy_hue.has_valid_value_order(triplet),
            Self::Sextant(sextant_hue) => sextant_hue.has_valid_value_order(triplet),
        }
    }

    fn has_valid_rgb_order(&self, triplet: &[Prop; 3]) -> bool {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.has_valid_rgb_order(triplet),
            Self::Secondary(cmy_hue) => cmy_hue.has_valid_rgb_order(triplet),
            Self::Sextant(sextant_hue) => sextant_hue.has_valid_rgb_order(triplet),
        }
    }

    fn triplet_to_rgb_order(&self, triplet: &[Prop; 3]) -> [Prop; 3] {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.triplet_to_rgb_order(triplet),
            Self::Secondary(cmy_hue) => cmy_hue.triplet_to_rgb_order(triplet),
            Self::Sextant(sextant_hue) => sextant_hue.triplet_to_rgb_order(triplet),
        }
    }

    fn rgb_ordered_triplet(&self, sum: Real, c_prop: Prop) -> Option<[Prop; 3]> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.rgb_ordered_triplet(sum, c_prop),
            Self::Secondary(cmy_hue) => cmy_hue.rgb_ordered_triplet(sum, c_prop),
            Self::Sextant(sextant_hue) => sextant_hue.rgb_ordered_triplet(sum, c_prop),
        }
    }

    fn ordered_triplet(&self, sum: Real, c_prop: Prop) -> Option<[Prop; 3]> {
        match self {
            Self::Primary(primary_hue) => primary_hue.ordered_triplet(sum, c_prop),
            Self::Secondary(secondary_hue) => secondary_hue.ordered_triplet(sum, c_prop),
            Self::Sextant(sextant_hue) => sextant_hue.ordered_triplet(sum, c_prop),
        }
    }

    // fn ordered_triplet_to_hcv(&self, triplet: &[Prop; 3]) -> HCV {
    //     match self {
    //         Self::Primary(primary_hue) => primary_hue.ordered_triplet_to_hcv(triplet),
    //         Self::Secondary(secondary_hue) => secondary_hue.ordered_triplet_to_hcv(triplet),
    //         Self::Sextant(sextant_hue) => sextant_hue.ordered_triplet_to_hcv(triplet),
    //     }
    // }
}

impl ColourModificationHelpers for Hue {
    fn trim_overs(&self, sum: Real, c_prop: Prop) -> Option<(Chroma, Real)> {
        match self {
            Self::Primary(primary_hue) => primary_hue.trim_overs(sum, c_prop),
            Self::Secondary(secondary_hue) => secondary_hue.trim_overs(sum, c_prop),
            Self::Sextant(sextant_hue) => sextant_hue.trim_overs(sum, c_prop),
        }
    }

    fn adjusted_favouring_chroma(&self, sum: Real, chroma: Chroma) -> Option<(Chroma, Real)> {
        match self {
            Self::Primary(primary_hue) => primary_hue.adjusted_favouring_chroma(sum, chroma),
            Self::Secondary(secondary_hue) => secondary_hue.adjusted_favouring_chroma(sum, chroma),
            Self::Sextant(sextant_hue) => sextant_hue.adjusted_favouring_chroma(sum, chroma),
        }
    }

    fn adjusted_favouring_sum(&self, sum: Real, chroma: Chroma) -> Option<(Chroma, Real)> {
        match self {
            Self::Primary(primary_hue) => primary_hue.adjusted_favouring_sum(sum, chroma),
            Self::Secondary(secondary_hue) => secondary_hue.adjusted_favouring_sum(sum, chroma),
            Self::Sextant(sextant_hue) => sextant_hue.adjusted_favouring_sum(sum, chroma),
        }
    }
}

impl HueIfce for Hue {
    fn angle(&self) -> Angle {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.angle(),
            Self::Secondary(cmy_hue) => cmy_hue.angle(),
            Self::Sextant(sextant_hue) => sextant_hue.angle(),
        }
    }

    fn warmth_for_chroma(&self, chroma: Chroma) -> Warmth {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.warmth_for_chroma(chroma),
            Self::Secondary(cmy_hue) => cmy_hue.warmth_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.warmth_for_chroma(chroma),
        }
    }

    fn min_sum_for_chroma(&self, chroma: Chroma) -> Option<Real> {
        match self {
            Self::Primary(primary_hue) => primary_hue.min_sum_for_chroma(chroma),
            Self::Secondary(secondary_hue) => secondary_hue.min_sum_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.min_sum_for_chroma(chroma),
        }
    }

    fn max_sum_for_chroma(&self, chroma: Chroma) -> Option<Real> {
        match self {
            Self::Primary(primary_hue) => primary_hue.max_sum_for_chroma(chroma),
            Self::Secondary(secondary_hue) => secondary_hue.max_sum_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.max_sum_for_chroma(chroma),
        }
    }

    fn max_chroma_rgb<L: LightLevel>(&self) -> RGB<L> {
        match self {
            Self::Primary(primary_hue) => primary_hue.max_chroma_rgb::<L>(),
            Self::Secondary(secondary_hue) => secondary_hue.max_chroma_rgb::<L>(),
            Self::Sextant(sextant_hue) => sextant_hue.max_chroma_rgb::<L>(),
        }
    }

    // fn max_chroma_hcv(&self) -> HCV {
    //     match self {
    //         Self::Primary(primary_hue) => primary_hue.max_chroma_hcv(),
    //         Self::Secondary(secondary_hue) => secondary_hue.max_chroma_hcv(),
    //         Self::Sextant(sextant_hue) => sextant_hue.max_chroma_hcv(),
    //     }
    // }

    fn max_chroma_rgb_for_sum<T: LightLevel>(&self, sum: Real) -> Option<RGB<T>> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.max_chroma_rgb_for_sum(sum),
            Self::Secondary(cmy_hue) => cmy_hue.max_chroma_rgb_for_sum(sum),
            Self::Sextant(sextant_hue) => sextant_hue.max_chroma_rgb_for_sum(sum),
        }
    }

    fn darkest_rgb_for_chroma<T: LightLevel>(&self, chroma: Chroma) -> Option<RGB<T>> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.darkest_rgb_for_chroma(chroma),
            Self::Secondary(cmy_hue) => cmy_hue.darkest_rgb_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.darkest_rgb_for_chroma(chroma),
        }
    }

    fn lightest_rgb_for_chroma<T: LightLevel>(&self, chroma: Chroma) -> Option<RGB<T>> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.lightest_rgb_for_chroma(chroma),
            Self::Secondary(cmy_hue) => cmy_hue.lightest_rgb_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.lightest_rgb_for_chroma(chroma),
        }
    }

    fn rgb_for_sum_and_chroma<T: LightLevel>(&self, sum: Real, chroma: Chroma) -> Option<RGB<T>> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.rgb_for_sum_and_chroma(sum, chroma),
            Self::Secondary(cmy_hue) => cmy_hue.rgb_for_sum_and_chroma(sum, chroma),
            Self::Sextant(sextant_hue) => sextant_hue.rgb_for_sum_and_chroma(sum, chroma),
        }
    }
}

impl PropDiff for Hue {
    fn prop_diff(&self, other: &Self) -> Option<Prop> {
        match self {
            Self::Primary(self_hue) => match other {
                Self::Primary(other_hue) => self_hue.prop_diff(other_hue),
                _ => None,
            },
            Self::Secondary(self_hue) => match other {
                Self::Secondary(other_hue) => self_hue.prop_diff(other_hue),
                _ => None,
            },
            Self::Sextant(self_hue) => match other {
                Self::Sextant(other_hue) => self_hue.prop_diff(other_hue),
                _ => None,
            },
        }
    }
}

impl ApproxEq for Hue {}

impl Hue {
    pub fn ord_index(&self) -> u8 {
        0
    }
}

impl Add<Angle> for Hue {
    type Output = Self;

    fn add(self, angle: Angle) -> Self {
        Hue::from(self.angle().add(angle))
    }
}

impl Sub<Angle> for Hue {
    type Output = Self;

    fn sub(self, angle: Angle) -> Self {
        Hue::from(self.angle().sub(angle))
    }
}

impl Sub for Hue {
    type Output = Angle;

    fn sub(self, other: Self) -> Angle {
        self.angle().sub(other.angle())
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
mod hue_tests;