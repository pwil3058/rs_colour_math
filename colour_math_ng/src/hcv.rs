// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::hue::{CMYHue, RGBHue};
use crate::{Chroma, Hue, HueConstants, RGBConstants, Sum};

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
