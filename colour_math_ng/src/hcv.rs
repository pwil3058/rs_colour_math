// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::{
    cmp::Ordering,
    convert::TryFrom,
    ops::{Add, Sub},
};

use crate::{
    fdrn::UFDRNumber,
    hue::{HueIfce, SumOrdering},
    proportion::Warmth,
    rgb::RGB,
    Angle, Chroma, ColourBasics, Hue, HueConstants, LightLevel, ManipulatedColour, Prop,
    RGBConstants,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetScalar {
    Clamp,
    Accommodate,
    Reject,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetHue {
    FavourChroma,
    FavourValue,
}

#[derive(Debug)]
pub enum Outcome {
    Ok,
    Clamped,
    Accommodated,
    NoChange,
    Rejected,
}

#[derive(Debug, Clone, Copy, Eq, Ord, Default, Serialize, Deserialize)]
pub struct HCV {
    hue: Hue,
    pub(crate) chroma: Chroma,
    pub(crate) sum: UFDRNumber,
}

impl PartialEq for HCV {
    fn eq(&self, rhs: &Self) -> bool {
        if self.sum != rhs.sum {
            false
        } else if self.chroma != rhs.chroma {
            false
        } else {
            match self.chroma {
                Chroma::ZERO => true,
                _ => self.hue == rhs.hue,
            }
        }
    }
}

impl PartialOrd for HCV {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        match self.chroma {
            Chroma::ZERO => {
                if rhs.chroma != Chroma::ZERO {
                    Some(Ordering::Less)
                } else {
                    match self.chroma.cmp(&rhs.chroma) {
                        Ordering::Equal => self.sum.partial_cmp(&rhs.sum),
                        ordering => Some(ordering),
                    }
                }
            }
            _ => match self.hue.cmp(&rhs.hue) {
                Ordering::Equal => match self.chroma.cmp(&rhs.chroma) {
                    Ordering::Equal => self.sum.partial_cmp(&rhs.sum),
                    ordering => Some(ordering),
                },
                ordering => Some(ordering),
            },
        }
    }
}

impl HCV {
    pub(crate) fn new(hue: Hue, chroma: Chroma, sum: UFDRNumber) -> Self {
        if let Some(range) = hue.sum_range_for_chroma_prop(chroma.prop()) {
            debug_assert!(range.compare_sum(sum).is_success());
            // TODO: verify chroma shade/tint
            Self { hue, chroma, sum }
        } else {
            Self { hue, chroma, sum }
        }
    }

    pub(crate) fn new_grey_sum(sum: UFDRNumber) -> Self {
        Self {
            hue: Hue::default(),
            chroma: Chroma::ZERO,
            sum,
        }
    }

    pub fn new_grey(value: Prop) -> Self {
        Self::new_grey_sum(value * 3)
    }

    pub fn is_grey(&self) -> bool {
        self.chroma == Chroma::ZERO
    }

    pub(crate) fn is_valid(&self) -> bool {
        match self.chroma.prop() {
            Prop::ZERO => self.sum <= UFDRNumber::THREE,
            prop => {
                if let Some(range) = self.hue.sum_range_for_chroma_prop(prop) {
                    range.compare_sum(self.sum).is_success()
                } else {
                    false
                }
            }
        }
    }

    pub fn sum_range_for_current_chroma(&self) -> (UFDRNumber, UFDRNumber) {
        if let Some(range) = self.hue.sum_range_for_chroma_prop(self.chroma.prop()) {
            (range.min, range.max)
        } else {
            (UFDRNumber::ZERO, UFDRNumber::THREE)
        }
    }

    pub fn max_chroma_for_current_sum(&self) -> Chroma {
        if let Some(max_chroma) = self.hue.max_chroma_for_sum(self.sum) {
            max_chroma
        } else {
            Chroma::ZERO
        }
    }

    pub fn set_hue(&mut self, hue: Hue, policy: SetHue) {
        if let Some(range) = hue.sum_range_for_chroma_prop(self.chroma.prop()) {
            match range.compare_sum(self.sum) {
                SumOrdering::TooSmall(shortfall) => match policy {
                    SetHue::FavourChroma => self.sum = self.sum + shortfall,
                    SetHue::FavourValue => {
                        if let Some(chroma) = hue.max_chroma_for_sum(self.sum) {
                            self.chroma = chroma
                        } else {
                            self.sum = self.sum + shortfall
                        }
                    }
                },
                SumOrdering::TooBig(overs) => match policy {
                    SetHue::FavourChroma => self.sum = self.sum - overs,
                    SetHue::FavourValue => {
                        if let Some(chroma) = hue.max_chroma_for_sum(self.sum) {
                            self.chroma = chroma
                        } else {
                            self.sum = self.sum - overs
                        }
                    }
                },
                _ => self.hue = hue,
            };
            self.hue = hue
        } else {
            self.hue = hue
        }
    }

    pub fn set_chroma_value(&mut self, chroma_value: Prop, policy: SetScalar) -> Outcome {
        match self.chroma.prop().cmp(&chroma_value) {
            Ordering::Equal => Outcome::NoChange,
            Ordering::Greater => {
                self.chroma = Chroma::from((chroma_value, self.hue, self.sum));
                Outcome::Ok
            }
            Ordering::Less => {
                if let Some(range) = self.hue.sum_range_for_chroma_prop(chroma_value) {
                    match range.compare_sum(self.sum) {
                        SumOrdering::TooSmall(shortfall) => match policy {
                            SetScalar::Clamp => {
                                if let Some(adj_c_val) = self.hue.max_chroma_for_sum(self.sum) {
                                    if adj_c_val == self.chroma {
                                        Outcome::NoChange
                                    } else {
                                        self.chroma =
                                            Chroma::from((adj_c_val.prop(), self.hue, self.sum));
                                        Outcome::Clamped
                                    }
                                } else {
                                    Outcome::Rejected
                                }
                            }
                            SetScalar::Accommodate => {
                                self.sum = self.sum + shortfall;
                                self.chroma = Chroma::from((chroma_value, self.hue, self.sum));
                                Outcome::Accommodated
                            }
                            SetScalar::Reject => Outcome::Rejected,
                        },
                        SumOrdering::TooBig(overs) => match policy {
                            SetScalar::Clamp => {
                                if let Some(adj_c_val) = self.hue.max_chroma_for_sum(self.sum) {
                                    if adj_c_val == self.chroma {
                                        Outcome::NoChange
                                    } else {
                                        self.chroma =
                                            Chroma::from((adj_c_val.prop(), self.hue, self.sum));
                                        Outcome::Clamped
                                    }
                                } else {
                                    Outcome::Rejected
                                }
                            }
                            SetScalar::Accommodate => {
                                self.sum = self.sum - overs;
                                self.chroma = Chroma::from((chroma_value, self.hue, self.sum));
                                Outcome::Accommodated
                            }
                            SetScalar::Reject => Outcome::Rejected,
                        },
                        _ => {
                            self.chroma = Chroma::from((chroma_value, self.hue, self.sum));
                            Outcome::Ok
                        }
                    }
                } else {
                    // new value must be zero and needs no checking
                    debug_assert_eq!(chroma_value, Prop::ZERO);
                    self.chroma = Chroma::ZERO;
                    Outcome::Ok
                }
            }
        }
    }

    pub(crate) fn set_sum(&mut self, new_sum: UFDRNumber, policy: SetScalar) -> Outcome {
        debug_assert!(new_sum.is_valid());
        let (min_sum, max_sum) = self.sum_range_for_current_chroma();
        if new_sum < min_sum {
            if policy == SetScalar::Clamp {
                if self.sum == min_sum {
                    Outcome::NoChange
                } else {
                    self.sum = min_sum;
                    Outcome::Clamped
                }
            } else if policy == SetScalar::Accommodate {
                self.sum = new_sum;
                self.chroma = if let Some(max_chroma) = self.hue.max_chroma_for_sum(new_sum) {
                    max_chroma
                } else {
                    Chroma::ZERO
                };
                Outcome::Accommodated
            } else {
                Outcome::Rejected
            }
        } else if new_sum > max_sum {
            if policy == SetScalar::Clamp {
                if self.sum == max_sum {
                    Outcome::NoChange
                } else {
                    self.sum = max_sum;
                    Outcome::Clamped
                }
            } else if policy == SetScalar::Accommodate {
                self.sum = new_sum;
                self.chroma = if let Some(max_chroma) = self.hue.max_chroma_for_sum(new_sum) {
                    max_chroma
                } else {
                    Chroma::ZERO
                };
                Outcome::Accommodated
            } else {
                Outcome::Rejected
            }
        } else {
            self.sum = new_sum;
            Outcome::Ok
        }
    }

    pub fn set_value(&mut self, new_value: Prop, policy: SetScalar) -> Outcome {
        self.set_sum(new_value * 3, policy)
    }
}

impl HueConstants for HCV {
    const RED: Self = Self {
        hue: Hue::RED,
        chroma: Chroma::ONE,
        sum: UFDRNumber::ONE,
    };

    const GREEN: Self = Self {
        hue: Hue::GREEN,
        chroma: Chroma::ONE,
        sum: UFDRNumber::ONE,
    };

    const BLUE: Self = Self {
        hue: Hue::BLUE,
        chroma: Chroma::ONE,
        sum: UFDRNumber::ONE,
    };

    const CYAN: Self = Self {
        hue: Hue::CYAN,
        chroma: Chroma::ONE,
        sum: UFDRNumber::TWO,
    };

    const MAGENTA: Self = Self {
        hue: Hue::MAGENTA,
        chroma: Chroma::ONE,
        sum: UFDRNumber::TWO,
    };

    const YELLOW: Self = Self {
        hue: Hue::YELLOW,
        chroma: Chroma::ONE,
        sum: UFDRNumber::TWO,
    };

    const BLUE_CYAN: Self = Self {
        hue: Hue::BLUE_CYAN,
        chroma: Chroma::ONE,
        sum: UFDRNumber::ONE_PT_5,
    };

    const BLUE_MAGENTA: Self = Self {
        hue: Hue::BLUE_MAGENTA,
        chroma: Chroma::ONE,
        sum: UFDRNumber::ONE_PT_5,
    };

    const RED_MAGENTA: Self = Self {
        hue: Hue::RED_MAGENTA,
        chroma: Chroma::ONE,
        sum: UFDRNumber::ONE_PT_5,
    };

    const RED_YELLOW: Self = Self {
        hue: Hue::RED_YELLOW,
        chroma: Chroma::ONE,
        sum: UFDRNumber::ONE_PT_5,
    };

    const GREEN_YELLOW: Self = Self {
        hue: Hue::GREEN_YELLOW,
        chroma: Chroma::ONE,
        sum: UFDRNumber::ONE_PT_5,
    };

    const GREEN_CYAN: Self = Self {
        hue: Hue::GREEN_CYAN,
        chroma: Chroma::ONE,
        sum: UFDRNumber::ONE_PT_5,
    };
}

impl RGBConstants for HCV {
    const WHITE: Self = Self {
        hue: Hue::RED,
        chroma: Chroma::ZERO,
        sum: UFDRNumber::THREE,
    };

    const BLACK: Self = Self {
        hue: Hue::RED,
        chroma: Chroma::ZERO,
        sum: UFDRNumber::ZERO,
    };
}

impl<L: LightLevel> From<&RGB<L>> for HCV {
    fn from(rgb: &RGB<L>) -> Self {
        if let Ok(hue) = Hue::try_from(rgb) {
            Self {
                hue: hue,
                chroma: rgb.chroma(),
                sum: rgb.sum(),
            }
        } else {
            Self {
                hue: Hue::default(),
                chroma: Chroma::ZERO,
                sum: rgb.sum(),
            }
        }
    }
}

impl<L: LightLevel> From<RGB<L>> for HCV {
    fn from(rgb: RGB<L>) -> Self {
        HCV::from(&rgb)
    }
}

impl<L: LightLevel> From<HCV> for RGB<L> {
    fn from(hcv: HCV) -> Self {
        hcv.rgb::<L>()
    }
}

impl<L: LightLevel> From<&HCV> for RGB<L> {
    fn from(hcv: &HCV) -> Self {
        hcv.rgb::<L>()
    }
}

impl ColourBasics for HCV {
    fn hue(&self) -> Option<Hue> {
        match self.chroma {
            Chroma::ZERO => None,
            _ => Some(self.hue),
        }
    }

    fn hue_angle(&self) -> Option<Angle> {
        match self.chroma {
            Chroma::ZERO => None,
            _ => Some(self.hue.angle()),
        }
    }

    fn is_grey(&self) -> bool {
        self.chroma == Chroma::ZERO
    }

    fn chroma(&self) -> Chroma {
        self.chroma
    }

    fn value(&self) -> Prop {
        (self.sum / 3).into()
    }

    fn warmth(&self) -> Warmth {
        match self.chroma {
            Chroma::ZERO => Warmth::calculate_monochrome_fm_sum(self.sum),
            _ => self.hue.warmth_for_chroma(self.chroma),
        }
    }

    fn rgb<L: LightLevel>(&self) -> RGB<L> {
        debug_assert!(self.is_valid());
        match self.chroma {
            Chroma::ZERO => RGB::new_grey(self.value()),
            chroma => self
                .hue
                .rgb_for_sum_and_chroma::<L>(self.sum, chroma)
                .expect("Assume that we're valid and there must be an equivalent RGB"),
        }
    }

    fn hcv(&self) -> HCV {
        *self
    }
}

impl ManipulatedColour for HCV {
    fn lightened(&self, prop: Prop) -> Self {
        let rgb = RGB::<u64>::from(self).lightened(prop);
        HCV::from(rgb)
    }

    fn darkened(&self, prop: Prop) -> Self {
        let rgb = RGB::<u64>::from(self).darkened(prop);
        HCV::from(rgb)
    }
    fn saturated(&self, prop: Prop) -> Self {
        let new_chroma = match self.chroma {
            Chroma::Shade(c_prop) => Chroma::Shade((c_prop - c_prop * prop + prop).into()),
            Chroma::Tint(c_prop) => Chroma::Tint((c_prop - c_prop * prop + prop).into()),
            Chroma::Neither(c_prop) => match prop {
                Prop::ONE => Chroma::ONE,
                Prop::ZERO => Chroma::from((prop, self.hue, self.sum)),
                prop => Chroma::Neither((c_prop - c_prop * prop + prop).into()),
            },
        };
        let new_sum = if let Some((min_sum, max_sum)) = self.hue.sum_range_for_chroma(new_chroma) {
            if self.sum < min_sum {
                min_sum
            } else if self.sum > max_sum {
                max_sum
            } else {
                self.sum
            }
        } else {
            self.sum
        };
        Self {
            hue: self.hue,
            chroma: new_chroma,
            sum: new_sum,
        }
    }

    fn greyed(&self, prop: Prop) -> Self {
        let new_chroma = match self.chroma {
            Chroma::Shade(c_prop) => Chroma::Shade((c_prop - c_prop * prop).into()),
            Chroma::Tint(c_prop) => Chroma::Tint((c_prop - c_prop * prop).into()),
            Chroma::Neither(c_prop) => match prop {
                Prop::ZERO => Chroma::ZERO,
                Prop::ONE => Chroma::from((Prop::ONE - prop, self.hue, self.sum)),
                prop => Chroma::Neither((c_prop - c_prop * prop).into()),
            },
        };
        let new_sum = if let Some((min_sum, max_sum)) = self.hue.sum_range_for_chroma(new_chroma) {
            if self.sum < min_sum {
                min_sum
            } else if self.sum > max_sum {
                max_sum
            } else {
                self.sum
            }
        } else {
            self.sum
        };
        Self {
            hue: self.hue,
            chroma: new_chroma,
            sum: new_sum,
        }
    }

    fn rotated(&self, angle: Angle) -> Self {
        *self + angle
    }
}

impl Add<Angle> for HCV {
    type Output = Self;

    fn add(self, angle: Angle) -> Self {
        let new_hue = self.hue + angle;
        if let Some((min_sum, max_sum)) = new_hue.sum_range_for_chroma(self.chroma()) {
            let new_sum = if self.sum < min_sum {
                min_sum
            } else if self.sum > max_sum {
                max_sum
            } else {
                self.sum
            };
            Self {
                hue: new_hue,
                chroma: self.chroma,
                sum: new_sum,
            }
        } else {
            // TODO: Should greys go through rotation unchanged
            self
        }
    }
}

impl Sub<Angle> for HCV {
    type Output = Self;

    fn sub(self, angle: Angle) -> Self {
        let new_hue = self.hue - angle;
        if let Some((min_sum, max_sum)) = new_hue.sum_range_for_chroma(self.chroma()) {
            let new_sum = if self.sum < min_sum {
                min_sum
            } else if self.sum > max_sum {
                max_sum
            } else {
                self.sum
            };
            Self {
                hue: new_hue,
                chroma: self.chroma,
                sum: new_sum,
            }
        } else {
            // TODO: Should greys go through rotation unchanged
            self
        }
    }
}

#[cfg(test)]
mod hcv_tests;
