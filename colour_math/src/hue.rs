// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    cmp::Ordering,
    convert::{From, Into, TryFrom},
    fmt::Debug,
    ops::{Add, Sub},
};

use num_traits_plus::float_plus::FloatPlus;

pub mod angle;

use crate::{
    attributes::{Chroma, Warmth},
    debug::{ApproxEq, PropDiff},
    fdrn::{FDRNumber, IntoProp, Prop, UFDRNumber},
    hcv::HCV,
    hue::angle::Angle,
    rgb::RGB,
    ColourBasics, HueConstants, LightLevel,
};

pub(crate) trait HueBasics: Copy + Debug + Sized + Into<Hue> {
    fn sum_for_max_chroma(&self) -> UFDRNumber;

    fn min_sum_for_chroma_prop(&self, c_prop: Prop) -> Option<UFDRNumber>;

    fn max_sum_for_chroma_prop(&self, c_prop: Prop) -> Option<UFDRNumber>;

    fn sum_range_for_chroma_prop(&self, c_prop: Prop) -> Option<(UFDRNumber, UFDRNumber)> {
        let min = self.min_sum_for_chroma_prop(c_prop)?;
        let max = self.max_sum_for_chroma_prop(c_prop)?;
        Some((min, max))
    }

    fn min_sum_for_chroma(&self, chroma: Chroma) -> Option<UFDRNumber>;

    fn max_sum_for_chroma(&self, chroma: Chroma) -> Option<UFDRNumber>;

    fn sum_range_for_chroma(&self, chroma: Chroma) -> Option<(UFDRNumber, UFDRNumber)> {
        debug_assert!(chroma.is_valid());
        let min = self.min_sum_for_chroma(chroma)?;
        let max = self.max_sum_for_chroma(chroma)?;
        Some((min, max))
    }

    fn max_chroma_prop_for_sum(&self, sum: UFDRNumber) -> Option<Prop> {
        Some(self.max_chroma_for_sum(sum)?.into_prop())
    }

    fn max_chroma_for_sum(&self, sum: UFDRNumber) -> Option<Chroma>;
}

pub(crate) trait SumChromaCompatibility: HueBasics {
    fn sum_in_chroma_prop_range(&self, sum: UFDRNumber, c_prop: Prop) -> bool {
        if let Some((min_sum, max_sum)) = self.sum_range_for_chroma_prop(c_prop) {
            sum >= min_sum && sum <= max_sum
        } else {
            false
        }
    }

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
        debug_assert!(chroma.is_valid());
        chroma.is_valid_re((*self).into(), sum)
            && self.sum_and_chroma_prop_are_compatible(sum, chroma.into_prop())
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

    fn try_rgb_ordered_triplet(
        &self,
        sum: UFDRNumber,
        c_prop: Prop,
    ) -> Option<Result<[Prop; 3], [Prop; 3]>> {
        match self.try_ordered_triplet(sum, c_prop)? {
            Ok(triplet) => {
                debug_assert!(self.has_valid_value_order(&triplet));
                Some(Ok(self.triplet_to_rgb_order(&triplet)))
            }
            Err(triplet) => Some(Err(self.triplet_to_rgb_order(&triplet))),
        }
    }

    fn ordered_triplet(&self, sum: UFDRNumber, c_prop: Prop) -> Option<[Prop; 3]> {
        match self.sum_for_max_chroma() * c_prop {
            min_sum if sum >= min_sum => {
                debug_assert!(self.sum_and_chroma_prop_are_compatible(sum, c_prop));
                let third = (sum - min_sum) / 3;
                let first = third + c_prop;
                if first.is_proportion() {
                    let second = sum - first - third;
                    debug_assert_eq!(first + second + third, sum);
                    debug_assert_eq!(first - third, c_prop.into());
                    let triplet = [first.to_prop(), second.to_prop(), third.to_prop()];
                    debug_assert!(self.has_valid_value_order(&triplet));
                    debug_assert_eq!(
                        Hue::try_from(self.triplet_to_rgb_order(&triplet)).unwrap(),
                        (*self).into()
                    );
                    Some(triplet)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn try_ordered_triplet(
        &self,
        sum: UFDRNumber,
        c_prop: Prop,
    ) -> Option<Result<[Prop; 3], [Prop; 3]>>;

    fn ordered_triplet_to_hcv(&self, triplet: &[Prop; 3]) -> HCV {
        debug_assert!(self.has_valid_value_order(triplet));
        let sum = triplet[0] + triplet[1] + triplet[2];
        let c_prop = triplet[0] - triplet[2];
        HCV {
            hue: Some((*self).into()),
            c_prop,
            sum,
        }
    }
}

pub(crate) trait HueIfce:
    HueBasics + OrderedTriplets + ColourModificationHelpers + SumChromaCompatibility
{
    fn angle(&self) -> Angle;

    fn warmth_for_chroma(&self, chroma: Chroma) -> Warmth;

    fn max_chroma_rgb<T: LightLevel>(&self) -> RGB<T> {
        self.max_chroma_hcv().rgb::<T>()
    }

    fn max_chroma_hcv(&self) -> HCV {
        HCV {
            hue: Some((*self).into()),
            c_prop: Prop::ONE,
            sum: self.sum_for_max_chroma(),
        }
    }

    fn max_chroma_rgb_for_sum<T: LightLevel>(&self, sum: UFDRNumber) -> Option<RGB<T>> {
        debug_assert!(sum.is_valid_sum(), "sum: {:?}", sum);
        match sum {
            UFDRNumber::ZERO | UFDRNumber::THREE => None,
            sum => {
                let max_c_prop = self.max_chroma_prop_for_sum(sum)?;
                let (c_prop, sum) = self.adjusted_favouring_sum(sum, max_c_prop)?;
                let triplet = self.rgb_ordered_triplet(sum, c_prop)?;
                Some(RGB::<T>::from(triplet))
            }
        }
    }
    //
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
    //
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

    fn darkest_rgb_for_chroma<T: LightLevel>(
        &self,
        chroma: Chroma,
    ) -> Option<Result<RGB<T>, RGB<T>>> {
        debug_assert!(chroma.is_valid());
        let sum = self.min_sum_for_chroma(chroma)?;
        match self.try_rgb_ordered_triplet(sum, chroma.into_prop())? {
            Ok(triplet) => Some(Ok(RGB::<T>::from(triplet))),
            Err(triplet) => Some(Err(RGB::<T>::from(triplet))),
        }
    }

    fn lightest_rgb_for_chroma<T: LightLevel>(
        &self,
        chroma: Chroma,
    ) -> Option<Result<RGB<T>, RGB<T>>> {
        debug_assert!(chroma.is_valid());
        let sum = self.max_sum_for_chroma(chroma)?;
        match self.try_rgb_ordered_triplet(sum, chroma.into_prop())? {
            Ok(triplet) => Some(Ok(RGB::<T>::from(triplet))),
            Err(triplet) => Some(Err(RGB::<T>::from(triplet))),
        }
    }

    fn try_rgb_for_sum_and_chroma_prop<T: LightLevel>(
        &self,
        sum: UFDRNumber,
        c_prop: Prop,
    ) -> Option<Result<RGB<T>, RGB<T>>> {
        debug_assert!(sum.is_valid_sum());
        match self.try_rgb_ordered_triplet(sum, c_prop)? {
            Ok(triplet) => Some(Ok(RGB::<T>::from(triplet))),
            Err(triplet) => Some(Err(RGB::<T>::from(triplet))),
        }
    }

    fn rgb_for_sum_and_chroma<T: LightLevel>(
        &self,
        sum: UFDRNumber,
        chroma: Chroma,
    ) -> Option<RGB<T>> {
        debug_assert!(chroma.is_valid() && sum.is_valid_sum());
        let (min_sum, max_sum) = self.sum_range_for_chroma(chroma)?;
        match sum {
            sum if sum < min_sum || sum > max_sum => None,
            sum => match chroma.into_prop() {
                Prop::ZERO => None,
                c_prop => {
                    // TODO: Do we need this
                    let (c_prop, sum) = self.trim_overs(sum, c_prop)?;
                    let triplet = self.rgb_ordered_triplet(sum, c_prop)?;
                    Some(RGB::<T>::from(triplet))
                }
            },
        }
    }
}

pub(crate) trait ColourModificationHelpers: HueBasics + Debug + Sized {
    fn trim_overs(&self, sum: UFDRNumber, c_prop: Prop) -> Option<(Prop, UFDRNumber)> {
        debug_assert!(sum.is_valid_sum());
        match self.sum_for_max_chroma() * c_prop {
            UFDRNumber::ZERO => None,
            min_sum if sum < min_sum => Some((c_prop, min_sum)),
            min_sum => match sum - (sum - min_sum) % 3 {
                UFDRNumber::ZERO => None,
                sum => Some((c_prop, sum)),
            },
        }
    }

    fn adjusted_favouring_chroma(
        &self,
        sum: UFDRNumber,
        c_prop: Prop,
    ) -> Option<(Prop, UFDRNumber)> {
        debug_assert!(sum.is_valid_sum());
        match c_prop {
            Prop::ZERO => None,
            Prop::ONE => Some((Prop::ONE, self.sum_for_max_chroma())),
            c_prop => {
                let (c_prop, sum) = self.trim_overs(sum, c_prop)?;
                match sum.cmp(&self.sum_for_max_chroma()) {
                    Ordering::Equal | Ordering::Less => Some((c_prop, sum)),
                    Ordering::Greater => {
                        let max_sum = self.max_sum_for_chroma_prop(c_prop).expect("chroma > 0");
                        if sum > max_sum {
                            Some(self.trim_overs(max_sum, c_prop)?)
                        } else {
                            Some((c_prop, sum))
                        }
                    }
                }
            }
        }
    }

    fn adjusted_favouring_sum(&self, sum: UFDRNumber, c_prop: Prop) -> Option<(Prop, UFDRNumber)> {
        debug_assert!(sum.is_valid_sum());
        match c_prop {
            Prop::ZERO => None,
            c_prop => match sum.cmp(&self.sum_for_max_chroma()) {
                Ordering::Equal => Some(self.trim_overs(sum, c_prop)?),
                Ordering::Less => {
                    let min_sum = self.min_sum_for_chroma_prop(c_prop).expect("c_prop > 0");
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
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord)]
pub enum RGBHue {
    Red = 5,
    Green = 9,
    Blue = 1,
}

impl RGBHue {
    pub const HUES: [Self; 3] = [RGBHue::Blue, RGBHue::Red, RGBHue::Green];

    fn prop_diff_sextant(&self, sextant_hue: &SextantHue) -> Option<Prop> {
        match self {
            RGBHue::Red => match sextant_hue {
                SextantHue(Sextant::RedYellow, prop) => Some(*prop),
                SextantHue(Sextant::RedMagenta, prop) => Some(*prop),
                _ => None,
            },
            RGBHue::Green => match sextant_hue {
                SextantHue(Sextant::GreenYellow, prop) => Some(*prop),
                SextantHue(Sextant::GreenCyan, prop) => Some(*prop),
                _ => None,
            },
            RGBHue::Blue => match sextant_hue {
                SextantHue(Sextant::BlueCyan, prop) => Some(*prop),
                SextantHue(Sextant::BlueMagenta, prop) => Some(*prop),
                _ => None,
            },
        }
    }
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
    fn sum_for_max_chroma(&self) -> UFDRNumber {
        UFDRNumber::ONE
    }

    fn min_sum_for_chroma_prop(&self, c_prop: Prop) -> Option<UFDRNumber> {
        match c_prop {
            Prop::ZERO => None,
            c_prop => Some(c_prop.into()),
        }
    }

    fn max_sum_for_chroma_prop(&self, c_prop: Prop) -> Option<UFDRNumber> {
        match c_prop {
            Prop::ZERO => None,
            c_prop => Some(UFDRNumber::THREE - c_prop * 2),
        }
    }

    fn min_sum_for_chroma(&self, chroma: Chroma) -> Option<UFDRNumber> {
        debug_assert!(chroma.is_valid());
        match chroma {
            Chroma::ZERO => None,
            Chroma::Neither(_) => Some(UFDRNumber::ONE),
            Chroma::Shade(c_prop) => Some(c_prop.into()),
            Chroma::Tint(c_prop) => {
                Some(UFDRNumber::ONE + UFDRNumber(3 - (u64::MAX - c_prop.0) as u128 % 3))
            }
        }
    }

    fn max_sum_for_chroma(&self, chroma: Chroma) -> Option<UFDRNumber> {
        debug_assert!(chroma.is_valid());
        match chroma {
            Chroma::ZERO => None,
            Chroma::Neither(_) => Some(UFDRNumber::ONE),
            Chroma::Shade(_) => Some(UFDRNumber::ONE - UFDRNumber(1)),
            Chroma::Tint(c_prop) => self.max_sum_for_chroma_prop(c_prop),
        }
    }

    fn max_chroma_for_sum(&self, sum: UFDRNumber) -> Option<Chroma> {
        debug_assert!(sum.is_valid_sum(), "sum: {:?}", sum);
        if sum.is_hue_valid() {
            match sum.cmp(&self.sum_for_max_chroma()) {
                Ordering::Equal => Some(Chroma::ONE),
                Ordering::Less => Some(Chroma::Shade(sum.into_prop())),
                Ordering::Greater => {
                    // NB: make sure it doesn't round to one or zero
                    let c_prop = ((UFDRNumber::THREE - sum) / 2)
                        .into_prop()
                        .min(Prop::ALMOST_ONE)
                        .max(Prop(1));
                    Some(Chroma::Tint(c_prop))
                }
            }
        } else {
            None
        }
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
        use RGBHue::*;
        match self {
            Red => *triplet,
            Green => [triplet[1], triplet[0], triplet[2]],
            Blue => [triplet[2], triplet[1], triplet[0]],
        }
    }

    fn try_ordered_triplet(
        &self,
        sum: UFDRNumber,
        c_prop: Prop,
    ) -> Option<Result<[Prop; 3], [Prop; 3]>> {
        debug_assert!(sum.is_valid_sum());
        match c_prop {
            Prop::ZERO => None,
            min_sum if sum >= min_sum.into() => {
                let third = (sum - min_sum) / 3;
                match third + c_prop {
                    first if first > UFDRNumber::ONE => None,
                    first => {
                        let second = sum - first - third;
                        debug_assert_eq!(first - third, c_prop.into());
                        debug_assert_eq!(first + second + third, sum);
                        if second == third {
                            debug_assert!(first > second);
                            Some(Ok([
                                first.into_prop(),
                                second.into_prop(),
                                third.into_prop(),
                            ]))
                        } else {
                            debug_assert!(first > second && second > third);
                            Some(Err([
                                first.into_prop(),
                                second.into_prop(),
                                third.into_prop(),
                            ]))
                        }
                    }
                }
            }
            _ => None,
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
            RGBHue::Red => ((UFDRNumber::ONE + chroma.into_prop()) / 2).into(),
            RGBHue::Green | RGBHue::Blue => ((UFDRNumber::TWO - chroma.into_prop()) / 4).into(),
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

impl CMYHue {
    pub const HUES: [Self; 3] = [CMYHue::Magenta, CMYHue::Yellow, CMYHue::Cyan];

    fn prop_diff_sextant(&self, sextant_hue: &SextantHue) -> Option<Prop> {
        match self {
            CMYHue::Cyan => match sextant_hue {
                SextantHue(Sextant::GreenCyan, prop) => Some(Prop::ONE - *prop),
                SextantHue(Sextant::BlueCyan, prop) => Some(Prop::ONE - *prop),
                _ => None,
            },
            CMYHue::Magenta => match sextant_hue {
                SextantHue(Sextant::RedMagenta, prop) => Some(Prop::ONE - *prop),
                SextantHue(Sextant::BlueMagenta, prop) => Some(Prop::ONE - *prop),
                _ => None,
            },
            CMYHue::Yellow => match sextant_hue {
                SextantHue(Sextant::RedYellow, prop) => Some(Prop::ONE - *prop),
                SextantHue(Sextant::GreenYellow, prop) => Some(Prop::ONE - *prop),
                _ => None,
            },
        }
    }
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
    fn sum_for_max_chroma(&self) -> UFDRNumber {
        UFDRNumber::TWO
    }

    fn min_sum_for_chroma_prop(&self, c_prop: Prop) -> Option<UFDRNumber> {
        match c_prop {
            Prop::ZERO => None,
            c_prop => Some(c_prop * 2),
        }
    }

    fn max_sum_for_chroma_prop(&self, c_prop: Prop) -> Option<UFDRNumber> {
        match c_prop {
            Prop::ZERO => None,
            c_prop => Some(UFDRNumber::THREE - c_prop),
        }
    }

    fn min_sum_for_chroma(&self, chroma: Chroma) -> Option<UFDRNumber> {
        debug_assert!(chroma.is_valid());
        match chroma {
            Chroma::ZERO => None,
            Chroma::Neither(_) => Some(UFDRNumber::TWO),
            Chroma::Shade(c_prop) => Some(c_prop * 2),
            Chroma::Tint(c_prop) => Some(UFDRNumber::TWO + UFDRNumber(3 - c_prop.0 as u128 % 3)),
        }
    }

    fn max_sum_for_chroma(&self, chroma: Chroma) -> Option<UFDRNumber> {
        debug_assert!(chroma.is_valid());
        match chroma {
            Chroma::ZERO => None,
            Chroma::Neither(_) => Some(UFDRNumber::TWO),
            Chroma::Shade(c_prop) => {
                Some(UFDRNumber::TWO - UFDRNumber(3 - (u64::MAX - c_prop.0) as u128 % 3))
            }
            Chroma::Tint(c_prop) => self.max_sum_for_chroma_prop(c_prop),
        }
    }

    fn max_chroma_for_sum(&self, sum: UFDRNumber) -> Option<Chroma> {
        debug_assert!(sum.is_valid_sum(), "sum: {:?}", sum);
        if sum.is_hue_valid() {
            match sum.cmp(&UFDRNumber::TWO) {
                Ordering::Equal => Some(Chroma::ONE),
                // TODO: fix max chroma for sum (more complicated for uneven sums)
                Ordering::Less => Some(Chroma::Shade((sum / 2).into_prop().max(Prop(1)))),
                Ordering::Greater => Some(Chroma::Tint((UFDRNumber::THREE - sum).into_prop())),
            }
        } else {
            None
        }
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
        //debug_assert!(self.has_valid_value_order(&triplet));
        use CMYHue::*;
        match self {
            Cyan => [triplet[2], triplet[0], triplet[1]],
            Magenta => [triplet[0], triplet[2], triplet[1]],
            Yellow => *triplet,
        }
    }

    fn try_ordered_triplet(
        &self,
        sum: UFDRNumber,
        c_prop: Prop,
    ) -> Option<Result<[Prop; 3], [Prop; 3]>> {
        debug_assert!(sum.is_valid_sum());
        match c_prop * 2 {
            UFDRNumber::ZERO => None,
            min_sum if sum >= min_sum => {
                let third = (sum - min_sum) / 3;
                match third + c_prop {
                    first if first > UFDRNumber::ONE => None,
                    first => match sum - first - third {
                        second if second == first => {
                            debug_assert!(second > third);
                            Some(Ok([first.to_prop(), second.to_prop(), third.to_prop()]))
                        }
                        second => {
                            debug_assert!(second > first && first > third);
                            let diff = second - first;
                            debug_assert!(diff < UFDRNumber(3));
                            if diff == UFDRNumber(1) {
                                if second.is_proportion() {
                                    let first = first - UFDRNumber(1);
                                    let third = third + UFDRNumber(1);
                                    debug_assert!(second > third && first > third);
                                    debug_assert_eq!(second - third, c_prop.into());
                                    debug_assert_eq!(first + second + third, sum);
                                    Some(Err([second.to_prop(), first.to_prop(), third.to_prop()]))
                                } else {
                                    None
                                }
                            } else {
                                let first = first + UFDRNumber(1);
                                if first.is_proportion() {
                                    let third = third + UFDRNumber(1);
                                    let second = second - UFDRNumber(2);
                                    debug_assert!(second > third);
                                    debug_assert_eq!(first - third, c_prop.into());
                                    debug_assert_eq!(first + second + third, sum);
                                    Some(Err([first.to_prop(), second.to_prop(), third.to_prop()]))
                                } else {
                                    None
                                }
                            }
                        }
                    },
                }
            }
            _ => None,
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
            CMYHue::Cyan => (UFDRNumber::ONE - chroma.into_prop()) / 2,
            CMYHue::Magenta | CMYHue::Yellow => (UFDRNumber::TWO + chroma.into_prop()) / 4,
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

impl Sextant {
    pub const SEXTANTS: [Self; 6] = [
        Sextant::BlueCyan,
        Sextant::BlueMagenta,
        Sextant::RedMagenta,
        Sextant::RedYellow,
        Sextant::GreenYellow,
        Sextant::GreenCyan,
    ];
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
        match self {
            SextantHue(Sextant::RedYellow, my_prop) => match other {
                SextantHue(Sextant::RedYellow, other_prop) => my_prop.prop_diff(other_prop),
                SextantHue(Sextant::RedMagenta, other_prop) => match *my_prop + *other_prop {
                    prop if prop <= UFDRNumber::ONE => Some(prop.into_prop()),
                    _ => None,
                },
                SextantHue(Sextant::GreenYellow, other_prop) => {
                    match UFDRNumber::TWO - (*my_prop + *other_prop) {
                        prop if prop <= UFDRNumber::ONE => Some(prop.into_prop()),
                        _ => None,
                    }
                }
                _ => None,
            },
            SextantHue(Sextant::GreenYellow, my_prop) => match other {
                SextantHue(Sextant::GreenYellow, other_prop) => my_prop.prop_diff(other_prop),
                SextantHue(Sextant::GreenCyan, other_prop) => match *my_prop + *other_prop {
                    prop if prop <= UFDRNumber::ONE => Some(prop.into_prop()),
                    _ => None,
                },
                SextantHue(Sextant::RedYellow, other_prop) => {
                    match UFDRNumber::TWO - (*my_prop + *other_prop) {
                        prop if prop <= UFDRNumber::ONE => Some(prop.into_prop()),
                        _ => None,
                    }
                }
                _ => None,
            },
            SextantHue(Sextant::GreenCyan, my_prop) => match other {
                SextantHue(Sextant::GreenCyan, other_prop) => my_prop.prop_diff(other_prop),
                SextantHue(Sextant::GreenYellow, other_prop) => match *my_prop + *other_prop {
                    prop if prop <= UFDRNumber::ONE => Some(prop.into_prop()),
                    _ => None,
                },
                SextantHue(Sextant::BlueCyan, other_prop) => {
                    match UFDRNumber::TWO - (*my_prop + *other_prop) {
                        prop if prop <= UFDRNumber::ONE => Some(prop.into_prop()),
                        _ => None,
                    }
                }
                _ => None,
            },
            SextantHue(Sextant::BlueCyan, my_prop) => match other {
                SextantHue(Sextant::BlueCyan, other_prop) => my_prop.prop_diff(other_prop),
                SextantHue(Sextant::BlueMagenta, other_prop) => match *my_prop + *other_prop {
                    prop if prop <= UFDRNumber::ONE => Some(prop.into_prop()),
                    _ => None,
                },
                SextantHue(Sextant::GreenCyan, other_prop) => {
                    match UFDRNumber::TWO - (*my_prop + *other_prop) {
                        prop if prop <= UFDRNumber::ONE => Some(prop.into_prop()),
                        _ => None,
                    }
                }
                _ => None,
            },
            SextantHue(Sextant::BlueMagenta, my_prop) => match other {
                SextantHue(Sextant::BlueMagenta, other_prop) => my_prop.prop_diff(other_prop),
                SextantHue(Sextant::BlueCyan, other_prop) => match *my_prop + *other_prop {
                    prop if prop <= UFDRNumber::ONE => Some(prop.into_prop()),
                    _ => None,
                },
                SextantHue(Sextant::RedMagenta, other_prop) => {
                    match UFDRNumber::TWO - (*my_prop + *other_prop) {
                        prop if prop <= UFDRNumber::ONE => Some(prop.into_prop()),
                        _ => None,
                    }
                }
                _ => None,
            },
            SextantHue(Sextant::RedMagenta, my_prop) => match other {
                SextantHue(Sextant::RedMagenta, other_prop) => my_prop.prop_diff(other_prop),
                SextantHue(Sextant::RedYellow, other_prop) => match *my_prop + *other_prop {
                    prop if prop <= UFDRNumber::ONE => Some(prop.into_prop()),
                    _ => None,
                },
                SextantHue(Sextant::BlueMagenta, other_prop) => {
                    match UFDRNumber::TWO - (*my_prop + *other_prop) {
                        prop if prop <= UFDRNumber::ONE => Some(prop.into_prop()),
                        _ => None,
                    }
                }
                _ => None,
            },
        }
    }
}

impl ApproxEq for SextantHue {}

impl Eq for SextantHue {}

impl HueBasics for SextantHue {
    fn sum_for_max_chroma(&self) -> UFDRNumber {
        UFDRNumber::ONE + self.1
    }

    fn min_sum_for_chroma_prop(&self, c_prop: Prop) -> Option<UFDRNumber> {
        match c_prop {
            Prop::ZERO => None,
            c_prop => Some(c_prop + self.1 * c_prop),
        }
    }

    fn max_sum_for_chroma_prop(&self, c_prop: Prop) -> Option<UFDRNumber> {
        match (UFDRNumber::TWO - self.1) * c_prop {
            UFDRNumber::ZERO => None,
            sum => Some(UFDRNumber::THREE - sum),
        }
    }

    fn min_sum_for_chroma(&self, chroma: Chroma) -> Option<UFDRNumber> {
        debug_assert!(chroma.is_valid());
        match chroma {
            Chroma::ZERO => None,
            Chroma::Neither(_) => Some(UFDRNumber::ONE + self.1),
            Chroma::Shade(c_prop) => Some(c_prop + self.1 * c_prop),
            Chroma::Tint(_) => Some(UFDRNumber::ONE + self.1 + UFDRNumber(1)),
        }
    }

    fn max_sum_for_chroma(&self, chroma: Chroma) -> Option<UFDRNumber> {
        debug_assert!(chroma.is_valid());
        match chroma {
            Chroma::ZERO => None,
            Chroma::Neither(_) => Some(UFDRNumber::ONE + self.1),
            Chroma::Shade(_) => Some(UFDRNumber::ONE + self.1 - UFDRNumber(1)),
            Chroma::Tint(c_prop) => self.max_sum_for_chroma_prop(c_prop),
        }
    }

    fn max_chroma_for_sum(&self, sum: UFDRNumber) -> Option<Chroma> {
        debug_assert!(sum.is_valid_sum(), "sum: {:?}", sum);
        if sum.is_hue_valid() {
            match sum.cmp(&(UFDRNumber::ONE + self.1)) {
                Ordering::Equal => Some(Chroma::ONE),
                Ordering::Less => {
                    let temp = sum / (UFDRNumber::ONE + self.1);
                    Some(Chroma::Shade(temp.into_prop()))
                }
                Ordering::Greater => {
                    // NB: make sure it doesn't round to one
                    let c_prop = ((UFDRNumber::THREE - sum) / (UFDRNumber::TWO - self.1))
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

    fn try_ordered_triplet(
        &self,
        sum: UFDRNumber,
        c_prop: Prop,
    ) -> Option<Result<[Prop; 3], [Prop; 3]>> {
        debug_assert!(sum.is_valid_sum());
        match self.sum_for_max_chroma() * c_prop {
            UFDRNumber::ZERO => None,
            min_sum if sum >= min_sum => {
                let third = (sum - min_sum) / 3;
                match third + c_prop {
                    first if !first.is_proportion() => None,
                    first => {
                        let second = sum - first - third;
                        match second.cmp(&first) {
                            Ordering::Less => {
                                debug_assert_eq!(first + second + third, sum);
                                debug_assert_eq!(first - third, c_prop.into());
                                let triplet = [first.to_prop(), second.to_prop(), third.to_prop()];
                                debug_assert!(first > second && second > third);
                                if Self::calculate_hue_parameter(triplet) == self.1 {
                                    Some(Ok(triplet))
                                } else {
                                    Some(Err(triplet))
                                }
                            }
                            Ordering::Equal => {
                                debug_assert_eq!(first + second + third, sum);
                                debug_assert_eq!(first - third, c_prop.into());
                                debug_assert!(second > third);
                                Some(Err([first.to_prop(), second.to_prop(), third.to_prop()]))
                            }
                            Ordering::Greater => {
                                if second.is_proportion() {
                                    let first = first - UFDRNumber(1);
                                    let third = third + UFDRNumber(1);
                                    debug_assert!(second > third && first > third);
                                    debug_assert_eq!(second - third, c_prop.into());
                                    debug_assert_eq!(first + second + third, sum);
                                    Some(Err([second.to_prop(), first.to_prop(), third.to_prop()]))
                                } else {
                                    None
                                }
                            }
                        }
                    }
                }
            }
            _ => None,
        }
    }
}

impl ColourModificationHelpers for SextantHue {}

impl SumChromaCompatibility for SextantHue {
    fn sum_and_chroma_prop_are_compatible(&self, sum: UFDRNumber, c_prop: Prop) -> bool {
        debug_assert!(sum.is_valid_sum());
        match c_prop + self.1 * c_prop {
            min_sum if sum < min_sum || min_sum == UFDRNumber::ZERO => false,
            min_sum => {
                let third = (sum - min_sum) / 3;
                match third + c_prop {
                    first if first.is_proportion() => {
                        let second = sum - first - third;
                        if first > second && second > third {
                            Self::calculate_hue_parameter([
                                first.into_prop(),
                                second.into_prop(),
                                third.into_prop(),
                            ]) == self.1
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            }
        }
    }

    fn sum_and_chroma_are_compatible(&self, sum: UFDRNumber, chroma: Chroma) -> bool {
        debug_assert!(chroma.is_valid() && sum.is_valid_sum());
        chroma.is_valid_re((*self).into(), sum)
            && self.sum_and_chroma_prop_are_compatible(sum, chroma.into_prop())
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

    fn calculate_hue_parameter(triplet: [Prop; 3]) -> Prop {
        let [first, second, third] = triplet;
        debug_assert!(first > second && second > third);
        let mut sum_for_max_chroma = UFDRNumber::ONE + ((second - third) / (first - third));
        // Handle possible (fatal) rounding error
        while sum_for_max_chroma > UFDRNumber::ONE + UFDRNumber(1)
            && sum_for_max_chroma * (first - third) > first + second + third
        {
            sum_for_max_chroma = sum_for_max_chroma - UFDRNumber(1);
        }
        while sum_for_max_chroma > UFDRNumber::ONE + UFDRNumber(1)
            && (first + second + third - sum_for_max_chroma * (first - third)) / 3 < third.into()
        {
            sum_for_max_chroma = sum_for_max_chroma - UFDRNumber(1);
        }
        debug_assert!(sum_for_max_chroma > UFDRNumber::ONE && sum_for_max_chroma < UFDRNumber::TWO);
        debug_assert_eq!(
            (first + second + third - sum_for_max_chroma * (first - third)) / 3,
            third.into()
        );
        (sum_for_max_chroma - UFDRNumber::ONE).into_prop()
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
        let hue_param = Self::calculate_hue_parameter(arg.1);
        Self(arg.0, hue_param)
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
        debug_assert!(chroma.is_valid());
        let kc = chroma.into_prop() * self.1;
        let x_dash = match self.0 {
            // TODO: take tint and shade into account
            Sextant::RedYellow | Sextant::RedMagenta => {
                (UFDRNumber::TWO + chroma.into_prop() * 2 - kc) / 4
            }
            Sextant::GreenYellow | Sextant::BlueMagenta => {
                (UFDRNumber::TWO + kc * 2 - chroma.into_prop()) / 4
            }
            Sextant::GreenCyan | Sextant::BlueCyan => {
                (UFDRNumber::TWO - kc - chroma.into_prop()) / 4
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

    fn try_rgb_ordered_triplet(
        &self,
        sum: UFDRNumber,
        c_prop: Prop,
    ) -> Option<Result<[Prop; 3], [Prop; 3]>> {
        match self {
            Self::Primary(primary_hue) => primary_hue.try_rgb_ordered_triplet(sum, c_prop),
            Self::Secondary(secondary_hue) => secondary_hue.try_rgb_ordered_triplet(sum, c_prop),
            Self::Sextant(sextant_hue) => sextant_hue.try_rgb_ordered_triplet(sum, c_prop),
        }
    }

    fn ordered_triplet(&self, sum: UFDRNumber, c_prop: Prop) -> Option<[Prop; 3]> {
        match self {
            Self::Primary(primary_hue) => primary_hue.ordered_triplet(sum, c_prop),
            Self::Secondary(secondary_hue) => secondary_hue.ordered_triplet(sum, c_prop),
            Self::Sextant(sextant_hue) => sextant_hue.ordered_triplet(sum, c_prop),
        }
    }

    fn try_ordered_triplet(
        &self,
        sum: UFDRNumber,
        c_prop: Prop,
    ) -> Option<Result<[Prop; 3], [Prop; 3]>> {
        match self {
            Self::Primary(primary_hue) => primary_hue.try_ordered_triplet(sum, c_prop),
            Self::Secondary(secondary_hue) => secondary_hue.try_ordered_triplet(sum, c_prop),
            Self::Sextant(sextant_hue) => sextant_hue.try_ordered_triplet(sum, c_prop),
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
    fn trim_overs(&self, sum: UFDRNumber, c_prop: Prop) -> Option<(Prop, UFDRNumber)> {
        match self {
            Self::Primary(primary_hue) => primary_hue.trim_overs(sum, c_prop),
            Self::Secondary(secondary_hue) => secondary_hue.trim_overs(sum, c_prop),
            Self::Sextant(sextant_hue) => sextant_hue.trim_overs(sum, c_prop),
        }
    }

    fn adjusted_favouring_chroma(
        &self,
        sum: UFDRNumber,
        c_prop: Prop,
    ) -> Option<(Prop, UFDRNumber)> {
        match self {
            Self::Primary(primary_hue) => primary_hue.adjusted_favouring_chroma(sum, c_prop),
            Self::Secondary(secondary_hue) => secondary_hue.adjusted_favouring_chroma(sum, c_prop),
            Self::Sextant(sextant_hue) => sextant_hue.adjusted_favouring_chroma(sum, c_prop),
        }
    }

    fn adjusted_favouring_sum(&self, sum: UFDRNumber, c_prop: Prop) -> Option<(Prop, UFDRNumber)> {
        match self {
            Self::Primary(primary_hue) => primary_hue.adjusted_favouring_sum(sum, c_prop),
            Self::Secondary(secondary_hue) => secondary_hue.adjusted_favouring_sum(sum, c_prop),
            Self::Sextant(sextant_hue) => sextant_hue.adjusted_favouring_sum(sum, c_prop),
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

    fn darkest_rgb_for_chroma<T: LightLevel>(
        &self,
        chroma: Chroma,
    ) -> Option<Result<RGB<T>, RGB<T>>> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.darkest_rgb_for_chroma(chroma),
            Self::Secondary(cmy_hue) => cmy_hue.darkest_rgb_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.darkest_rgb_for_chroma(chroma),
        }
    }

    fn lightest_rgb_for_chroma<T: LightLevel>(
        &self,
        chroma: Chroma,
    ) -> Option<Result<RGB<T>, RGB<T>>> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.lightest_rgb_for_chroma(chroma),
            Self::Secondary(cmy_hue) => cmy_hue.lightest_rgb_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.lightest_rgb_for_chroma(chroma),
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

impl PropDiff for Hue {
    fn prop_diff(&self, other: &Self) -> Option<Prop> {
        match self {
            Self::Primary(self_hue) => match other {
                Self::Primary(other_hue) => self_hue.prop_diff(other_hue),
                Self::Sextant(sextant_hue) => self_hue.prop_diff_sextant(sextant_hue),
                _ => None,
            },
            Self::Secondary(self_hue) => match other {
                Self::Secondary(other_hue) => self_hue.prop_diff(other_hue),
                Self::Sextant(sextant_hue) => self_hue.prop_diff_sextant(sextant_hue),
                _ => None,
            },
            Self::Sextant(self_hue) => match other {
                Self::Sextant(other_hue) => self_hue.prop_diff(other_hue),
                Self::Primary(primary_hue) => primary_hue.prop_diff_sextant(self_hue),
                Self::Secondary(secondary_hue) => secondary_hue.prop_diff_sextant(self_hue),
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
