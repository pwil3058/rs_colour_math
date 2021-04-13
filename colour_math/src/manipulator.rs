// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use crate::{
    attributes::Chroma,
    fdrn::{IntoProp, Prop, UFDRNumber},
    hcv::HCV,
    hue::Hue,
    hue::{angle::Angle, ColourModificationHelpers, HueBasics, HueIfce, SumChromaCompatibility},
    rgb::RGB,
    ColourBasics, HueConstants, LightLevel,
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
        debug_assert!(self.hcv.is_valid());
        let cur_chroma = self.hcv.chroma;
        if let Some(hue) = self.hcv.hue {
            self.saved_hue = hue; // Just in case we end up grey (which is possible)
            if hue.sum_and_chroma_are_compatible(self.hcv.sum, new_chroma) {
                self.hcv = match HCV::try_new(Some((hue, new_chroma.into_prop())), self.hcv.sum) {
                    Ok(hcv) => hcv,
                    Err(hcv) => hcv,
                };
                if cur_chroma == self.hcv.chroma {
                    Outcome::NoChange
                } else {
                    Outcome::Ok
                }
            } else {
                match policy {
                    SetScalar::Clamp => {
                        if let Some(max_chroma) = hue.max_chroma_for_sum(self.hcv.sum) {
                            let clamped_new_chroma =
                                if new_chroma.into_prop() < max_chroma.into_prop() {
                                    new_chroma
                                } else if self.hcv.chroma.into_prop() < max_chroma.into_prop() {
                                    max_chroma
                                } else {
                                    return Outcome::NoChange;
                                };
                            self.hcv = if let Some((chroma, sum)) =
                                hue.adjusted_favouring_sum(self.hcv.sum, clamped_new_chroma)
                            {
                                if chroma == Chroma::ZERO {
                                    HCV::new_grey((sum / 3).into())
                                } else {
                                    match HCV::try_new(Some((hue, chroma.into_prop())), sum) {
                                        Ok(hcv) => hcv,
                                        Err(hcv) => hcv,
                                    }
                                }
                            } else {
                                HCV::new_grey((self.hcv.sum / 3).into())
                            };
                            if cur_chroma == self.hcv.chroma {
                                Outcome::NoChange
                            } else {
                                Outcome::Clamped
                            }
                        } else {
                            Outcome::NoChange
                        }
                    }
                    SetScalar::Accommodate => {
                        if let Some((chroma, sum)) =
                            hue.adjusted_favouring_chroma(self.hcv.sum, new_chroma)
                        {
                            if chroma == Chroma::ZERO {
                                self.hcv = HCV::new_grey((self.hcv.sum / 3).into());
                            } else {
                                self.hcv = match HCV::try_new(Some((hue, chroma.into_prop())), sum)
                                {
                                    Ok(hcv) => hcv,
                                    Err(hcv) => hcv,
                                };
                            }
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
                self.hcv = match HCV::try_new(
                    Some((self.saved_hue, new_chroma.into_prop())),
                    self.hcv.sum,
                ) {
                    Ok(hcv) => hcv,
                    Err(hcv) => hcv,
                };
                if cur_chroma == self.hcv.chroma {
                    Outcome::NoChange
                } else {
                    Outcome::Ok
                }
            } else {
                match policy {
                    SetScalar::Clamp => {
                        if let Some(max_chroma) = self.saved_hue.max_chroma_for_sum(self.hcv.sum) {
                            let clamped_new_chroma =
                                if new_chroma.into_prop() < max_chroma.into_prop() {
                                    new_chroma
                                } else if self.hcv.chroma.into_prop() < max_chroma.into_prop() {
                                    max_chroma
                                } else {
                                    return Outcome::NoChange;
                                };
                            self.hcv = if let Some((chroma, sum)) = self
                                .saved_hue
                                .adjusted_favouring_sum(self.hcv.sum, clamped_new_chroma)
                            {
                                match HCV::try_new(Some((self.saved_hue, chroma.into_prop())), sum)
                                {
                                    Ok(hcv) => hcv,
                                    Err(hcv) => hcv,
                                }
                            } else {
                                self.hcv
                            };
                            if cur_chroma == self.hcv.chroma {
                                Outcome::NoChange
                            } else {
                                Outcome::Clamped
                            }
                        } else {
                            Outcome::NoChange
                        }
                    }
                    SetScalar::Accommodate => {
                        if let Some((chroma, sum)) = self
                            .saved_hue
                            .adjusted_favouring_chroma(self.hcv.sum, new_chroma)
                        {
                            self.hcv =
                                match HCV::try_new(Some((self.saved_hue, chroma.into_prop())), sum)
                                {
                                    Ok(hcv) => hcv,
                                    Err(hcv) => hcv,
                                };
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
        debug_assert!(self.hcv.is_valid());
        match self.hcv.chroma.into_prop() {
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
                    Outcome::Ok | Outcome::Clamped | Outcome::Accommodated => {
                        debug_assert!(self.hcv.is_valid());
                        true
                    }
                    _ => false,
                }
            }
        }
    }

    pub fn incr_chroma(&mut self, delta: Prop) -> bool {
        debug_assert!(self.hcv.is_valid());
        let policy = if self.clamped {
            SetScalar::Clamp
        } else {
            SetScalar::Accommodate
        };
        match self.hcv.chroma.into_prop() {
            Prop::ONE => false,
            Prop::ZERO => {
                let new_chroma = match self.hcv.chroma {
                    Chroma::Shade(_) => Chroma::Shade(delta),
                    Chroma::Tint(_) => Chroma::Tint(delta),
                    Chroma::Neither(_) => Chroma::Neither(delta),
                };
                match self.set_chroma(new_chroma, policy) {
                    Outcome::Ok | Outcome::Clamped | Outcome::Accommodated => {
                        debug_assert!(self.hcv.is_valid());
                        true
                    }
                    _ => false,
                }
            }
            c_prop => {
                let new_chroma = if c_prop < Prop::ONE - delta {
                    match self.hcv.chroma {
                        Chroma::Shade(c_prop) => Chroma::Shade((c_prop + delta).into()),
                        Chroma::Tint(c_prop) => Chroma::Tint((c_prop + delta).into()),
                        Chroma::Neither(c_prop) => Chroma::Neither((c_prop + delta).into()),
                    }
                } else {
                    Chroma::ONE
                };
                match self.set_chroma(new_chroma, policy) {
                    Outcome::Ok | Outcome::Clamped | Outcome::Accommodated => {
                        debug_assert!(self.hcv.is_valid());
                        true
                    }
                    _ => false,
                }
            }
        }
    }

    pub fn set_sum(&mut self, new_sum: UFDRNumber, policy: SetScalar) -> Outcome {
        debug_assert!(self.hcv.is_valid());
        debug_assert!(new_sum.is_valid_sum());
        let cur_sum = self.hcv.sum;
        if let Some(hue) = self.hcv.hue {
            self.saved_hue = hue;
            if hue.sum_and_chroma_are_compatible(new_sum, self.hcv.chroma) {
                self.hcv = match HCV::try_new(Some((hue, self.hcv.chroma.into_prop())), new_sum) {
                    Ok(hcv) => hcv,
                    Err(hcv) => hcv,
                };
                if cur_sum == self.hcv.sum {
                    Outcome::NoChange
                } else {
                    Outcome::Ok
                }
            } else {
                match policy {
                    SetScalar::Clamp => {
                        if let Some((chroma, sum)) =
                            hue.adjusted_favouring_chroma(new_sum, self.hcv.chroma)
                        {
                            self.hcv = match HCV::try_new(Some((hue, chroma.into_prop())), sum) {
                                Ok(hcv) => hcv,
                                Err(hcv) => hcv,
                            };
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
                        if let Some((chroma, sum)) =
                            hue.adjusted_favouring_sum(new_sum, self.hcv.chroma)
                        {
                            self.hcv = match HCV::try_new(Some((hue, chroma.into_prop())), sum) {
                                Ok(hcv) => hcv,
                                Err(hcv) => hcv,
                            };
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
        debug_assert!(self.hcv.is_valid());
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
                Outcome::Ok | Outcome::Clamped | Outcome::Accommodated => {
                    debug_assert!(self.hcv.is_valid());
                    true
                }
                _ => false,
            }
        }
    }

    pub fn incr_value(&mut self, delta: Prop) -> bool {
        debug_assert!(self.hcv.is_valid());
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
                Outcome::Ok | Outcome::Clamped | Outcome::Accommodated => {
                    debug_assert!(self.hcv.is_valid());
                    true
                }
                _ => false,
            }
        }
    }

    pub fn set_hue(&mut self, new_hue: Hue, policy: SetHue) {
        // TODO: change argument to Option<Hue>
        debug_assert!(self.hcv.is_valid());
        match self.hcv.chroma {
            Chroma::ZERO => self.saved_hue = new_hue,
            Chroma::ONE => match policy {
                SetHue::FavourChroma => self.hcv = new_hue.max_chroma_hcv(),
                SetHue::FavourValue => {
                    let chroma = new_hue.max_chroma_for_sum(self.hcv.sum).unwrap();
                    let (chroma, sum) = new_hue
                        .adjusted_favouring_sum(self.hcv.sum, chroma)
                        .unwrap();
                    self.hcv = match HCV::try_new(Some((new_hue, chroma.into_prop())), sum) {
                        Ok(hcv) => hcv,
                        Err(hcv) => hcv,
                    };
                }
            },
            chroma => {
                if new_hue.sum_and_chroma_are_compatible(self.hcv.sum, chroma) {
                    self.hcv = match HCV::try_new(Some((new_hue, chroma.into_prop())), self.hcv.sum)
                    {
                        Ok(hcv) => hcv,
                        Err(hcv) => hcv,
                    };
                } else {
                    self.hcv = match policy {
                        SetHue::FavourChroma => {
                            if let Some((chroma, sum)) = if let Some((min_sum, max_sum)) =
                                new_hue.sum_range_for_chroma(chroma)
                            {
                                if self.hcv.sum < min_sum {
                                    new_hue.trim_overs(min_sum + UFDRNumber(2), chroma.into_prop())
                                } else if self.hcv.sum > max_sum {
                                    new_hue.trim_overs(max_sum, chroma.into_prop())
                                } else {
                                    new_hue.trim_overs(self.hcv.sum, chroma.into_prop())
                                }
                            } else {
                                new_hue.trim_overs(self.hcv.sum, chroma.into_prop())
                            } {
                                match HCV::try_new(Some((new_hue, chroma.into_prop())), sum) {
                                    Ok(hcv) => hcv,
                                    Err(hcv) => hcv,
                                }
                            } else {
                                self.saved_hue = new_hue;
                                HCV::new_grey((self.hcv.sum / 3).into())
                            }
                        }
                        SetHue::FavourValue => {
                            let max_chroma = new_hue
                                .max_chroma_for_sum(self.hcv.sum)
                                .expect("0.0 < sum < 3.0");
                            let chroma = if self.hcv.chroma < max_chroma {
                                self.hcv.chroma
                            } else {
                                max_chroma
                            };
                            let (chroma, sum) = new_hue
                                .adjusted_favouring_sum(self.hcv.sum, chroma)
                                .unwrap();
                            match HCV::try_new(Some((new_hue, chroma.into_prop())), sum) {
                                Ok(hcv) => hcv,
                                Err(hcv) => hcv,
                            }
                        }
                    }
                }
            }
        }
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
                    // NB: set_hue() doesn't guarantee hue won't be None
                    if let Some(new_hue) = self.hcv.hue() {
                        self.saved_hue = new_hue;
                    };
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

#[cfg(test)]
mod manipulator_tests;
