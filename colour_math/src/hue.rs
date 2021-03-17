// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    cmp::Ordering,
    convert::{From, Into, TryFrom},
    fmt::Debug,
};

pub mod angle;

use crate::{
    fdrn::UFDRNumber, hue::angle::Angle, proportion::Warmth, Chroma, ColourBasics, HueConstants,
    LightLevel, Prop, HCV, RGB,
};

use crate::fdrn::FDRNumber;
use num_traits_plus::{debug_assert_approx_eq, float_plus::FloatPlus};
use std::ops::{Add, Sub};

pub(crate) trait HueBasics: Copy + Debug + Sized + Into<Hue> {
    fn sum_for_max_chroma(&self) -> UFDRNumber;

    fn min_sum_for_chroma_prop(&self, c_prop: Prop) -> Option<UFDRNumber> {
        match c_prop {
            Prop::ZERO => None,
            c_prop => match self.sum_for_max_chroma() * c_prop {
                UFDRNumber::ZERO => None,
                sum => Some(sum),
            },
        }
    }

    fn max_sum_for_chroma_prop(&self, c_prop: Prop) -> Option<UFDRNumber> {
        match c_prop {
            Prop::ZERO => None,
            c_prop => {
                match UFDRNumber::THREE - (UFDRNumber::THREE - self.sum_for_max_chroma()) * c_prop {
                    UFDRNumber::ZERO => None,
                    sum => Some(sum),
                }
            }
        }
    }

    fn sum_range_for_chroma_prop(&self, c_prop: Prop) -> Option<(UFDRNumber, UFDRNumber)> {
        let min = self.min_sum_for_chroma_prop(c_prop)?;
        let max = self.max_sum_for_chroma_prop(c_prop)?;
        Some((min, max))
    }

    fn sum_range_for_chroma(&self, chroma: Chroma) -> Option<(UFDRNumber, UFDRNumber)> {
        let (min_sum, max_sum) = self.sum_range_for_chroma_prop(chroma.prop())?;
        match chroma {
            Chroma::ZERO => None,
            Chroma::Neither(_) => Some((self.sum_for_max_chroma(), self.sum_for_max_chroma())),
            Chroma::Shade(_) => Some((min_sum, self.sum_for_max_chroma() - UFDRNumber(1))),
            Chroma::Tint(_) => Some((self.sum_for_max_chroma() + UFDRNumber(1), max_sum)),
        }
    }

    fn max_chroma_prop_for_sum(&self, sum: UFDRNumber) -> Option<Prop> {
        Some(self.max_chroma_for_sum(sum)?.prop())
    }

    fn max_chroma_for_sum(&self, sum: UFDRNumber) -> Option<Chroma> {
        debug_assert!(sum.is_valid_sum(), "sum: {:?}", sum);
        if sum.is_hue_valid() {
            match sum.cmp(&self.sum_for_max_chroma()) {
                Ordering::Equal => Some(Chroma::ONE),
                Ordering::Less => {
                    let temp = sum / self.sum_for_max_chroma();
                    Some(Chroma::Shade(temp.into()))
                }
                Ordering::Greater => {
                    let temp =
                        (UFDRNumber::THREE - sum) / (UFDRNumber::THREE - self.sum_for_max_chroma());
                    Some(Chroma::Tint(temp.into()))
                }
            }
        } else {
            None
        }
    }
}

pub(crate) trait SumChromaCompatibility: HueBasics {
    fn sum_and_chroma_prop_are_compatible(&self, sum: UFDRNumber, c_prop: Prop) -> bool {
        if let Some((min_sum, max_sum)) = self.sum_range_for_chroma_prop(c_prop) {
            sum >= min_sum
                && sum <= max_sum
                && (sum - self.sum_for_max_chroma() * c_prop) % 3 == UFDRNumber::ZERO
        } else {
            false
        }
    }

    fn sum_and_chroma_are_compatible(&self, sum: UFDRNumber, chroma: Chroma) -> bool {
        if let Some((min_sum, max_sum)) = self.sum_range_for_chroma(chroma) {
            sum >= min_sum
                && sum <= max_sum
                && (sum - self.sum_for_max_chroma() * chroma.prop()) % 3 == UFDRNumber::ZERO
        } else {
            false
        }
    }
}

pub(crate) trait OrderedTriplets: HueBasics + SumChromaCompatibility {
    fn has_valid_value_order(&self, triplet: &[Prop; 3]) -> bool;
    fn has_valid_rgb_order(&self, triplet: &[Prop; 3]) -> bool;
    fn triplet_to_rgb_order(&self, triplet: &[Prop; 3]) -> [Prop; 3];

    fn rgb_ordered_triplet(&self, sum: UFDRNumber, c_prop: Prop) -> Option<[Prop; 3]> {
        let triplet = self.ordered_triplet(sum, c_prop)?;
        debug_assert!(self.has_valid_value_order(&triplet));
        Some(self.triplet_to_rgb_order(&triplet))
    }

    fn ordered_triplet(&self, sum: UFDRNumber, c_prop: Prop) -> Option<[Prop; 3]> {
        if sum >= self.sum_for_max_chroma() * c_prop {
            debug_assert!(self.sum_and_chroma_prop_are_compatible(sum, c_prop));
            let third = (sum - self.sum_for_max_chroma() * c_prop) / 3;
            let first = third + c_prop;
            if first.is_proportion() {
                let second = sum - first - third;
                debug_assert_eq!(first + second + third, sum);
                debug_assert_eq!(first - third, c_prop.into());
                debug_assert_approx_eq!(first + second + third, sum_for_max_chroma * chroma.prop());
                let triplet = [first.to_prop(), second.to_prop(), third.to_prop()];
                debug_assert!(self.has_valid_value_order(&triplet));
                Some(triplet)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn ordered_triplet_to_hcv(&self, triplet: &[Prop; 3]) -> HCV {
        debug_assert!(self.has_valid_value_order(triplet));
        let sum = triplet[0] + triplet[1] + triplet[2];
        let c_prop = triplet[0] - triplet[2];
        let chroma = match sum.cmp(&self.sum_for_max_chroma()) {
            Ordering::Less => Chroma::Shade(c_prop),
            Ordering::Equal => Chroma::Neither(c_prop),
            Ordering::Greater => Chroma::Tint(c_prop),
        };
        HCV {
            hue: Some((*self).into()),
            chroma,
            sum,
        }
    }
}

pub(crate) trait HueIfce:
    HueBasics + OrderedTriplets + ColourModificationHelpers + SumChromaCompatibility
{
    fn angle(&self) -> Angle;

    fn warmth_for_chroma(&self, chroma: Chroma) -> Warmth;

    fn min_sum_for_chroma(&self, chroma: Chroma) -> Option<UFDRNumber> {
        match chroma {
            Chroma::ZERO => None,
            Chroma::Neither(_) => Some(self.sum_for_max_chroma()),
            Chroma::Shade(c_prop) => self.min_sum_for_chroma_prop(c_prop),
            Chroma::Tint(_) => Some(self.sum_for_max_chroma() + UFDRNumber(1)),
        }
    }

    fn max_sum_for_chroma(&self, chroma: Chroma) -> Option<UFDRNumber> {
        match chroma {
            Chroma::ZERO => None,
            Chroma::Neither(_) => Some(self.sum_for_max_chroma()),
            Chroma::Shade(_) => Some(self.sum_for_max_chroma() - UFDRNumber(1)),
            Chroma::Tint(c_prop) => self.max_sum_for_chroma_prop(c_prop),
        }
    }

    fn max_chroma_rgb<T: LightLevel>(&self) -> RGB<T> {
        self.max_chroma_hcv().rgb::<T>()
    }

    fn max_chroma_hcv(&self) -> HCV {
        HCV {
            hue: Some((*self).into()),
            chroma: Chroma::ONE,
            sum: self.sum_for_max_chroma(),
        }
    }

    fn max_chroma_rgb_for_sum<T: LightLevel>(&self, sum: UFDRNumber) -> Option<RGB<T>> {
        debug_assert!(sum.is_valid_sum(), "sum: {:?}", sum);
        match sum {
            UFDRNumber::ZERO | UFDRNumber::THREE => None,
            sum => {
                let max_chroma = self.max_chroma_for_sum(sum)?;
                let (chroma, sum) = self.adjusted_favouring_sum(sum, max_chroma)?;
                let triplet = self.rgb_ordered_triplet(sum, chroma.prop())?;
                Some(RGB::<T>::from(triplet))
            }
        }
    }

    fn min_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Chroma) -> Option<RGB<T>> {
        let min_sum = self.min_sum_for_chroma(chroma)?;
        let (chroma, sum) = self.adjusted_favouring_chroma(min_sum, chroma)?;
        let triplet = self.rgb_ordered_triplet(sum, chroma.prop())?;
        Some(RGB::<T>::from(triplet))
    }

    fn max_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Chroma) -> Option<RGB<T>> {
        let max_sum = self.max_sum_for_chroma(chroma)?;
        let (chroma, sum) = self.adjusted_favouring_chroma(max_sum, chroma)?;
        let triplet = self.rgb_ordered_triplet(sum, chroma.prop())?;
        Some(RGB::<T>::from(triplet))
    }

    fn rgb_for_sum_and_chroma<T: LightLevel>(
        &self,
        sum: UFDRNumber,
        chroma: Chroma,
    ) -> Option<RGB<T>> {
        let (min_sum, max_sum) = self.sum_range_for_chroma(chroma)?;
        match sum {
            sum if sum < min_sum || sum > max_sum => None,
            sum => match chroma.prop() {
                Prop::ZERO => None,
                c_prop => {
                    let (chroma, sum) = self.trim_overs(sum, c_prop);
                    let triplet = self.rgb_ordered_triplet(sum, chroma.prop())?;
                    Some(RGB::<T>::from(triplet))
                }
            },
        }
    }
}

pub(crate) trait ColourModificationHelpers: HueBasics + Debug + Sized {
    fn overs(&self, sum: UFDRNumber, c_prop: Prop) -> Option<UFDRNumber> {
        debug_assert!(sum.is_valid_sum() && c_prop > Prop::ZERO);
        if sum < self.sum_for_max_chroma() * c_prop {
            None
        } else {
            Some((sum - self.sum_for_max_chroma() * c_prop) % 3)
        }
    }

    fn trim_overs(&self, sum: UFDRNumber, c_prop: Prop) -> (Chroma, UFDRNumber) {
        if c_prop == Prop::ZERO {
            (Chroma::ZERO, sum / 3 * 3)
        } else if let Some(overs) = self.overs(sum, c_prop) {
            let sum = sum - overs;
            match sum.cmp(&self.sum_for_max_chroma()) {
                Ordering::Equal => (Chroma::Neither(c_prop), sum),
                Ordering::Less => (Chroma::Shade(c_prop), sum),
                Ordering::Greater => (Chroma::Tint(c_prop), sum),
            }
        } else {
            (Chroma::Shade(c_prop), self.sum_for_max_chroma() * c_prop)
        }
    }

    fn adjusted_favouring_chroma(
        &self,
        sum: UFDRNumber,
        chroma: Chroma,
    ) -> Option<(Chroma, UFDRNumber)> {
        debug_assert!(sum.is_valid_sum());
        match chroma {
            Chroma::ZERO => None,
            Chroma::ONE => Some((Chroma::ONE, self.sum_for_max_chroma())),
            Chroma::Neither(c_prop) | Chroma::Tint(c_prop) | Chroma::Shade(c_prop) => {
                let (chroma, sum) = self.trim_overs(sum, c_prop);
                match sum.cmp(&self.sum_for_max_chroma()) {
                    Ordering::Equal | Ordering::Less => Some((chroma, sum)),
                    Ordering::Greater => {
                        let max_sum = self
                            .max_sum_for_chroma_prop(chroma.prop())
                            .expect("chroma > 0");
                        if sum > max_sum {
                            Some(self.trim_overs(max_sum, c_prop))
                        } else {
                            Some((chroma, sum))
                        }
                    }
                }
            }
        }
    }

    fn adjusted_favouring_sum(
        &self,
        sum: UFDRNumber,
        chroma: Chroma,
    ) -> Option<(Chroma, UFDRNumber)> {
        debug_assert!(sum.is_valid_sum());
        match chroma {
            Chroma::ZERO => None,
            Chroma::Neither(c_prop) | Chroma::Tint(c_prop) | Chroma::Shade(c_prop) => {
                match sum.cmp(&self.sum_for_max_chroma()) {
                    //Ordering::Equal => Some((chroma, sum)),
                    Ordering::Less | Ordering::Equal => {
                        let min_sum = self.min_sum_for_chroma_prop(c_prop).expect("chroma > 0");
                        if sum < min_sum {
                            if let Some(max_chroma) = self.max_chroma_prop_for_sum(sum) {
                                Some(self.trim_overs(sum, max_chroma))
                            } else {
                                None
                            }
                        } else {
                            Some(self.trim_overs(sum, c_prop))
                        }
                    }
                    Ordering::Greater => {
                        if let Some(max_chroma) = self.max_chroma_prop_for_sum(sum) {
                            if chroma.prop() > max_chroma {
                                Some(self.trim_overs(sum, max_chroma))
                            } else {
                                Some(self.trim_overs(sum, c_prop))
                            }
                        } else {
                            None
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

impl HueBasics for RGBHue {
    fn sum_for_max_chroma(&self) -> UFDRNumber {
        UFDRNumber::ONE
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
        let x_dash = match self {
            RGBHue::Red => ((UFDRNumber::ONE + chroma.prop()) / 2).into(),
            RGBHue::Green | RGBHue::Blue => ((UFDRNumber::TWO - chroma.prop()) / 4).into(),
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

impl HueBasics for CMYHue {
    fn sum_for_max_chroma(&self) -> UFDRNumber {
        UFDRNumber::TWO
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
        let x_dash = match self {
            CMYHue::Cyan => (UFDRNumber::ONE - chroma.prop()) / 2,
            CMYHue::Magenta | CMYHue::Yellow => (UFDRNumber::TWO + chroma.prop()) / 4,
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

impl Eq for SextantHue {}

impl HueBasics for SextantHue {
    fn sum_for_max_chroma(&self) -> UFDRNumber {
        UFDRNumber::ONE + self.1
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
    fn sum_and_chroma_prop_are_compatible(&self, sum: UFDRNumber, c_prop: Prop) -> bool {
        if let Some((min_sum, max_sum)) = self.sum_range_for_chroma_prop(c_prop) {
            sum >= min_sum
                && sum <= max_sum
                && (sum - self.sum_for_max_chroma() * c_prop) % 3 < UFDRNumber(3)
        } else {
            false
        }
    }

    fn sum_and_chroma_are_compatible(&self, sum: UFDRNumber, chroma: Chroma) -> bool {
        if let Some((min_sum, max_sum)) = self.sum_range_for_chroma(chroma) {
            sum >= min_sum
                && sum <= max_sum
                && (sum - self.sum_for_max_chroma() * chroma.prop()) % 3 < UFDRNumber(3)
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
    pub fn approx_eq(&self, other: &Self, acceptable_rounding_error: Option<u64>) -> bool {
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
        let mut sum_for_max_chroma = UFDRNumber::ONE + ((second - third) / (first - third));
        debug_assert!(sum_for_max_chroma > UFDRNumber::ONE && sum_for_max_chroma < UFDRNumber::TWO);
        // Handle possible (fatal) rounding error
        while sum_for_max_chroma > UFDRNumber::ONE + UFDRNumber(1)
            && sum_for_max_chroma * (first - third) > first + second + third
        {
            sum_for_max_chroma = sum_for_max_chroma - UFDRNumber(1);
        }
        debug_assert!(sum_for_max_chroma > UFDRNumber::ONE && sum_for_max_chroma < UFDRNumber::TWO);
        debug_assert_eq!(
            (first + second + third - sum_for_max_chroma * (first - third)) / 3,
            third.into()
        );
        Self(arg.0, (sum_for_max_chroma - UFDRNumber::ONE).into())
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
                let angle = Angle::asin(FDRNumber::from(sin));
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
        let kc = chroma.prop() * self.1;
        let x_dash = match self.0 {
            // TODO: take tint and shade into account
            Sextant::RedYellow | Sextant::RedMagenta => {
                (UFDRNumber::TWO + chroma.prop() * 2 - kc) / 4
            }
            Sextant::GreenYellow | Sextant::BlueMagenta => {
                (UFDRNumber::TWO + kc * 2 - chroma.prop()) / 4
            }
            Sextant::GreenCyan | Sextant::BlueCyan => (UFDRNumber::TWO - kc - chroma.prop()) / 4,
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
    fn sum_for_max_chroma(&self) -> UFDRNumber {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.sum_for_max_chroma(),
            Self::Secondary(cmy_hue) => cmy_hue.sum_for_max_chroma(),
            Self::Sextant(sextant_hue) => sextant_hue.sum_for_max_chroma(),
        }
    }

    fn min_sum_for_chroma_prop(&self, c_prop: Prop) -> Option<UFDRNumber> {
        match self {
            Self::Primary(primary_hue) => primary_hue.min_sum_for_chroma_prop(c_prop),
            Self::Secondary(secondary_hue) => secondary_hue.min_sum_for_chroma_prop(c_prop),
            Self::Sextant(sextant_hue) => sextant_hue.min_sum_for_chroma_prop(c_prop),
        }
    }

    fn max_sum_for_chroma_prop(&self, c_prop: Prop) -> Option<UFDRNumber> {
        match self {
            Self::Primary(primary_hue) => primary_hue.max_sum_for_chroma_prop(c_prop),
            Self::Secondary(secondary_hue) => secondary_hue.max_sum_for_chroma_prop(c_prop),
            Self::Sextant(sextant_hue) => sextant_hue.max_sum_for_chroma_prop(c_prop),
        }
    }

    fn sum_range_for_chroma_prop(&self, c_prop: Prop) -> Option<(UFDRNumber, UFDRNumber)> {
        match self {
            Self::Primary(primary_hue) => primary_hue.sum_range_for_chroma_prop(c_prop),
            Self::Secondary(secondary_hue) => secondary_hue.sum_range_for_chroma_prop(c_prop),
            Self::Sextant(sextant_hue) => sextant_hue.sum_range_for_chroma_prop(c_prop),
        }
    }

    fn sum_range_for_chroma(&self, chroma: Chroma) -> Option<(UFDRNumber, UFDRNumber)> {
        match self {
            Self::Primary(primary_hue) => primary_hue.sum_range_for_chroma(chroma),
            Self::Secondary(secondary_hue) => secondary_hue.sum_range_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.sum_range_for_chroma(chroma),
        }
    }

    fn max_chroma_prop_for_sum(&self, sum: UFDRNumber) -> Option<Prop> {
        match self {
            Self::Primary(primary_hue) => primary_hue.max_chroma_prop_for_sum(sum),
            Self::Secondary(secondary_hue) => secondary_hue.max_chroma_prop_for_sum(sum),
            Self::Sextant(sextant_hue) => sextant_hue.max_chroma_prop_for_sum(sum),
        }
    }

    fn max_chroma_for_sum(&self, sum: UFDRNumber) -> Option<Chroma> {
        match self {
            Self::Primary(primary_hue) => primary_hue.max_chroma_for_sum(sum),
            Self::Secondary(secondary_hue) => secondary_hue.max_chroma_for_sum(sum),
            Self::Sextant(sextant_hue) => sextant_hue.max_chroma_for_sum(sum),
        }
    }
}

impl SumChromaCompatibility for Hue {
    fn sum_and_chroma_prop_are_compatible(&self, sum: UFDRNumber, c_prop: Prop) -> bool {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.sum_and_chroma_prop_are_compatible(sum, c_prop),
            Self::Secondary(cmy_hue) => cmy_hue.sum_and_chroma_prop_are_compatible(sum, c_prop),
            Self::Sextant(sextant_hue) => {
                sextant_hue.sum_and_chroma_prop_are_compatible(sum, c_prop)
            }
        }
    }

    fn sum_and_chroma_are_compatible(&self, sum: UFDRNumber, chroma: Chroma) -> bool {
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

    fn rgb_ordered_triplet(&self, sum: UFDRNumber, c_prop: Prop) -> Option<[Prop; 3]> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.rgb_ordered_triplet(sum, c_prop),
            Self::Secondary(cmy_hue) => cmy_hue.rgb_ordered_triplet(sum, c_prop),
            Self::Sextant(sextant_hue) => sextant_hue.rgb_ordered_triplet(sum, c_prop),
        }
    }

    fn ordered_triplet(&self, sum: UFDRNumber, c_prop: Prop) -> Option<[Prop; 3]> {
        match self {
            Self::Primary(primary_hue) => primary_hue.ordered_triplet(sum, c_prop),
            Self::Secondary(secondary_hue) => secondary_hue.ordered_triplet(sum, c_prop),
            Self::Sextant(sextant_hue) => sextant_hue.ordered_triplet(sum, c_prop),
        }
    }

    fn ordered_triplet_to_hcv(&self, triplet: &[Prop; 3]) -> HCV {
        match self {
            Self::Primary(primary_hue) => primary_hue.ordered_triplet_to_hcv(triplet),
            Self::Secondary(secondary_hue) => secondary_hue.ordered_triplet_to_hcv(triplet),
            Self::Sextant(sextant_hue) => sextant_hue.ordered_triplet_to_hcv(triplet),
        }
    }
}

impl ColourModificationHelpers for Hue {
    fn overs(&self, sum: UFDRNumber, c_prop: Prop) -> Option<UFDRNumber> {
        match self {
            Self::Primary(primary_hue) => primary_hue.overs(sum, c_prop),
            Self::Secondary(secondary_hue) => secondary_hue.overs(sum, c_prop),
            Self::Sextant(sextant_hue) => sextant_hue.overs(sum, c_prop),
        }
    }

    fn trim_overs(&self, sum: UFDRNumber, c_prop: Prop) -> (Chroma, UFDRNumber) {
        match self {
            Self::Primary(primary_hue) => primary_hue.trim_overs(sum, c_prop),
            Self::Secondary(secondary_hue) => secondary_hue.trim_overs(sum, c_prop),
            Self::Sextant(sextant_hue) => sextant_hue.trim_overs(sum, c_prop),
        }
    }

    fn adjusted_favouring_chroma(
        &self,
        sum: UFDRNumber,
        chroma: Chroma,
    ) -> Option<(Chroma, UFDRNumber)> {
        match self {
            Self::Primary(primary_hue) => primary_hue.adjusted_favouring_chroma(sum, chroma),
            Self::Secondary(secondary_hue) => secondary_hue.adjusted_favouring_chroma(sum, chroma),
            Self::Sextant(sextant_hue) => sextant_hue.adjusted_favouring_chroma(sum, chroma),
        }
    }

    fn adjusted_favouring_sum(
        &self,
        sum: UFDRNumber,
        chroma: Chroma,
    ) -> Option<(Chroma, UFDRNumber)> {
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

    fn min_sum_for_chroma(&self, chroma: Chroma) -> Option<UFDRNumber> {
        match self {
            Self::Primary(primary_hue) => primary_hue.min_sum_for_chroma(chroma),
            Self::Secondary(secondary_hue) => secondary_hue.min_sum_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.min_sum_for_chroma(chroma),
        }
    }

    fn max_sum_for_chroma(&self, chroma: Chroma) -> Option<UFDRNumber> {
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

    fn max_chroma_hcv(&self) -> HCV {
        match self {
            Self::Primary(primary_hue) => primary_hue.max_chroma_hcv(),
            Self::Secondary(secondary_hue) => secondary_hue.max_chroma_hcv(),
            Self::Sextant(sextant_hue) => sextant_hue.max_chroma_hcv(),
        }
    }

    fn max_chroma_rgb_for_sum<T: LightLevel>(&self, sum: UFDRNumber) -> Option<RGB<T>> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.max_chroma_rgb_for_sum(sum),
            Self::Secondary(cmy_hue) => cmy_hue.max_chroma_rgb_for_sum(sum),
            Self::Sextant(sextant_hue) => sextant_hue.max_chroma_rgb_for_sum(sum),
        }
    }

    fn min_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Chroma) -> Option<RGB<T>> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.min_sum_rgb_for_chroma(chroma),
            Self::Secondary(cmy_hue) => cmy_hue.min_sum_rgb_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.min_sum_rgb_for_chroma(chroma),
        }
    }

    fn max_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Chroma) -> Option<RGB<T>> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.max_sum_rgb_for_chroma(chroma),
            Self::Secondary(cmy_hue) => cmy_hue.max_sum_rgb_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.max_sum_rgb_for_chroma(chroma),
        }
    }

    fn rgb_for_sum_and_chroma<T: LightLevel>(
        &self,
        sum: UFDRNumber,
        chroma: Chroma,
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

    pub fn abs_diff(&self, other: &Self) -> Prop {
        match self {
            Self::Primary(rgb_hue) => match other {
                Self::Primary(other_rgb_hue) => {
                    if rgb_hue == other_rgb_hue {
                        Prop::ZERO
                    } else {
                        Prop::ONE
                    }
                }
                _ => Prop::ONE,
            },
            Self::Secondary(cmy_hue) => match other {
                Self::Secondary(other_cmy_hue) => {
                    if cmy_hue == other_cmy_hue {
                        Prop::ZERO
                    } else {
                        Prop::ONE
                    }
                }
                _ => Prop::ONE,
            },
            Self::Sextant(sextant_hue) => match other {
                Self::Sextant(other_sextant_hue) => sextant_hue.1.abs_diff(&other_sextant_hue.1),
                _ => Prop::ONE,
            },
        }
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

#[cfg(test)]
impl Hue {
    pub fn approx_eq(&self, other: &Self, acceptable_rounding_error: Option<u64>) -> bool {
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
                    sextant_hue.approx_eq(other_sextant_hue, acceptable_rounding_error)
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
mod hue_tests;
