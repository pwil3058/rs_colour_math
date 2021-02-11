// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::convert::TryFrom;

use crate::{
    hue::{CMYHue, HueIfce, RGBHue},
    rgb::RGB,
    Chroma, Hue, HueConstants, LightLevel, Prop, RGBConstants, Sum,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct HCV {
    pub(crate) hue: Option<Hue>,
    pub(crate) chroma: Chroma,
    pub(crate) sum: Sum,
}

impl HueConstants for HCV {
    const RED: Self = Self {
        hue: Some(Hue::Primary(RGBHue::Red)),
        chroma: Chroma::ONE,
        sum: Sum::ONE,
    };

    const GREEN: Self = Self {
        hue: Some(Hue::Primary(RGBHue::Green)),
        chroma: Chroma::ONE,
        sum: Sum::ONE,
    };

    const BLUE: Self = Self {
        hue: Some(Hue::Primary(RGBHue::Blue)),
        chroma: Chroma::ONE,
        sum: Sum::ONE,
    };

    const CYAN: Self = Self {
        hue: Some(Hue::Secondary(CMYHue::Cyan)),
        chroma: Chroma::ONE,
        sum: Sum::TWO,
    };

    const MAGENTA: Self = Self {
        hue: Some(Hue::Secondary(CMYHue::Magenta)),
        chroma: Chroma::ONE,
        sum: Sum::TWO,
    };

    const YELLOW: Self = Self {
        hue: Some(Hue::Secondary(CMYHue::Yellow)),
        chroma: Chroma::ONE,
        sum: Sum::TWO,
    };
}

impl RGBConstants for HCV {
    const WHITE: Self = Self {
        hue: None,
        chroma: Chroma::ZERO,
        sum: Sum::THREE,
    };

    const BLACK: Self = Self {
        hue: None,
        chroma: Chroma::ZERO,
        sum: Sum::ZERO,
    };
}

impl<L: LightLevel> From<&RGB<L>> for HCV {
    fn from(rgb: &RGB<L>) -> Self {
        if let Ok(hue) = Hue::try_from(rgb) {
            let prop = rgb.chroma().prop();
            let sum = rgb.sum();
            let sum_range = hue
                .sum_range_for_chroma(Chroma::Either(prop))
                .expect("RGB exists");
            let chroma = if sum >= sum_range.crossover() {
                Chroma::Tint(prop)
            } else {
                Chroma::Shade(prop)
            };
            Self {
                hue: Some(hue),
                chroma,
                sum,
            }
        } else {
            Self {
                hue: None,
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

    fn rgb<L: LightLevel>(&self) -> RGB<L>;
}

impl ColourIfce for HCV {
    fn hue(&self) -> Option<Hue> {
        if let Some(hue) = self.hue {
            Some(hue)
        } else {
            None
        }
    }
    fn chroma(&self) -> Chroma {
        self.chroma
    }

    fn value(&self) -> Prop {
        self.sum / 3
    }

    fn rgb<L: LightLevel>(&self) -> RGB<L> {
        if let Some(hue) = self.hue {
            if let Some(rgb) = hue.rgb_for_sum_and_chroma::<L>(self.sum, self.chroma) {
                rgb
            } else {
                // This can possibly be due floating point arithmetic's inability to properly
                // represent reals resulting in the HCV having a sum value slightly higher/lower
                // than that which is possible for the hue and chroma.
                match self.chroma {
                    Chroma::Shade(_) => hue.min_sum_rgb_for_chroma(self.chroma),
                    _ => hue.max_sum_rgb_for_chroma(self.chroma),
                }
            }
        } else {
            RGB::new_grey(self.value())
        }
    }
}

#[cfg(test)]
mod hcv_tests;
