// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::convert::TryFrom;

use crate::hue::SumOrdering;
use crate::{
    hue::{CMYHue, HueIfce, RGBHue},
    rgb::RGB,
    Chroma, Hue, HueConstants, LightLevel, Prop, RGBConstants, Sum,
};
use std::cmp::Ordering;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct HCV {
    hue: Hue,
    pub(crate) chroma: Chroma,
    pub(crate) sum: Sum,
}

impl HCV {
    // pub(crate) fn new_grey(sum: Sum) -> Self {
    //     Self {
    //         hue: Hue::default(),
    //         chroma: Chroma::ZERO,
    //         sum,
    //     }
    // }

    pub fn is_grey(&self) -> bool {
        self.chroma == Chroma::ZERO
    }

    // pub(crate) fn is_valid(&self) -> bool {
    //     match self.chroma.prop() {
    //         Prop::ZERO => self.sum <= Sum::THREE,
    //         prop => {
    //             let range = self
    //                 .hue
    //                 .sum_range_for_chroma_prop(prop)
    //                 .expect("chroma != 0");
    //             range.compare_sum(self.sum).is_success()
    //         }
    //     }
    // }

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
}

impl HueConstants for HCV {
    const RED: Self = Self {
        hue: Hue::Primary(RGBHue::Red),
        chroma: Chroma::ONE,
        sum: Sum::ONE,
    };

    const GREEN: Self = Self {
        hue: Hue::Primary(RGBHue::Green),
        chroma: Chroma::ONE,
        sum: Sum::ONE,
    };

    const BLUE: Self = Self {
        hue: Hue::Primary(RGBHue::Blue),
        chroma: Chroma::ONE,
        sum: Sum::ONE,
    };

    const CYAN: Self = Self {
        hue: Hue::Secondary(CMYHue::Cyan),
        chroma: Chroma::ONE,
        sum: Sum::TWO,
    };

    const MAGENTA: Self = Self {
        hue: Hue::Secondary(CMYHue::Magenta),
        chroma: Chroma::ONE,
        sum: Sum::TWO,
    };

    const YELLOW: Self = Self {
        hue: Hue::Secondary(CMYHue::Yellow),
        chroma: Chroma::ONE,
        sum: Sum::TWO,
    };
}

impl RGBConstants for HCV {
    const WHITE: Self = Self {
        hue: Hue::RED,
        chroma: Chroma::ZERO,
        sum: Sum::THREE,
    };

    const BLACK: Self = Self {
        hue: Hue::RED,
        chroma: Chroma::ZERO,
        sum: Sum::ZERO,
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

impl<L: LightLevel> From<&HCV> for RGB<L> {
    fn from(hcv: &HCV) -> Self {
        hcv.rgb::<L>()
    }
}

pub trait ColourIfce {
    fn hue(&self) -> Option<Hue>;
    fn chroma(&self) -> Chroma;
    fn value(&self) -> Prop;
    fn warmth(&self) -> Prop;

    fn rgb<L: LightLevel>(&self) -> RGB<L>;
}

impl ColourIfce for HCV {
    fn hue(&self) -> Option<Hue> {
        match self.chroma {
            Chroma::ZERO => None,
            _ => Some(self.hue),
        }
    }

    fn chroma(&self) -> Chroma {
        self.chroma
    }

    fn value(&self) -> Prop {
        self.sum / 3
    }

    fn warmth(&self) -> Prop {
        match self.chroma {
            Chroma::ONE => (Sum::THREE - self.sum) / 6,
            _ => self.hue.warmth_for_chroma(self.chroma),
        }
    }

    fn rgb<L: LightLevel>(&self) -> RGB<L> {
        match self.chroma {
            Chroma::ZERO => RGB::new_grey(self.value()),
            chroma => self
                .hue
                .rgb_for_sum_and_chroma::<L>(self.sum, chroma)
                .expect("Assume that we're valid and there must be an equivalent RGB"),
        }
    }
}

#[cfg(test)]
mod hcv_tests;
