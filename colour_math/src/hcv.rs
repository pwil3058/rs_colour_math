// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::chroma::HueData;
use crate::{ColourComponent, HueConstants, RGBConstants};
use normalised_angles::Degrees;

#[derive(Debug, Clone, Copy)]
pub struct HCV<F: ColourComponent> {
    hue_data: Option<HueData<F>>,
    chroma: F,
    sum: F,
}

impl<F: ColourComponent> HCV<F> {
    pub fn hue_angle(&self) -> Option<Degrees<F>> {
        match self.hue_data {
            Some(hue_data) => Some(hue_data.hue_angle()),
            None => None,
        }
    }

    pub fn hue_data(&self) -> Option<&HueData<F>> {
        match self.hue_data {
            Some(ref hue_data) => Some(hue_data),
            None => None,
        }
    }

    pub fn chroma(&self) -> F {
        self.chroma
    }

    pub fn value(&self) -> F {
        self.sum / F::THREE
    }

    pub fn is_grey(&self) -> bool {
        self.hue_data.is_none()
    }
}

impl<F: ColourComponent> HueConstants for HCV<F> {
    const RED: Self = Self {
        hue_data: Some(HueData {
            second: F::ZERO,
            io: [0, 1, 2],
        }),
        chroma: F::ONE,
        sum: F::ONE,
    };

    const GREEN: Self = Self {
        hue_data: Some(HueData {
            second: F::ZERO,
            io: [1, 0, 2],
        }),
        chroma: F::ONE,
        sum: F::ONE,
    };

    const BLUE: Self = Self {
        hue_data: Some(HueData {
            second: F::ZERO,
            io: [2, 1, 0],
        }),
        chroma: F::ONE,
        sum: F::ONE,
    };

    const CYAN: Self = Self {
        hue_data: Some(HueData {
            second: F::ONE,
            io: [1, 2, 0],
        }),
        chroma: F::ONE,
        sum: F::TWO,
    };

    const MAGENTA: Self = Self {
        hue_data: Some(HueData {
            second: F::ONE,
            io: [0, 2, 1],
        }),
        chroma: F::ONE,
        sum: F::TWO,
    };

    const YELLOW: Self = Self {
        hue_data: Some(HueData {
            second: F::ONE,
            io: [0, 1, 2],
        }),
        chroma: F::ONE,
        sum: F::TWO,
    };
}

impl<F: ColourComponent> RGBConstants for HCV<F> {
    const WHITE: Self = Self {
        hue_data: None,
        chroma: F::ZERO,
        sum: F::THREE,
    };

    const BLACK: Self = Self {
        hue_data: None,
        chroma: F::ZERO,
        sum: F::ZERO,
    };
}
