// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use crate::{
    fdrn::UFDRNumber, Angle, Chroma, ColourBasics, Hue, HueConstants, LightLevel, Prop, HCV, RGB,
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

    pub fn set_chroma(&mut self, new_chroma: Chroma, policy: SetScalar) -> Outcome {
        let cur_chroma = self.hcv.chroma;
        if let Some(hue) = self.hcv.hue {
            self.saved_hue = hue; // Just in case we end up grey (which is possible)
            if hue.sum_and_chroma_are_compatible(self.hcv.sum, new_chroma) {
                self.hcv = HCV::new(Some((hue, new_chroma)), self.hcv.sum);
                if cur_chroma == self.hcv.chroma {
                    Outcome::NoChange
                } else {
                    Outcome::Ok
                }
            } else {
                match policy {
                    SetScalar::Clamp => {
                        if let Some(chroma) = hue
                            .adjusted_chroma_for_sum_compatibility(new_chroma.prop(), self.hcv.sum)
                        {
                            self.hcv = HCV::new(Some((hue, chroma)), self.hcv.sum);
                        } else {
                            self.hcv = HCV::new_grey((self.hcv.sum / 3).into());
                        }
                        if cur_chroma == self.hcv.chroma {
                            Outcome::NoChange
                        } else {
                            Outcome::Clamped
                        }
                    }
                    SetScalar::Accommodate => {
                        if let Some((sum, chroma)) = hue
                            .adjusted_sum_and_chroma_for_chroma_compatibility(
                                new_chroma.prop(),
                                self.hcv.sum,
                            )
                        {
                            self.hcv = HCV::new(Some((hue, chroma)), sum);
                        } else {
                            self.hcv = HCV::new_grey((self.hcv.sum / 3).into())
                        }
                        if cur_chroma == self.hcv.chroma {
                            Outcome::NoChange
                        } else {
                            Outcome::Accommodated
                        }
                    }
                    SetScalar::Reject => Outcome::Rejected,
                }
            }
        } else if new_chroma == Chroma::ZERO {
            Outcome::NoChange
        } else {
            if self
                .saved_hue
                .sum_and_chroma_are_compatible(self.hcv.sum, new_chroma)
            {
                self.hcv = HCV::new(Some((self.saved_hue, new_chroma)), self.hcv.sum);
                if cur_chroma == self.hcv.chroma {
                    Outcome::NoChange
                } else {
                    Outcome::Ok
                }
            } else {
                match policy {
                    SetScalar::Clamp => {
                        if let Some(chroma) = self
                            .saved_hue
                            .adjusted_chroma_for_sum_compatibility(new_chroma.prop(), self.hcv.sum)
                        {
                            self.hcv = HCV::new(Some((self.saved_hue, chroma)), self.hcv.sum);
                        } else {
                            self.hcv = HCV::new_grey((self.hcv.sum / 3).into());
                        }
                        if cur_chroma == self.hcv.chroma {
                            Outcome::NoChange
                        } else {
                            Outcome::Clamped
                        }
                    }
                    SetScalar::Accommodate => {
                        if let Some((sum, chroma)) = self
                            .saved_hue
                            .adjusted_sum_and_chroma_for_chroma_compatibility(
                                new_chroma.prop(),
                                self.hcv.sum,
                            )
                        {
                            self.hcv = HCV::new(Some((self.saved_hue, chroma)), sum);
                        } else {
                            self.hcv = HCV::new_grey((self.hcv.sum / 3).into())
                        }
                        if cur_chroma == self.hcv.chroma {
                            Outcome::NoChange
                        } else {
                            Outcome::Accommodated
                        }
                    }
                    SetScalar::Reject => Outcome::Rejected,
                }
            }
        }
    }

    pub fn decr_chroma(&mut self, delta: Prop) -> bool {
        match self.hcv.chroma.prop() {
            Prop::ZERO => false,
            c_prop => {
                let new_chroma = if c_prop > delta {
                    match self.hcv.chroma {
                        Chroma::Shade(c_prop) => Chroma::Shade(c_prop - delta),
                        Chroma::Tint(c_prop) => Chroma::Tint(c_prop - delta),
                        Chroma::Neither(c_prop) => Chroma::Neither(c_prop - delta),
                    }
                } else {
                    Chroma::ZERO
                };
                let policy = if self.clamped {
                    SetScalar::Clamp
                } else {
                    SetScalar::Accommodate
                };
                match self.set_chroma(new_chroma, policy) {
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
                    match self.hcv.chroma {
                        Chroma::Shade(c_prop) => Chroma::Shade((c_prop + delta).into()),
                        Chroma::Tint(c_prop) => Chroma::Tint((c_prop + delta).into()),
                        Chroma::Neither(c_prop) => Chroma::Neither((c_prop + delta).into()),
                    }
                } else {
                    Chroma::ONE
                };
                let policy = if self.clamped {
                    SetScalar::Clamp
                } else {
                    SetScalar::Accommodate
                };
                match self.set_chroma(new_chroma_value, policy) {
                    Outcome::Ok | Outcome::Clamped | Outcome::Accommodated => true,
                    _ => false,
                }
            }
        }
    }

    pub fn set_sum(&mut self, new_sum: UFDRNumber, policy: SetScalar) -> Outcome {
        debug_assert!(new_sum.is_valid_sum());
        let cur_sum = self.hcv.sum;
        if let Some(hue) = self.hcv.hue {
            self.saved_hue = hue;
            if hue.sum_and_chroma_are_compatible(new_sum, self.hcv.chroma) {
                self.hcv = HCV::new(Some((hue, self.hcv.chroma)), new_sum);
                if cur_sum == self.hcv.sum {
                    Outcome::NoChange
                } else {
                    Outcome::Ok
                }
            } else {
                match policy {
                    SetScalar::Clamp => {
                        if let Some((sum, chroma)) = hue
                            .adjusted_sum_and_chroma_for_chroma_compatibility(
                                self.hcv.chroma.prop(),
                                new_sum,
                            )
                        {
                            self.hcv = HCV::new(Some((hue, chroma)), sum);
                        } else {
                            self.hcv = HCV::new_grey((new_sum / 3).into());
                        };
                        if cur_sum == self.hcv.sum {
                            Outcome::NoChange
                        } else {
                            Outcome::Clamped
                        }
                    }
                    SetScalar::Accommodate => {
                        if let Some(chroma) = hue
                            .adjusted_chroma_for_sum_compatibility(self.hcv.chroma.prop(), new_sum)
                        {
                            self.hcv = HCV::new(Some((hue, chroma)), new_sum);
                        } else {
                            self.hcv = HCV::new_grey((new_sum / 3).into());
                        }
                        if cur_sum == self.hcv.sum {
                            Outcome::NoChange
                        } else {
                            Outcome::Accommodated
                        }
                    }
                    SetScalar::Reject => Outcome::Rejected,
                }
            }
        } else {
            self.hcv = HCV::new_grey((new_sum / 3).into());
            if cur_sum == self.hcv.sum {
                Outcome::NoChange
            } else {
                Outcome::Ok
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
            match self.set_sum(new_sum, policy) {
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
            match self.set_sum(new_sum, policy) {
                Outcome::Ok | Outcome::Clamped | Outcome::Accommodated => true,
                _ => false,
            }
        }
    }

    pub fn set_hue(&mut self, new_hue: Hue, policy: SetHue) {
        // TODO: change argument to Option<Hue>
        debug_assert!(self.hcv.is_valid());
        if let Some(hue) = self.hcv.hue {
            self.saved_hue = hue;
            if let Some((min_sum, max_sum)) = new_hue.sum_range_for_chroma(self.hcv.chroma) {
                match policy {
                    SetHue::FavourChroma => {
                        if self.hcv.sum < min_sum {
                            self.hcv.sum = min_sum
                        } else if self.hcv.sum > max_sum {
                            self.hcv.sum = max_sum
                        }
                    }
                    SetHue::FavourValue => {
                        if let Some(chroma) = new_hue.adjusted_chroma_for_sum_compatibility(
                            self.hcv.chroma.prop(),
                            self.hcv.sum,
                        ) {
                            self.hcv = HCV::new(Some((new_hue, chroma)), self.hcv.sum);
                        } else {
                            self.hcv = HCV::new(None, self.hcv.sum)
                        }
                    }
                }
                self.hcv.hue = Some(new_hue);
            };
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

// #[cfg(test)]
// mod manipulator_tests;
