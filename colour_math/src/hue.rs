// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    cmp::Ordering,
    convert::{Into, TryFrom},
    fmt::Debug,
};

pub mod angle;

use crate::{
    fdrn::UFDRNumber, hue::angle::Angle, proportion::Warmth, Chroma, HueConstants, LightLevel,
    Prop, RGBConstants, HCV, RGB,
};

use crate::fdrn::FDRNumber;
use num_traits_plus::{debug_assert_approx_eq, float_plus::FloatPlus};
use std::ops::{Add, Sub};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct SunRange {
    pub min: UFDRNumber,
    pub max_chroma_sum: UFDRNumber,
    pub max: UFDRNumber,
}

impl From<(UFDRNumber, UFDRNumber, UFDRNumber)> for SunRange {
    fn from(tuple: (UFDRNumber, UFDRNumber, UFDRNumber)) -> Self {
        debug_assert!(tuple.0.is_hue_valid() && tuple.1.is_hue_valid() && tuple.2.is_hue_valid());
        debug_assert!(tuple.0 <= tuple.1 && tuple.1 <= tuple.2);
        Self {
            min: tuple.0,
            max_chroma_sum: tuple.1,
            max: tuple.2,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum SumOrdering {
    TooSmall(UFDRNumber),
    Shade(UFDRNumber, UFDRNumber),
    Neither(UFDRNumber),
    Tint(UFDRNumber, UFDRNumber),
    TooBig(UFDRNumber),
}

impl SumOrdering {
    pub fn is_failure(&self) -> bool {
        use SumOrdering::*;
        match self {
            TooSmall(_) | TooBig(_) => true,
            _ => false,
        }
    }

    pub fn is_success(&self) -> bool {
        use SumOrdering::*;
        match self {
            TooSmall(_) | TooBig(_) => false,
            _ => true,
        }
    }
}

impl SunRange {
    pub fn compare_sum(&self, sum: UFDRNumber) -> SumOrdering {
        if sum < self.min {
            SumOrdering::TooSmall(self.min - sum)
        } else if sum < self.max_chroma_sum - UFDRNumber(1) {
            SumOrdering::Shade(self.min, self.max_chroma_sum - UFDRNumber(2))
        } else if sum > self.max_chroma_sum + UFDRNumber(1) {
            if sum <= self.max {
                SumOrdering::Tint(self.max_chroma_sum + UFDRNumber(2), self.max)
            } else {
                SumOrdering::TooBig(sum - self.max)
            }
        } else {
            SumOrdering::Neither(self.max_chroma_sum)
        }
    }

    pub fn shade_min(&self) -> UFDRNumber {
        self.min
    }

    pub fn shade_max(&self) -> UFDRNumber {
        self.max_chroma_sum
    }

    pub fn crossover(&self) -> UFDRNumber {
        self.max_chroma_sum
    }

    pub fn tint_min(&self) -> UFDRNumber {
        self.max_chroma_sum
    }

    pub fn tint_max(&self) -> UFDRNumber {
        self.max
    }
}

pub(crate) trait HueBasics: Debug + Sized {
    fn sum_for_max_chroma(&self) -> UFDRNumber;
    fn to_hue(&self) -> Hue;

    fn min_sum_for_chroma_prop(&self, c_prop: Prop) -> Option<UFDRNumber> {
        match c_prop {
            Prop::ZERO => None,
            c_prop => Some(self.sum_for_max_chroma() * c_prop),
        }
    }

    fn max_sum_for_chroma_prop(&self, c_prop: Prop) -> Option<UFDRNumber> {
        match c_prop {
            Prop::ZERO => None,
            c_prop => {
                Some(UFDRNumber::THREE - (UFDRNumber::THREE - self.sum_for_max_chroma()) * c_prop)
            }
        }
    }

    fn sum_range_for_chroma_prop(&self, c_prop: Prop) -> Option<(UFDRNumber, UFDRNumber)> {
        let min = self.min_sum_for_chroma_prop(c_prop)?;
        let max = self.max_sum_for_chroma_prop(c_prop)?;
        Some((min, max))
    }

    fn max_chroma_prop_for_sum(&self, sum: UFDRNumber) -> Option<Prop> {
        debug_assert!(sum.is_valid_sum(), "sum: {:?}", sum);
        if sum.is_hue_valid() {
            match sum.cmp(&self.sum_for_max_chroma()) {
                Ordering::Equal => Some(Prop::ONE),
                Ordering::Less => {
                    let temp = sum / self.sum_for_max_chroma();
                    Some(temp.into())
                }
                Ordering::Greater => {
                    let temp =
                        (UFDRNumber::THREE - sum) / (UFDRNumber::THREE - self.sum_for_max_chroma());
                    Some(temp.into())
                }
            }
        } else {
            None
        }
    }
}

pub(crate) trait OrderedTriplets: HueBasics {
    fn ordered_triplet_is_valid(&self, triplet: &[Prop; 3]) -> bool;
    fn rgb_ordered_triplet_for_sum_and_chroma(
        &self,
        sum: UFDRNumber,
        c_prop: Prop,
    ) -> Option<[Prop; 3]>;

    fn ordered_triplet_for_sum_and_chroma(
        &self,
        sum: UFDRNumber,
        c_prop: Prop,
    ) -> Option<[Prop; 3]> {
        if sum >= self.sum_for_max_chroma() * c_prop {
            let third = (sum - self.sum_for_max_chroma() * c_prop) / 3;
            let first = third + c_prop;
            if first.is_proportion() {
                let second = sum - first - third;
                debug_assert_eq!(first + second + third, sum);
                debug_assert_eq!(first - third, c_prop.into());
                debug_assert_approx_eq!(first + second + third, sum_for_max_chroma * chroma.prop());
                let triplet = [first.to_prop(), second.to_prop(), third.to_prop()];
                debug_assert!(self.ordered_triplet_is_valid(&triplet));
                Some(triplet)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn ordered_triplet_to_hcv(&self, triplet: &[Prop; 3]) -> HCV {
        debug_assert!(self.ordered_triplet_is_valid(triplet));
        let sum = triplet[0] + triplet[1] + triplet[2];
        let c_prop = triplet[0] - triplet[2];
        let chroma = match sum.cmp(&self.sum_for_max_chroma()) {
            Ordering::Less => Chroma::Shade(c_prop),
            Ordering::Equal => Chroma::Neither(c_prop),
            Ordering::Greater => Chroma::Tint(c_prop),
        };
        HCV {
            hue: Some(self.to_hue()),
            chroma,
            sum,
        }
    }
}

pub(crate) trait HueIfce: HueBasics {
    fn angle(&self) -> Angle;
    fn max_chroma_for_sum(&self, sum: UFDRNumber) -> Option<Chroma>;
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

    fn max_chroma_rgb<T: LightLevel>(&self) -> RGB<T>;
    fn max_chroma_hcv(&self) -> HCV;
    fn max_chroma_rgb_for_sum<T: LightLevel>(&self, sum: UFDRNumber) -> Option<RGB<T>>;
    fn min_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Chroma) -> RGB<T>;
    fn max_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Chroma) -> RGB<T>;
    fn rgb_for_sum_and_chroma_prop<T: LightLevel>(
        &self,
        sum: UFDRNumber,
        chroma: Chroma,
    ) -> Option<RGB<T>>;
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

impl HueBasics for RGBHue {
    fn sum_for_max_chroma(&self) -> UFDRNumber {
        UFDRNumber::ONE
    }

    fn to_hue(&self) -> Hue {
        Hue::Primary(*self)
    }
}

impl OrderedTriplets for RGBHue {
    fn ordered_triplet_is_valid(&self, triplet: &[Prop; 3]) -> bool {
        triplet[0] > triplet[1] && triplet[1] == triplet[2]
    }

    fn rgb_ordered_triplet_for_sum_and_chroma(
        &self,
        sum: UFDRNumber,
        c_prop: Prop,
    ) -> Option<[Prop; 3]> {
        let triplet = self.ordered_triplet_for_sum_and_chroma(sum, c_prop)?;
        debug_assert!(self.ordered_triplet_is_valid(&triplet));
        use RGBHue::*;
        match self {
            Red => Some(triplet),
            Green => Some([triplet[1], triplet[0], triplet[2]]),
            Blue => Some([triplet[2], triplet[1], triplet[0]]),
        }
    }
}

impl ColourModificationHelpers for RGBHue {}

impl RGBHue {
    fn make_rgb<T: LightLevel>(&self, components: (Prop, Prop)) -> RGB<T> {
        use RGBHue::*;
        match self {
            Red => [components.0, components.1, components.1].into(),
            Green => [components.1, components.0, components.1].into(),
            Blue => [components.1, components.1, components.0].into(),
        }
    }

    pub fn sum_range_for_chroma(&self, chroma: Chroma) -> Option<(UFDRNumber, UFDRNumber)> {
        match chroma {
            Chroma::ZERO => None,
            Chroma::Neither(_) => Some((UFDRNumber::ONE, UFDRNumber::ONE)),
            Chroma::Shade(c_prop) => Some((c_prop.into(), UFDRNumber::ALMOST_ONE)),
            Chroma::Tint(c_prop) => {
                Some((UFDRNumber::JUST_OVER_ONE, UFDRNumber::THREE - c_prop * 2))
            }
        }
    }

    pub fn sum_and_chroma_are_compatible(&self, sum: UFDRNumber, chroma: Chroma) -> bool {
        if let Some((min_sum, max_sum)) = self.sum_range_for_chroma(chroma) {
            sum >= min_sum && sum <= max_sum && (sum - chroma.prop()) % 3 == UFDRNumber::ZERO
        } else {
            false
        }
    }

    pub fn adjusted_chroma_for_sum_compatibility(
        &self,
        c_prop: Prop,
        sum: UFDRNumber,
    ) -> Option<(Chroma, UFDRNumber)> {
        if sum.is_hue_valid() && c_prop > Prop::ZERO {
            match sum.cmp(&UFDRNumber::ONE) {
                Ordering::Equal => Some((Chroma::Neither(c_prop), sum)),
                Ordering::Less => {
                    let min_sum = self.min_sum_for_chroma_prop(c_prop).expect("c_prop > 0");
                    if sum < min_sum {
                        Some((Chroma::Shade(sum.into()), sum))
                    } else {
                        Some((Chroma::Shade(c_prop), sum))
                    }
                }
                Ordering::Greater => {
                    let max_sum = self.max_sum_for_chroma_prop(c_prop).expect("c_prop > 0");
                    if sum > max_sum {
                        let new_c_prop: Prop = ((UFDRNumber::THREE - sum) / 2).into();
                        if new_c_prop == Prop::ZERO {
                            None
                        } else {
                            Some((Chroma::Tint(new_c_prop), sum))
                        }
                    } else {
                        Some((Chroma::Tint(c_prop), sum))
                    }
                }
            }
        } else {
            None
        }
    }

    pub fn adjusted_sum_and_chroma_for_chroma_compatibility(
        &self,
        c_prop: Prop,
        sum: UFDRNumber,
    ) -> Option<(UFDRNumber, Chroma)> {
        match c_prop {
            Prop::ZERO => None,
            Prop::ONE => Some((UFDRNumber::ONE, Chroma::ONE)),
            c_prop => match sum.cmp(&UFDRNumber::ONE) {
                Ordering::Equal => Some((sum, Chroma::Neither(c_prop))),
                Ordering::Less => {
                    let min_sum = self.min_sum_for_chroma_prop(c_prop).expect("c_prop > 0");
                    if sum < min_sum {
                        Some((min_sum, Chroma::Shade(c_prop)))
                    } else {
                        Some((sum, Chroma::Shade(c_prop)))
                    }
                }
                Ordering::Greater => {
                    let max_sum = self.max_sum_for_chroma_prop(c_prop).expect("c_prop > 0");
                    if sum > max_sum {
                        Some((max_sum, Chroma::Tint(c_prop)))
                    } else {
                        Some((sum, Chroma::Tint(c_prop)))
                    }
                }
            },
        }
    }

    pub fn array_for_sum_and_chroma(&self, sum: UFDRNumber, chroma: Chroma) -> Option<[Prop; 3]> {
        debug_assert!(sum.is_valid_sum());
        let sum_for_max_chroma = UFDRNumber::ONE;
        let (first, other) = match sum.cmp(&sum_for_max_chroma) {
            Ordering::Equal => match chroma {
                Chroma::ZERO => return None,
                Chroma::Neither(c_prop) => {
                    let other = (sum - c_prop) / 3;
                    (other + c_prop, other)
                }
                _ => return None,
            },
            Ordering::Less => match chroma {
                Chroma::Shade(c_prop) => {
                    if sum < c_prop.into() {
                        return None;
                    } else {
                        let other = (sum - c_prop) / 3;
                        (other + c_prop, other)
                    }
                }
                _ => return None,
            },
            Ordering::Greater => match chroma {
                Chroma::Tint(c_prop) => {
                    if sum <= self.max_sum_for_chroma_prop(c_prop).expect("c_prop > 0") {
                        let other = (sum - c_prop) / 3;
                        (other + c_prop, other)
                    } else {
                        return None;
                    }
                }
                _ => return None,
            },
        };
        if first.is_proportion() {
            debug_assert!(first > other);
            debug_assert_eq!(first + other * 2, sum);
            debug_assert_eq!(first - other, chroma.prop().into());
            let p_first: Prop = first.into();
            let p_other: Prop = other.into();
            match self {
                RGBHue::Red => Some([p_first, p_other, p_other]),
                RGBHue::Green => Some([p_other, p_first, p_other]),
                RGBHue::Blue => Some([p_other, p_other, p_first]),
            }
        } else {
            None
        }
    }
}

impl HueIfce for RGBHue {
    fn angle(&self) -> Angle {
        match self {
            RGBHue::Red => Angle::RED,
            RGBHue::Green => Angle::GREEN,
            RGBHue::Blue => Angle::BLUE,
        }
    }

    fn max_chroma_for_sum(&self, sum: UFDRNumber) -> Option<Chroma> {
        debug_assert!(sum.is_valid_sum(), "sum: {:?}", sum);
        if sum == UFDRNumber::ZERO || sum == UFDRNumber::THREE {
            None
        } else if sum < UFDRNumber::ONE {
            Some(Chroma::Shade(sum.into()))
        } else if sum > UFDRNumber::ONE {
            Some(Chroma::Tint(((UFDRNumber::THREE - sum) / 2).into()))
        } else {
            Some(Chroma::ONE)
        }
    }

    fn min_sum_for_chroma(&self, chroma: Chroma) -> Option<UFDRNumber> {
        match chroma {
            Chroma::ZERO => None,
            Chroma::Neither(_) => Some(UFDRNumber::ONE),
            Chroma::Shade(c_prop) => Some(c_prop.into()),
            Chroma::Tint(_) => Some(UFDRNumber::JUST_OVER_ONE),
        }
    }

    fn max_sum_for_chroma(&self, chroma: Chroma) -> Option<UFDRNumber> {
        match chroma {
            Chroma::ZERO => None,
            Chroma::Neither(_) => Some(UFDRNumber::ONE),
            Chroma::Shade(_) => Some(UFDRNumber::ALMOST_ONE),
            Chroma::Tint(c_prop) => Some(UFDRNumber::THREE - c_prop * 2),
        }
    }

    fn warmth_for_chroma(&self, chroma: Chroma) -> Warmth {
        let x_dash = match self {
            RGBHue::Red => ((UFDRNumber::ONE + chroma.prop()) / 2).into(),
            RGBHue::Green | RGBHue::Blue => ((UFDRNumber::TWO - chroma.prop()) / 4).into(),
        };
        Warmth::calculate(chroma, x_dash)
    }

    fn max_chroma_rgb<T: LightLevel>(&self) -> RGB<T> {
        match self {
            RGBHue::Red => RGB::RED,
            RGBHue::Green => RGB::GREEN,
            RGBHue::Blue => RGB::BLUE,
        }
    }

    fn max_chroma_hcv(&self) -> HCV {
        match self {
            RGBHue::Red => HCV::RED,
            RGBHue::Green => HCV::GREEN,
            RGBHue::Blue => HCV::BLUE,
        }
    }

    fn max_chroma_rgb_for_sum<T: LightLevel>(&self, sum: UFDRNumber) -> Option<RGB<T>> {
        debug_assert!(sum.is_valid_sum(), "sum: {:?}", sum);
        if sum == UFDRNumber::ZERO || sum == UFDRNumber::THREE {
            None
        } else {
            if sum <= UFDRNumber::ONE {
                Some(self.make_rgb((sum.into(), Prop::ZERO)))
            } else {
                Some(self.make_rgb((Prop::ONE, ((sum - UFDRNumber::ONE) / 2).into())))
            }
        }
    }

    fn min_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Chroma) -> RGB<T> {
        match chroma.prop() {
            Prop::ZERO => RGB::<T>::BLACK,
            Prop::ONE => self.max_chroma_rgb(),
            prop => self.make_rgb((prop, Prop::ZERO)),
        }
    }

    fn max_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Chroma) -> RGB<T> {
        match chroma.prop() {
            Prop::ZERO => RGB::<T>::WHITE,
            Prop::ONE => self.max_chroma_rgb(),
            prop => self.make_rgb((Prop::ONE, Prop::ONE - prop)),
        }
    }

    fn rgb_for_sum_and_chroma_prop<T: LightLevel>(
        &self,
        sum: UFDRNumber,
        chroma: Chroma,
    ) -> Option<RGB<T>> {
        debug_assert!(sum.is_valid_sum(), "sum: {:?}", sum);
        match chroma.prop() {
            Prop::ZERO => None,
            c_prop => match sum.cmp(&c_prop.into()) {
                Ordering::Less => None,
                Ordering::Equal => Some(self.make_rgb::<T>((c_prop, Prop::ZERO))),
                Ordering::Greater => {
                    // NB: adjusting for rounding errors is proving elusive so we take the easiest
                    // option of having accurate chroma and up to 2 least significant errors in
                    // sum for the generated RGB (but we can adjust the UFDRNumber test to avoid unnecessary
                    // None returns.
                    let other = (sum - c_prop) / 3;
                    let first = other + c_prop;
                    // NB: Need to check that UFDRNumber wasn't too big
                    if first <= UFDRNumber::ONE {
                        assert_eq!((first.0 - other.0) as u64, c_prop.0);
                        assert!(sum.abs_diff(&(first + other * 2)) < UFDRNumber(3));
                        Some(self.make_rgb::<T>((first.into(), other.into())))
                    } else {
                        None
                    }
                }
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord)]
pub enum CMYHue {
    Cyan = 113,
    Magenta = 3,
    Yellow = 7,
}

impl HueBasics for CMYHue {
    fn sum_for_max_chroma(&self) -> UFDRNumber {
        UFDRNumber::TWO
    }

    fn to_hue(&self) -> Hue {
        Hue::Secondary(*self)
    }
}

impl OrderedTriplets for CMYHue {
    fn ordered_triplet_is_valid(&self, triplet: &[Prop; 3]) -> bool {
        triplet[0] == triplet[1] && triplet[1] > triplet[2]
    }

    fn rgb_ordered_triplet_for_sum_and_chroma(
        &self,
        sum: UFDRNumber,
        c_prop: Prop,
    ) -> Option<[Prop; 3]> {
        let triplet = self.ordered_triplet_for_sum_and_chroma(sum, c_prop)?;
        debug_assert!(self.ordered_triplet_is_valid(&triplet));
        use CMYHue::*;
        match self {
            Cyan => Some([triplet[2], triplet[0], triplet[1]]),
            Magenta => Some([triplet[0], triplet[2], triplet[1]]),
            Yellow => Some(triplet),
        }
    }
}

impl ColourModificationHelpers for CMYHue {}

impl CMYHue {
    fn make_rgb<T: LightLevel>(&self, components: (Prop, Prop)) -> RGB<T> {
        use CMYHue::*;
        match self {
            Cyan => [components.1, components.0, components.0].into(),
            Magenta => [components.0, components.1, components.0].into(),
            Yellow => [components.0, components.0, components.1].into(),
        }
    }

    pub fn sum_range_for_chroma(&self, chroma: Chroma) -> Option<(UFDRNumber, UFDRNumber)> {
        match chroma {
            Chroma::ZERO => None,
            Chroma::Neither(_) => Some((UFDRNumber::TWO, UFDRNumber::TWO)),
            Chroma::Shade(c_prop) => Some((
                (c_prop * 2).min(UFDRNumber::ALMOST_TWO),
                UFDRNumber::ALMOST_TWO,
            )),
            Chroma::Tint(c_prop) => Some((UFDRNumber::JUST_OVER_TWO, UFDRNumber::THREE - c_prop)),
        }
    }

    pub fn sum_and_chroma_are_compatible(&self, sum: UFDRNumber, chroma: Chroma) -> bool {
        if let Some((min_sum, max_sum)) = self.sum_range_for_chroma(chroma) {
            sum >= min_sum && sum <= max_sum && (sum - chroma.prop() * 2) % 3 == UFDRNumber::ZERO
        } else {
            false
        }
    }

    pub fn adjusted_chroma_for_sum_compatibility(
        &self,
        c_prop: Prop,
        sum: UFDRNumber,
    ) -> Option<(Chroma, UFDRNumber)> {
        if sum.is_hue_valid() && c_prop > Prop::ZERO {
            match sum.cmp(&UFDRNumber::TWO) {
                Ordering::Equal => Some((Chroma::Neither(c_prop), sum)),
                Ordering::Less => {
                    let min_sum = self.min_sum_for_chroma_prop(c_prop).expect("c_prop > 0");
                    if sum < min_sum {
                        let new_c_prop: Prop = (sum / 2).into();
                        if new_c_prop == Prop::ZERO {
                            None
                        } else {
                            // Rounding errors from the division can result in the chroma and sum
                            // still being incompatible so we'll fudge it
                            let new_min_sum = self
                                .min_sum_for_chroma_prop(new_c_prop)
                                .expect("new_c_prop > 0");
                            Some((Chroma::Shade(new_c_prop), new_min_sum))
                        }
                    } else {
                        Some((Chroma::Shade(c_prop), sum))
                    }
                }
                Ordering::Greater => {
                    let max_sum = self.max_sum_for_chroma_prop(c_prop).expect("c_prop > 0");
                    if sum > max_sum {
                        let new_c_prop: Prop = (UFDRNumber::THREE - sum).into();
                        if new_c_prop == Prop::ZERO {
                            None
                        } else {
                            // Rounding errors from the division can result in the chroma and sum
                            // still being incompatible so we'll fudge it
                            let new_max_sum = self
                                .max_sum_for_chroma_prop(new_c_prop)
                                .expect("new_c_prop > 0");
                            Some((Chroma::Tint(new_c_prop), new_max_sum))
                        }
                    } else {
                        Some((Chroma::Tint(c_prop), sum))
                    }
                }
            }
        } else {
            None
        }
    }

    pub fn adjusted_sum_and_chroma_for_chroma_compatibility(
        &self,
        c_prop: Prop,
        sum: UFDRNumber,
    ) -> Option<(UFDRNumber, Chroma)> {
        match c_prop {
            Prop::ZERO => None,
            Prop::ONE => Some((UFDRNumber::TWO, Chroma::ONE)),
            c_prop => match sum.cmp(&UFDRNumber::TWO) {
                Ordering::Equal => Some((sum, Chroma::Neither(c_prop))),
                Ordering::Less => {
                    let min_sum = self.min_sum_for_chroma_prop(c_prop).expect("c_prop > 0");
                    if sum < min_sum {
                        Some((min_sum, Chroma::Shade(c_prop)))
                    } else {
                        Some((sum, Chroma::Shade(c_prop)))
                    }
                }
                Ordering::Greater => {
                    let max_sum = self.max_sum_for_chroma_prop(c_prop).expect("c_prop > 0");
                    if sum > max_sum {
                        Some((max_sum, Chroma::Tint(c_prop)))
                    } else {
                        Some((sum, Chroma::Tint(c_prop)))
                    }
                }
            },
        }
    }

    pub fn array_for_sum_and_chroma(&self, sum: UFDRNumber, chroma: Chroma) -> Option<[Prop; 3]> {
        debug_assert!(sum.is_valid_sum());
        let sum_for_max_chroma = UFDRNumber::TWO;
        let (primary, other) = match sum.cmp(&sum_for_max_chroma) {
            Ordering::Equal => match chroma {
                Chroma::ZERO => return None,
                Chroma::Neither(c_prop) => {
                    let other = (sum - c_prop * 2) / 3;
                    (other + c_prop, other)
                }
                _ => {
                    return None;
                }
            },
            Ordering::Less => match chroma {
                Chroma::Shade(c_prop) => {
                    if sum < c_prop * 2 {
                        return None;
                    } else {
                        let other = (sum - c_prop * 2) / 3;
                        (other + c_prop, other)
                    }
                }
                _ => {
                    return None;
                }
            },
            Ordering::Greater => match chroma {
                Chroma::Tint(c_prop) => {
                    if sum <= self.max_sum_for_chroma_prop(c_prop).expect("c_prop > 0") {
                        let other = (sum - c_prop * 2) / 3;
                        (other + c_prop, other)
                    } else {
                        return None;
                    }
                }
                _ => {
                    return None;
                }
            },
        };
        if primary.is_proportion() {
            debug_assert!(primary > other);
            debug_assert_eq!(primary * 2 + other, sum);
            debug_assert_eq!(primary - other, chroma.prop().into());
            let p_primary: Prop = primary.into();
            let p_other: Prop = other.into();
            match self {
                CMYHue::Cyan => Some([p_other, p_primary, p_primary]),
                CMYHue::Magenta => Some([p_primary, p_other, p_primary]),
                CMYHue::Yellow => Some([p_primary, p_primary, p_other]),
            }
        } else {
            None
        }
    }
}

impl HueIfce for CMYHue {
    fn angle(&self) -> Angle {
        match self {
            CMYHue::Cyan => Angle::CYAN,
            CMYHue::Magenta => Angle::MAGENTA,
            CMYHue::Yellow => Angle::YELLOW,
        }
    }

    fn max_chroma_for_sum(&self, sum: UFDRNumber) -> Option<Chroma> {
        debug_assert!(sum.is_valid_sum(), "sum: {:?}", sum);
        if sum == UFDRNumber::ZERO || sum == UFDRNumber::THREE {
            None
        } else if sum < UFDRNumber::TWO {
            Some(Chroma::Shade((sum / 2).into()))
        } else if sum > UFDRNumber::TWO {
            Some(Chroma::Tint((UFDRNumber::THREE - sum).into()))
        } else {
            Some(Chroma::ONE)
        }
    }

    fn min_sum_for_chroma(&self, chroma: Chroma) -> Option<UFDRNumber> {
        match chroma {
            Chroma::ZERO => None,
            Chroma::Neither(_) => Some(UFDRNumber::TWO),
            Chroma::Shade(c_prop) => Some((c_prop * 2).min(UFDRNumber::ALMOST_TWO)),
            Chroma::Tint(_) => Some(UFDRNumber::JUST_OVER_TWO),
        }
    }

    fn max_sum_for_chroma(&self, chroma: Chroma) -> Option<UFDRNumber> {
        match chroma {
            Chroma::ZERO => None,
            Chroma::Neither(_) => Some(UFDRNumber::TWO),
            Chroma::Shade(_) => Some(UFDRNumber::ALMOST_TWO),
            Chroma::Tint(c_prop) => Some(UFDRNumber::THREE - c_prop),
        }
    }

    fn warmth_for_chroma(&self, chroma: Chroma) -> Warmth {
        let x_dash = match self {
            CMYHue::Cyan => (UFDRNumber::ONE - chroma.prop()) / 2,
            CMYHue::Magenta | CMYHue::Yellow => (UFDRNumber::TWO + chroma.prop()) / 4,
        };
        Warmth::calculate(chroma, x_dash.into())
    }

    fn max_chroma_rgb<T: LightLevel>(&self) -> RGB<T> {
        match self {
            CMYHue::Cyan => RGB::CYAN,
            CMYHue::Magenta => RGB::MAGENTA,
            CMYHue::Yellow => RGB::YELLOW,
        }
    }

    fn max_chroma_hcv(&self) -> HCV {
        match self {
            CMYHue::Cyan => HCV::CYAN,
            CMYHue::Magenta => HCV::MAGENTA,
            CMYHue::Yellow => HCV::YELLOW,
        }
    }

    fn max_chroma_rgb_for_sum<T: LightLevel>(&self, sum: UFDRNumber) -> Option<RGB<T>> {
        debug_assert!(sum.is_valid_sum(), "sum: {:?}", sum);
        if sum == UFDRNumber::ZERO || sum == UFDRNumber::THREE {
            None
        } else if sum < UFDRNumber::TWO {
            Some(self.make_rgb(((sum / 2).into(), Prop::ZERO)))
        } else if sum > UFDRNumber::TWO {
            Some(self.make_rgb((Prop::ONE, (sum - UFDRNumber::TWO).into())))
        } else {
            Some(self.max_chroma_rgb())
        }
    }

    fn min_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Chroma) -> RGB<T> {
        match chroma.prop() {
            Prop::ZERO => RGB::<T>::BLACK,
            Prop::ONE => self.max_chroma_rgb(),
            prop => self.make_rgb((prop, Prop::ZERO)),
        }
    }

    fn max_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Chroma) -> RGB<T> {
        match chroma.prop() {
            Prop::ZERO => RGB::<T>::WHITE,
            Prop::ONE => self.max_chroma_rgb(),
            prop => self.make_rgb((Prop::ONE, Prop::ONE - prop)),
        }
    }

    fn rgb_for_sum_and_chroma_prop<T: LightLevel>(
        &self,
        sum: UFDRNumber,
        chroma: Chroma,
    ) -> Option<RGB<T>> {
        debug_assert!(sum.is_valid_sum(), "sum: {:?}", sum);
        match chroma.prop() {
            Prop::ZERO => None,
            c_prop => match sum.cmp(&(c_prop * 2)) {
                Ordering::Less => None,
                Ordering::Equal => Some(self.make_rgb::<T>((c_prop, Prop::ZERO))),
                Ordering::Greater => {
                    // NB: adjusting for rounding errors is proving elusive so we take the easiest
                    // option of having accurate chroma and up to 2 least significant errors in
                    // sum for the generated RGB (but we can adjust the UFDRNumber test to avoid unnecessary
                    // None returns.
                    let other = (sum - c_prop * 2) / 3;
                    let first = other + c_prop;
                    // NB: Need to check that UFDRNumber wasn't too big
                    if first <= UFDRNumber::ONE {
                        assert_eq!((first.0 - other.0) as u64, c_prop.0);
                        assert!(sum.abs_diff(&(first * 2 + other)) < UFDRNumber(3));
                        Some(self.make_rgb::<T>((first.into(), other.into())))
                    } else {
                        None
                    }
                }
            },
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

impl HueBasics for SextantHue {
    fn sum_for_max_chroma(&self) -> UFDRNumber {
        UFDRNumber::ONE + self.1
    }

    fn to_hue(&self) -> Hue {
        Hue::Sextant(*self)
    }
}

impl OrderedTriplets for SextantHue {
    fn ordered_triplet_is_valid(&self, triplet: &[Prop; 3]) -> bool {
        triplet[0] > triplet[1] && triplet[1] > triplet[2]
    }

    fn rgb_ordered_triplet_for_sum_and_chroma(
        &self,
        sum: UFDRNumber,
        c_prop: Prop,
    ) -> Option<[Prop; 3]> {
        let triplet = self.ordered_triplet_for_sum_and_chroma(sum, c_prop)?;
        debug_assert!(self.ordered_triplet_is_valid(&triplet));
        use Sextant::*;
        match self.0 {
            RedMagenta => Some([triplet[0], triplet[2], triplet[1]]),
            RedYellow => Some(triplet),
            GreenYellow => Some([triplet[1], triplet[0], triplet[2]]),
            GreenCyan => Some([triplet[2], triplet[0], triplet[1]]),
            BlueCyan => Some([triplet[2], triplet[1], triplet[0]]),
            BlueMagenta => Some([triplet[1], triplet[2], triplet[0]]),
        }
    }
}

impl ColourModificationHelpers for SextantHue {}

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

    fn make_rgb_sum<T: LightLevel>(
        &self,
        components: (UFDRNumber, UFDRNumber, UFDRNumber),
    ) -> RGB<T> {
        debug_assert!(
            components.0 <= UFDRNumber::ONE
                && components.1 <= UFDRNumber::ONE
                && components.2 <= UFDRNumber::ONE
        );
        self.make_rgb((
            components.0.into(),
            components.1.into(),
            components.2.into(),
        ))
    }

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

    pub fn sum_range_for_chroma(&self, chroma: Chroma) -> Option<(UFDRNumber, UFDRNumber)> {
        match chroma {
            Chroma::ZERO => None,
            Chroma::Neither(_) => Some((UFDRNumber::ONE + self.1, UFDRNumber::ONE + self.1)),
            Chroma::Shade(c_prop) => Some((
                ((UFDRNumber::ONE + self.1) * c_prop).min(UFDRNumber::ALMOST_ONE + self.1),
                UFDRNumber::ALMOST_ONE + self.1,
            )),
            Chroma::Tint(c_prop) => Some((
                UFDRNumber::JUST_OVER_ONE + self.1,
                UFDRNumber::THREE - (UFDRNumber::TWO - self.1) * c_prop,
            )),
        }
    }

    pub fn sum_and_chroma_are_compatible(&self, sum: UFDRNumber, chroma: Chroma) -> bool {
        if let Some((min_sum, max_sum)) = self.sum_range_for_chroma(chroma) {
            sum >= min_sum
                && sum <= max_sum
                && (sum - (UFDRNumber::ONE + self.1) * chroma.prop()) % 3 < UFDRNumber(3)
        } else {
            false
        }
    }

    pub fn adjusted_chroma_for_sum_compatibility(
        &self,
        c_prop: Prop,
        sum: UFDRNumber,
    ) -> Option<(Chroma, UFDRNumber)> {
        if sum.is_hue_valid() && c_prop > Prop::ZERO {
            match sum.cmp(&(UFDRNumber::ONE + self.1)) {
                Ordering::Equal => Some((Chroma::Neither(c_prop), sum)),
                Ordering::Less => {
                    let min_sum = self.min_sum_for_chroma_prop(c_prop).expect("c_prop > 0");
                    if sum < min_sum {
                        let new_c_prop: Prop = (sum / (Prop::ONE + self.1)).into();
                        // Rounding errors from the division can result in the chroma and sum
                        // still being incompatible so we'll fudge it
                        if let Some(new_min_sum) = self.min_sum_for_chroma_prop(new_c_prop) {
                            Some((Chroma::Shade(new_c_prop), new_min_sum))
                        } else {
                            None
                        }
                    } else {
                        Some((Chroma::Shade(c_prop), sum))
                    }
                }
                Ordering::Greater => {
                    let max_sum = self.max_sum_for_chroma_prop(c_prop).expect("c_prop > 0");
                    if sum > max_sum {
                        let new_c_prop: Prop =
                            ((UFDRNumber::THREE - sum) / (UFDRNumber::TWO - self.1)).into();
                        // Rounding errors from the division can result in the chroma and sum
                        // still being incompatible so we'll fudge
                        if let Some(new_max_sum) = self.max_sum_for_chroma_prop(new_c_prop) {
                            Some((Chroma::Tint(new_c_prop), new_max_sum))
                        } else {
                            None
                        }
                    } else {
                        Some((Chroma::Tint(c_prop), sum))
                    }
                }
            }
        } else {
            None
        }
    }

    pub fn adjusted_sum_and_chroma_for_chroma_compatibility(
        &self,
        c_prop: Prop,
        sum: UFDRNumber,
    ) -> Option<(UFDRNumber, Chroma)> {
        match c_prop {
            Prop::ZERO => None,
            Prop::ONE => Some((UFDRNumber::ONE + self.1, Chroma::ONE)),
            c_prop => match sum.cmp(&(UFDRNumber::ONE + self.1)) {
                Ordering::Equal => Some((sum, Chroma::Neither(c_prop))),
                Ordering::Less => {
                    let min_sum = self.min_sum_for_chroma_prop(c_prop).expect("c_prop > 0");
                    if sum < min_sum {
                        Some((min_sum, Chroma::Shade(c_prop)))
                    } else {
                        Some((sum, Chroma::Shade(c_prop)))
                    }
                }
                Ordering::Greater => {
                    let max_sum = self.max_sum_for_chroma_prop(c_prop).expect("c_prop > 0");
                    if sum > max_sum {
                        Some((max_sum, Chroma::Tint(c_prop)))
                    } else {
                        Some((sum, Chroma::Tint(c_prop)))
                    }
                }
            },
        }
    }

    pub fn array_for_sum_and_chroma(&self, sum: UFDRNumber, chroma: Chroma) -> Option<[Prop; 3]> {
        debug_assert!(sum.is_valid_sum());
        let sum_for_max_chroma = UFDRNumber::ONE + self.1;
        // check sum and chroma are compatible
        match sum.cmp(&sum_for_max_chroma) {
            Ordering::Equal => match chroma {
                Chroma::Neither(c_prop) => {
                    if c_prop == Prop::ZERO {
                        return None;
                    }
                }
                _ => return None,
            },
            Ordering::Less => match chroma {
                Chroma::Shade(c_prop) => {
                    if sum < sum_for_max_chroma * c_prop {
                        return None;
                    }
                }
                _ => return None,
            },
            Ordering::Greater => match chroma {
                Chroma::Tint(c_prop) => {
                    if sum > self.max_sum_for_chroma_prop(c_prop).expect("c_prop > 0") {
                        return None;
                    }
                }
                _ => return None,
            },
        }
        let third = (sum - sum_for_max_chroma * chroma.prop()) / 3;
        let first = third + chroma.prop();
        if first.is_proportion() {
            let second = sum - first - third;
            debug_assert!(first > second && second > third);
            debug_assert_eq!(first + second + third, sum);
            debug_assert_eq!(first - third, chroma.prop().into());
            // TODO: try to find way to eliminate this rounding error
            debug_assert_approx_eq!(first + second + third, sum_for_max_chroma * chroma.prop());
            let p_first: Prop = first.into();
            let p_second: Prop = second.into();
            let p_third: Prop = third.into();
            use Sextant::*;
            match self.0 {
                RedMagenta => Some([p_first, p_third, p_second]),
                RedYellow => Some([p_first, p_second, p_third]),
                GreenYellow => Some([p_second, p_first, p_third]),
                GreenCyan => Some([p_third, p_first, p_second]),
                BlueCyan => Some([p_third, p_second, p_first]),
                BlueMagenta => Some([p_second, p_third, p_first]),
            }
        } else {
            None
        }
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

    fn max_chroma_for_sum(&self, sum: UFDRNumber) -> Option<Chroma> {
        debug_assert!(sum.is_valid_sum(), "sum: {:?}", sum);
        if sum == UFDRNumber::ZERO || sum == UFDRNumber::THREE {
            None
        } else {
            match sum.cmp(&(UFDRNumber::ONE + self.1)) {
                Ordering::Less => {
                    let temp = sum / (Prop::ONE + self.1);
                    Some(Chroma::Shade(temp.into()))
                }
                Ordering::Greater => {
                    let temp = (UFDRNumber::THREE - sum) / (UFDRNumber::TWO - self.1);
                    Some(Chroma::Tint(temp.into()))
                }
                Ordering::Equal => Some(Chroma::ONE),
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

    fn max_chroma_rgb<T: LightLevel>(&self) -> RGB<T> {
        self.make_rgb((Prop::ONE, self.1, Prop::ZERO))
    }

    fn max_chroma_hcv(&self) -> HCV {
        HCV::new(Some((Hue::Sextant(*self), Chroma::ONE)), Prop::ONE + self.1)
    }

    fn max_chroma_rgb_for_sum<T: LightLevel>(&self, sum: UFDRNumber) -> Option<RGB<T>> {
        debug_assert!(sum.is_valid_sum(), "sum: {:?}", sum);
        if sum == UFDRNumber::ZERO || sum == UFDRNumber::THREE {
            None
        } else {
            let sum_for_max_chroma = UFDRNumber::ONE + self.1;
            match sum.cmp(&sum_for_max_chroma) {
                Ordering::Equal => Some(self.max_chroma_rgb()),
                Ordering::Greater => {
                    let third = (sum - sum_for_max_chroma) / (UFDRNumber::TWO - self.1);
                    let second = third + self.1 - third * self.1;
                    debug_assert!(second < UFDRNumber::ONE);
                    Some(self.make_rgb_sum((UFDRNumber::ONE, second, third)))
                }
                Ordering::Less => {
                    let ratio = sum / sum_for_max_chroma;
                    Some(self.make_rgb_sum((ratio, ratio * self.1, UFDRNumber::ZERO)))
                }
            }
        }
    }

    fn min_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Chroma) -> RGB<T> {
        match chroma.prop() {
            Prop::ZERO => RGB::<T>::BLACK,
            Prop::ONE => self.max_chroma_rgb(),
            c_prop => self.make_rgb((c_prop, self.1 * c_prop, Prop::ZERO)),
        }
    }

    fn max_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Chroma) -> RGB<T> {
        match chroma.prop() {
            Prop::ZERO => RGB::<T>::WHITE,
            Prop::ONE => self.max_chroma_rgb(),
            c_prop => {
                let third = Prop::ONE - c_prop;
                let second = c_prop * self.1 + third;
                self.make_rgb((Prop::ONE, second.into(), third))
            }
        }
    }

    fn rgb_for_sum_and_chroma_prop<T: LightLevel>(
        &self,
        sum: UFDRNumber,
        chroma: Chroma,
    ) -> Option<RGB<T>> {
        debug_assert!(sum.is_valid_sum(), "sum: {:?}", sum);
        match chroma.prop() {
            Prop::ZERO => None,
            c_prop => {
                let ck = self.1 * c_prop;
                let ck_plus_c = ck + c_prop;
                match sum.cmp(&ck_plus_c) {
                    Ordering::Less => None,
                    Ordering::Equal => Some(self.make_rgb((c_prop, ck, Prop::ZERO))),
                    Ordering::Greater => {
                        let three_delta = sum - ck_plus_c;
                        let delta = three_delta / 3;
                        let components = match three_delta % UFDRNumber(3) {
                            // NB: allocation os spare light levels is done so as to preserve
                            // both the requested chroma and sum. Attempts to ensure hue does
                            // not drift have failed to rounding errors involved with division
                            UFDRNumber(1) => (delta + c_prop, delta + ck + Prop(1), delta),
                            UFDRNumber(2) => {
                                (delta + c_prop + Prop(1), delta + ck, delta + Prop(1))
                            }
                            _ => (delta + c_prop, delta + ck, delta),
                        };
                        debug_assert_eq!(components.0 + components.1 + components.2, sum);
                        debug_assert_eq!(components.0 - components.2, c_prop.into());
                        debug_assert!(
                            self.1
                                .abs_diff(
                                    &((components.1 - components.2)
                                        / (components.0 - components.2))
                                        .into()
                                )
                                .0
                                < 0xF000
                        );
                        if components.0 <= UFDRNumber::ONE {
                            Some(self.make_rgb_sum::<T>(components))
                        } else {
                            None
                        }
                    }
                }
            }
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

    fn to_hue(&self) -> Hue {
        *self
    }
}

impl OrderedTriplets for Hue {
    fn ordered_triplet_is_valid(&self, triplet: &[Prop; 3]) -> bool {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.ordered_triplet_is_valid(triplet),
            Self::Secondary(cmy_hue) => cmy_hue.ordered_triplet_is_valid(triplet),
            Self::Sextant(sextant_hue) => sextant_hue.ordered_triplet_is_valid(triplet),
        }
    }

    fn rgb_ordered_triplet_for_sum_and_chroma(
        &self,
        sum: UFDRNumber,
        c_prop: Prop,
    ) -> Option<[Prop; 3]> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.rgb_ordered_triplet_for_sum_and_chroma(sum, c_prop),
            Self::Secondary(cmy_hue) => cmy_hue.rgb_ordered_triplet_for_sum_and_chroma(sum, c_prop),
            Self::Sextant(sextant_hue) => {
                sextant_hue.rgb_ordered_triplet_for_sum_and_chroma(sum, c_prop)
            }
        }
    }
}

impl ColourModificationHelpers for Hue {}

impl HueIfce for Hue {
    fn angle(&self) -> Angle {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.angle(),
            Self::Secondary(cmy_hue) => cmy_hue.angle(),
            Self::Sextant(sextant_hue) => sextant_hue.angle(),
        }
    }

    fn max_chroma_for_sum(&self, sum: UFDRNumber) -> Option<Chroma> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.max_chroma_for_sum(sum),
            Self::Secondary(cmy_hue) => cmy_hue.max_chroma_for_sum(sum),
            Self::Sextant(sextant_hue) => sextant_hue.max_chroma_for_sum(sum),
        }
    }

    fn warmth_for_chroma(&self, chroma: Chroma) -> Warmth {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.warmth_for_chroma(chroma),
            Self::Secondary(cmy_hue) => cmy_hue.warmth_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.warmth_for_chroma(chroma),
        }
    }

    fn max_chroma_rgb<T: LightLevel>(&self) -> RGB<T> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.max_chroma_rgb(),
            Self::Secondary(cmy_hue) => cmy_hue.max_chroma_rgb(),
            Self::Sextant(sextant_hue) => sextant_hue.max_chroma_rgb(),
        }
    }

    fn max_chroma_hcv(&self) -> HCV {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.max_chroma_hcv(),
            Self::Secondary(cmy_hue) => cmy_hue.max_chroma_hcv(),
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

    fn min_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Chroma) -> RGB<T> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.min_sum_rgb_for_chroma(chroma),
            Self::Secondary(cmy_hue) => cmy_hue.min_sum_rgb_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.min_sum_rgb_for_chroma(chroma),
        }
    }

    fn max_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Chroma) -> RGB<T> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.max_sum_rgb_for_chroma(chroma),
            Self::Secondary(cmy_hue) => cmy_hue.max_sum_rgb_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.max_sum_rgb_for_chroma(chroma),
        }
    }

    fn rgb_for_sum_and_chroma_prop<T: LightLevel>(
        &self,
        sum: UFDRNumber,
        chroma: Chroma,
    ) -> Option<RGB<T>> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.rgb_for_sum_and_chroma_prop(sum, chroma),
            Self::Secondary(cmy_hue) => cmy_hue.rgb_for_sum_and_chroma_prop(sum, chroma),
            Self::Sextant(sextant_hue) => sextant_hue.rgb_for_sum_and_chroma_prop(sum, chroma),
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

    pub fn min_sum_for_chroma(&self, chroma: Chroma) -> Option<UFDRNumber> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.min_sum_for_chroma(chroma),
            Self::Secondary(cmy_hue) => cmy_hue.min_sum_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.min_sum_for_chroma(chroma),
        }
    }

    pub fn max_sum_for_chroma(&self, chroma: Chroma) -> Option<UFDRNumber> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.max_sum_for_chroma(chroma),
            Self::Secondary(cmy_hue) => cmy_hue.max_sum_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.max_sum_for_chroma(chroma),
        }
    }

    pub fn sum_range_for_chroma(&self, chroma: Chroma) -> Option<(UFDRNumber, UFDRNumber)> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.sum_range_for_chroma(chroma),
            Self::Secondary(cmy_hue) => cmy_hue.sum_range_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.sum_range_for_chroma(chroma),
        }
    }

    pub fn sum_and_chroma_are_compatible(&self, sum: UFDRNumber, chroma: Chroma) -> bool {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.sum_and_chroma_are_compatible(sum, chroma),
            Self::Secondary(cmy_hue) => cmy_hue.sum_and_chroma_are_compatible(sum, chroma),
            Self::Sextant(sextant_hue) => sextant_hue.sum_and_chroma_are_compatible(sum, chroma),
        }
    }

    pub fn adjusted_chroma_for_sum_compatibility(
        &self,
        c_prop: Prop,
        sum: UFDRNumber,
    ) -> Option<(Chroma, UFDRNumber)> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.adjusted_chroma_for_sum_compatibility(c_prop, sum),
            Self::Secondary(cmy_hue) => cmy_hue.adjusted_chroma_for_sum_compatibility(c_prop, sum),
            Self::Sextant(sextant_hue) => {
                sextant_hue.adjusted_chroma_for_sum_compatibility(c_prop, sum)
            }
        }
    }

    pub fn adjusted_sum_and_chroma_for_chroma_compatibility(
        &self,
        c_prop: Prop,
        sum: UFDRNumber,
    ) -> Option<(UFDRNumber, Chroma)> {
        match self {
            Self::Primary(rgb_hue) => {
                rgb_hue.adjusted_sum_and_chroma_for_chroma_compatibility(c_prop, sum)
            }
            Self::Secondary(cmy_hue) => {
                cmy_hue.adjusted_sum_and_chroma_for_chroma_compatibility(c_prop, sum)
            }
            Self::Sextant(sextant_hue) => {
                sextant_hue.adjusted_sum_and_chroma_for_chroma_compatibility(c_prop, sum)
            }
        }
    }

    pub fn array_for_sum_and_chroma(&self, sum: UFDRNumber, chroma: Chroma) -> Option<[Prop; 3]> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.array_for_sum_and_chroma(sum, chroma),
            Self::Secondary(cmy_hue) => cmy_hue.array_for_sum_and_chroma(sum, chroma),
            Self::Sextant(sextant_hue) => sextant_hue.array_for_sum_and_chroma(sum, chroma),
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
