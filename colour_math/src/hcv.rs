// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::chroma::HueData;
use crate::{chroma, ColourComponent, HueConstants, RGBConstants, RGB};
use normalised_angles::Degrees;
use std::convert::TryFrom;
use std::io::Read;

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

impl<F: ColourComponent> From<&RGB<F>> for HCV<F> {
    fn from(rgb: &RGB<F>) -> Self {
        let xy = rgb.xy();
        let hypot = xy.0.hypot(xy.1);
        let sum = rgb.iter().copied().sum();
        if hypot > F::ZERO {
            let io = rgb.indices_value_order();
            let second = chroma::calc_other_from_xy_alt(xy);
            let chroma = (hypot * chroma::calc_chroma_correction(second)).min(F::ONE);
            Self {
                hue_data: Some(HueData { io, second }),
                chroma,
                sum,
            }
        } else {
            Self {
                hue_data: None,
                chroma: F::ZERO,
                sum,
            }
        }
    }
}

impl<F: ColourComponent> TryFrom<&HCV<F>> for RGB<F> {
    type Error = (RGB<F>, RGB<F>, RGB<F>);

    fn try_from(hcv: &HCV<F>) -> Result<Self, Self::Error> {
        if let Some(hue_data) = hcv.hue_data {
            if let Some(rgb) = hue_data.rgb_for_sum_and_chroma(hcv.sum, hcv.chroma) {
                Ok(rgb)
            } else {
                let rgb_one = hue_data.min_sum_rgb_for_chroma(hcv.chroma);
                let rgb_two = hue_data.max_sum_rgb_for_chroma(hcv.chroma);
                let rgb_three: RGB<F> = hue_data.max_sum_rgb_for_chroma(hcv.sum);
                Err((rgb_one, rgb_two, rgb_three))
            }
        } else {
            let value = hcv.sum / F::THREE;
            Ok(RGB::from([value, value, value]))
        }
    }
}
