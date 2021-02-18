// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::hcv::{Outcome, SetScalar};
use crate::{
    hcv::{ColourIfce, SetHue},
    Angle, Hue, HueAngle, HueConstants, LightLevel, Prop, HCV, RGB,
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

    pub fn set_colour<L: LightLevel>(&mut self, rgb: &RGB<L>) {
        self.hcv = rgb.into()
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
                match self.hcv.set_chroma_value(new_chroma_value, policy) {
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
                let outcome = self.hcv.set_chroma_value(new_chroma_value, policy);
                match outcome {
                    Outcome::Ok | Outcome::Clamped | Outcome::Accommodated => true,
                    _ => false,
                }
            }
        }
    }

    pub fn rotate(&mut self, angle: Angle) -> bool {
        match self.hcv.hue_angle() {
            None => false,
            Some(cur_angle) => {
                let new_angle = cur_angle + angle;
                if new_angle == cur_angle {
                    false
                } else {
                    self.hcv.set_hue(Hue::from(new_angle), self.rotation_policy);
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

#[cfg(test)]
mod manipulator_tests;
