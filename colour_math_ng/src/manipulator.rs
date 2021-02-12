// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::{Hue, HueConstants, LightLevel, HCV, RGB};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RotationPolicy {
    FavourChroma,
    FavourValue,
}

#[derive(Debug)]
pub struct ColourManipulator {
    hcv: HCV,
    clamped: bool,
    rotation_policy: RotationPolicy,
    saved_hue: Hue,
}

#[derive(Debug)]
pub struct ColourManipulatorBuilder {
    init_hcv: Option<HCV>,
    clamped: bool,
    rotation_policy: RotationPolicy,
}

impl ColourManipulatorBuilder {
    pub fn new() -> Self {
        Self {
            init_hcv: None,
            clamped: false,
            rotation_policy: RotationPolicy::FavourChroma,
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

    pub fn rotation_policy(&mut self, rotation_policy: RotationPolicy) -> &mut Self {
        self.rotation_policy = rotation_policy;
        self
    }

    pub fn build(&self) -> ColourManipulator {
        let hcv = if let Some(init_hcv) = self.init_hcv {
            init_hcv
        } else {
            HCV::default()
        };
        let saved_hue = if let Some(hue) = hcv.hue {
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
