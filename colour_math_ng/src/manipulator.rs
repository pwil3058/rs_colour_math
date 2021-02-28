// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::cmp::Ordering;

use crate::{
    fdrn::UFDRNumber,
    hcv::{Outcome, SetHue, SetScalar},
    hue::{HueIfce, SumOrdering},
    Angle, Chroma, ColourBasics, Hue, HueConstants, LightLevel, Prop, HCV, RGB,
};

#[derive(Debug)]
pub struct ColourManipulator {
    hcv: HCV,
    clamped: bool,
    rotation_policy: SetHue,
    saved_hue: Hue,
}

impl ColourManipulator {
    pub fn rgb<L: LightLevel>(&self) -> RGB<L> {
        self.hcv.rgb()
    }

    pub fn hcv(&self) -> HCV {
        self.hcv
    }

    pub fn set_colour(&mut self, colour: &impl ColourBasics) {
        self.hcv = colour.hcv()
    }

    pub fn clamped(&self) -> bool {
        self.clamped
    }

    pub fn set_clamped(&mut self, clamped: bool) {
        self.clamped = clamped
    }

    pub fn rotation_policy(&self) -> SetHue {
        self.rotation_policy
    }

    pub fn set_rotation_policy(&mut self, rotation_policy: SetHue) {
        self.rotation_policy = rotation_policy
    }

    pub fn set_chroma_value(&mut self, chroma_value: Prop, policy: SetScalar) -> Outcome {
        let outcome = match self.hcv.chroma.prop().cmp(&chroma_value) {
            Ordering::Equal => Outcome::NoChange,
            Ordering::Greater => {
                println!("greater: {:?}", chroma_value);
                self.hcv.chroma = Chroma::from((chroma_value, self.hcv.hue, self.hcv.sum));
                Outcome::Ok
            }
            Ordering::Less => {
                println!("less: {:?}", chroma_value);
                if let Some(range) = self.hcv.hue.sum_range_for_chroma_prop(chroma_value) {
                    match range.compare_sum(self.hcv.sum) {
                        SumOrdering::TooSmall(shortfall) => match policy {
                            SetScalar::Clamp => {
                                if let Some(adj_c_val) =
                                    self.hcv.hue.max_chroma_for_sum(self.hcv.sum)
                                {
                                    if adj_c_val == self.hcv.chroma {
                                        Outcome::NoChange
                                    } else {
                                        self.hcv.chroma = Chroma::from((
                                            adj_c_val.prop(),
                                            self.hcv.hue,
                                            self.hcv.sum,
                                        ));
                                        Outcome::Clamped
                                    }
                                } else {
                                    Outcome::Rejected
                                }
                            }
                            SetScalar::Accommodate => {
                                self.hcv.sum = self.hcv.sum + shortfall;
                                self.hcv.chroma =
                                    Chroma::from((chroma_value, self.hcv.hue, self.hcv.sum));
                                Outcome::Accommodated
                            }
                            SetScalar::Reject => Outcome::Rejected,
                        },
                        SumOrdering::TooBig(overs) => match policy {
                            SetScalar::Clamp => {
                                if let Some(adj_c_val) =
                                    self.hcv.hue.max_chroma_for_sum(self.hcv.sum)
                                {
                                    if adj_c_val == self.hcv.chroma {
                                        Outcome::NoChange
                                    } else {
                                        self.hcv.chroma = Chroma::from((
                                            adj_c_val.prop(),
                                            self.hcv.hue,
                                            self.hcv.sum,
                                        ));
                                        Outcome::Clamped
                                    }
                                } else {
                                    Outcome::Rejected
                                }
                            }
                            SetScalar::Accommodate => {
                                self.hcv.sum = self.hcv.sum - overs;
                                self.hcv.chroma =
                                    Chroma::from((chroma_value, self.hcv.hue, self.hcv.sum));
                                Outcome::Accommodated
                            }
                            SetScalar::Reject => Outcome::Rejected,
                        },
                        _ => {
                            self.hcv.chroma =
                                Chroma::from((chroma_value, self.hcv.hue, self.hcv.sum));
                            Outcome::Ok
                        }
                    }
                } else {
                    // new value must be zero and needs no checking
                    debug_assert_eq!(chroma_value, Prop::ZERO);
                    self.hcv.chroma = Chroma::ZERO;
                    Outcome::Ok
                }
            }
        };
        if self.hcv.chroma == Chroma::ZERO {
            self.hcv.sum = self.hcv.sum / 3 * 3;
        }
        debug_assert!(self.hcv.is_valid());
        outcome
    }

    pub fn decr_chroma(&mut self, delta: Prop) -> bool {
        match self.hcv.chroma.prop() {
            Prop::ZERO => false,
            c_prop => {
                let new_chroma_value = if c_prop > delta {
                    c_prop - delta
                } else {
                    self.saved_hue = self.hcv.hue().expect("chroma != 0");
                    Prop::ZERO
                };
                let policy = if self.clamped {
                    SetScalar::Clamp
                } else {
                    SetScalar::Accommodate
                };
                match self.set_chroma_value(new_chroma_value, policy) {
                    Outcome::Ok | Outcome::Clamped | Outcome::Accommodated => true,
                    _ => false,
                }
            }
        }
    }

    pub fn incr_chroma(&mut self, delta: Prop) -> bool {
        match self.hcv.chroma.prop() {
            Prop::ONE => false,
            c_prop => {
                let new_chroma_value = if c_prop < Prop::ONE - delta {
                    (c_prop + delta).into()
                } else {
                    Prop::ONE
                };
                let policy = if self.clamped {
                    SetScalar::Clamp
                } else {
                    SetScalar::Accommodate
                };
                match self.set_chroma_value(new_chroma_value, policy) {
                    Outcome::Ok | Outcome::Clamped | Outcome::Accommodated => true,
                    _ => false,
                }
            }
        }
    }

    pub fn decr_value(&mut self, delta: Prop) -> bool {
        if self.hcv.sum == UFDRNumber::ZERO {
            false
        } else {
            let new_sum = if delta * 3 < self.hcv.sum {
                self.hcv.sum - delta * 3
            } else {
                UFDRNumber::ZERO
            };
            let policy = if self.clamped {
                SetScalar::Clamp
            } else {
                SetScalar::Accommodate
            };
            match self.hcv.set_sum(new_sum, policy) {
                Outcome::Ok | Outcome::Clamped | Outcome::Accommodated => true,
                _ => false,
            }
        }
    }

    pub fn incr_value(&mut self, delta: Prop) -> bool {
        if self.hcv.sum == UFDRNumber::THREE {
            false
        } else {
            let new_sum = if delta * 3 < UFDRNumber::THREE - self.hcv.sum {
                self.hcv.sum + delta * 3
            } else {
                UFDRNumber::THREE
            };
            let policy = if self.clamped {
                SetScalar::Clamp
            } else {
                SetScalar::Accommodate
            };
            match self.hcv.set_sum(new_sum, policy) {
                Outcome::Ok | Outcome::Clamped | Outcome::Accommodated => true,
                _ => false,
            }
        }
    }

    pub fn set_hue(&mut self, hue: Hue, policy: SetHue) {
        debug_assert!(self.hcv.is_valid());
        if let Some((min_sum, max_sum)) = hue.sum_range_for_chroma(self.hcv.chroma) {
            match policy {
                SetHue::FavourChroma => {
                    if self.hcv.sum < min_sum {
                        self.hcv.sum = min_sum
                    } else if self.hcv.sum > max_sum {
                        self.hcv.sum = max_sum
                    }
                }
                SetHue::FavourValue => {
                    if self.hcv.sum < min_sum || self.hcv.sum > max_sum && min_sum != max_sum {
                        println!(
                            "FV: {:#X} : {:#X} : {:#X}",
                            min_sum.0, self.hcv.sum.0, max_sum.0
                        );
                        if let Some(chroma) = hue.max_chroma_for_sum(self.hcv.sum) {
                            self.hcv.chroma = chroma
                        } else {
                            debug_assert_eq!(self.hcv.chroma, Chroma::ZERO);
                        }
                    }
                }
            }
            self.hcv.hue = hue;
        };
        debug_assert!(self.hcv.is_valid());
    }

    pub fn rotate(&mut self, angle: Angle) -> bool {
        match self.hcv.hue_angle() {
            None => false,
            Some(cur_angle) => {
                let new_angle = cur_angle + angle;
                if new_angle == cur_angle {
                    false
                } else {
                    self.set_hue(Hue::from(new_angle), self.rotation_policy);
                    self.saved_hue = self.hcv.hue().expect("chroma is not zero");
                    true
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct ColourManipulatorBuilder {
    init_hcv: Option<HCV>,
    clamped: bool,
    rotation_policy: SetHue,
}

impl ColourManipulatorBuilder {
    pub fn new() -> Self {
        Self {
            init_hcv: None,
            clamped: false,
            rotation_policy: SetHue::FavourChroma,
        }
    }

    pub fn init_rgb<L: LightLevel>(&mut self, rgb: &RGB<L>) -> &mut Self {
        self.init_hcv = Some(rgb.into());
        self
    }

    pub fn init_hcv(&mut self, hcv: &HCV) -> &mut Self {
        self.init_hcv = Some(*hcv);
        self
    }

    pub fn clamped(&mut self, clamped: bool) -> &mut Self {
        self.clamped = clamped;
        self
    }

    pub fn rotation_policy(&mut self, rotation_policy: SetHue) -> &mut Self {
        self.rotation_policy = rotation_policy;
        self
    }

    pub fn build(&self) -> ColourManipulator {
        let hcv = if let Some(init_hcv) = self.init_hcv {
            init_hcv
        } else {
            HCV::default()
        };
        let saved_hue = if let Some(hue) = hcv.hue() {
            hue
        } else {
            Hue::RED
        };
        ColourManipulator {
            hcv,
            saved_hue,
            clamped: self.clamped,
            rotation_policy: self.rotation_policy,
        }
    }
}

//#[cfg(test)]
//mod manipulator_tests;
