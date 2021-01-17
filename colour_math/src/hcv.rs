// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::chroma::HueData;
use crate::urgb::UnsignedComponent;
use crate::{
    chroma, ColourComponent, ColourInterface, HueConstants, HueIfce, RGBConstants, RGB, URGB,
};
use normalised_angles::Degrees;
use std::convert::{TryFrom, TryInto};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
            io: [2, 0, 1],
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
        debug_assert!(rgb.is_valid());
        let xy = rgb.xy();
        let hypot = xy.0.hypot(xy.1);
        let sum = rgb.iter().copied().sum();
        debug_assert!(sum <= F::THREE);
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

pub trait ChromaTolerance {
    const COMA_TOLERANCE: Self;
}

impl ChromaTolerance for f32 {
    const COMA_TOLERANCE: Self = 0.000_001;
}

impl ChromaTolerance for f64 {
    const COMA_TOLERANCE: Self = 0.000_000_000_01;
}

impl<F: ColourComponent + ChromaTolerance> TryFrom<&HCV<F>> for RGB<F> {
    type Error = String;

    fn try_from(hcv: &HCV<F>) -> Result<Self, Self::Error> {
        if let Some(hue_data) = hcv.hue_data {
            if let Some(rgb) = hue_data.rgb_for_sum_and_chroma(hcv.sum, hcv.chroma) {
                Ok(rgb)
            } else {
                // This can possibly be due floating point arithmetic's inability to properly
                // represent reals resulting in the HCV having a chroma value slightly higher
                // than that which is possible for the hue and sum so we'll check if the RGB
                // with the maximum chroma for the hue and sum is approximately equal to the HCV's
                // chroma and if so use that.
                let rgb = hue_data.max_chroma_rgb_for_sum(hcv.sum);
                if rgb.chroma().approx_eq(&hcv.chroma, Some(F::COMA_TOLERANCE)) {
                    Ok(rgb)
                } else {
                    Err("This HCV does not represent a valid colour".to_string())
                }
            }
        } else {
            debug_assert_eq!(hcv.chroma, F::ZERO);
            let value = hcv.sum / F::THREE;
            debug_assert!(value >= F::ZERO && value <= F::ONE);
            Ok(RGB::from([value, value, value]))
        }
    }
}

impl<U: UnsignedComponent, F: ColourComponent + ChromaTolerance> From<&URGB<U>> for HCV<F> {
    fn from(urgb: &URGB<U>) -> Self {
        let rgb: RGB<F> = urgb.into();
        Self::from(&rgb)
    }
}

impl<U: UnsignedComponent, F: ColourComponent + ChromaTolerance> TryFrom<&HCV<F>> for URGB<U> {
    type Error = String;

    fn try_from(hcv: &HCV<F>) -> Result<Self, Self::Error> {
        let rgb: RGB<F> = hcv.try_into()?;
        Ok(Self::from(rgb))
    }
}

#[cfg(test)]
mod hcv_tests {
    use super::*;
    use num_traits_plus::{assert_approx_eq, float_plus::FloatApproxEq};

    #[test]
    fn default_hcv_is_black() {
        assert_eq!(HCV::<f64>::default(), HCV::BLACK);
        assert_eq!(HCV::<f32>::default(), HCV::BLACK);
    }

    #[test]
    fn create_hcv_consts() {
        assert_eq!(HCV::<f64>::from(&RGB::<f64>::RED), HCV::RED);
        assert_eq!(HCV::<f64>::from(&RGB::<f64>::GREEN), HCV::GREEN);
        assert_eq!(HCV::<f64>::from(&RGB::<f64>::BLUE), HCV::BLUE);
        assert_eq!(HCV::<f64>::from(&RGB::<f64>::CYAN), HCV::CYAN);
        assert_eq!(HCV::<f64>::from(&RGB::<f64>::MAGENTA), HCV::MAGENTA);
        assert_eq!(HCV::<f64>::from(&RGB::<f64>::YELLOW), HCV::YELLOW);
        assert_eq!(HCV::<f64>::from(&RGB::<f64>::WHITE), HCV::WHITE);
        assert_eq!(HCV::<f64>::from(&RGB::<f64>::BLACK), HCV::BLACK);
    }

    #[test]
    fn create_hcv_consts_u8() {
        assert_eq!(HCV::<f64>::from(&URGB::<u8>::RED), HCV::RED);
        assert_eq!(HCV::<f64>::from(&URGB::<u8>::GREEN), HCV::GREEN);
        assert_eq!(HCV::<f64>::from(&URGB::<u8>::BLUE), HCV::BLUE);
        assert_eq!(HCV::<f64>::from(&URGB::<u8>::CYAN), HCV::CYAN);
        assert_eq!(HCV::<f64>::from(&URGB::<u8>::MAGENTA), HCV::MAGENTA);
        assert_eq!(HCV::<f64>::from(&URGB::<u8>::YELLOW), HCV::YELLOW);
        assert_eq!(HCV::<f64>::from(&URGB::<u8>::WHITE), HCV::WHITE);
        assert_eq!(HCV::<f64>::from(&URGB::<u8>::BLACK), HCV::BLACK);
    }

    #[test]
    fn create_rgb_consts() {
        assert_eq!(RGB::<f64>::try_from(&HCV::<f64>::RED).unwrap(), RGB::RED);
        assert_eq!(
            RGB::<f64>::try_from(&HCV::<f64>::GREEN).unwrap(),
            RGB::GREEN
        );
        assert_eq!(RGB::<f64>::try_from(&HCV::<f64>::BLUE).unwrap(), RGB::BLUE);
        assert_eq!(RGB::<f64>::try_from(&HCV::<f64>::CYAN).unwrap(), RGB::CYAN);
        assert_eq!(
            RGB::<f64>::try_from(&HCV::<f64>::MAGENTA).unwrap(),
            RGB::MAGENTA
        );
        assert_eq!(
            RGB::<f64>::try_from(&HCV::<f64>::YELLOW).unwrap(),
            RGB::YELLOW
        );
        assert_eq!(
            RGB::<f64>::try_from(&HCV::<f64>::WHITE).unwrap(),
            RGB::WHITE
        );
        assert_eq!(
            RGB::<f64>::try_from(&HCV::<f64>::BLACK).unwrap(),
            RGB::BLACK
        );
    }

    #[test]
    fn create_urgb_consts() {
        assert_eq!(URGB::<u8>::try_from(&HCV::<f64>::RED).unwrap(), URGB::RED);
        assert_eq!(
            URGB::<u8>::try_from(&HCV::<f64>::GREEN).unwrap(),
            URGB::GREEN
        );
        assert_eq!(URGB::<u8>::try_from(&HCV::<f64>::BLUE).unwrap(), URGB::BLUE);
        assert_eq!(URGB::<u8>::try_from(&HCV::<f64>::CYAN).unwrap(), URGB::CYAN);
        assert_eq!(
            URGB::<u8>::try_from(&HCV::<f64>::MAGENTA).unwrap(),
            URGB::MAGENTA
        );
        assert_eq!(
            URGB::<u8>::try_from(&HCV::<f64>::YELLOW).unwrap(),
            URGB::YELLOW
        );
        assert_eq!(
            URGB::<u8>::try_from(&HCV::<f64>::WHITE).unwrap(),
            URGB::WHITE
        );
        assert_eq!(
            URGB::<u8>::try_from(&HCV::<f64>::BLACK).unwrap(),
            URGB::BLACK
        );
    }

    #[test]
    fn from_to_rgb_f32() {
        let values = vec![0.0_f32, 0.001, 0.01, 0.499, 0.5, 0.99, 0.999, 1.0];
        for red in values.iter() {
            for green in values.iter() {
                for blue in values.iter() {
                    let rgb_in: RGB<f32> = [*red, *green, *blue].into();
                    println!("[{}, {}, {}] -> {:?}", red, green, blue, rgb_in);
                    let hcv = HCV::<f32>::from(&rgb_in);
                    println!("{:?}", hcv);
                    let rgb_out = RGB::<f32>::try_from(&hcv).unwrap();
                    assert_approx_eq!(rgb_in, rgb_out);
                }
            }
        }
    }

    #[test]
    fn from_to_rgb_f64() {
        let values = vec![0.0_f64, 0.001, 0.01, 0.499, 0.5, 0.99, 0.999, 1.0];
        for red in values.iter() {
            for green in values.iter() {
                for blue in values.iter() {
                    let rgb_in: RGB<f64> = [*red, *green, *blue].into();
                    println!("[{}, {}, {}] -> {:?}", red, green, blue, rgb_in);
                    let hcv = HCV::<f64>::from(&rgb_in);
                    println!("{:?}", hcv);
                    let rgb_out = RGB::<f64>::try_from(&hcv).unwrap();
                    assert_approx_eq!(rgb_in, rgb_out);
                }
            }
        }
    }

    #[test]
    fn from_to_rgb_u8() {
        let values = vec![0u8, 1, 2, 127, 128, 253, 254, 255];
        for red in values.iter() {
            for green in values.iter() {
                for blue in values.iter() {
                    let urgb_in: URGB<u8> = [*red, *green, *blue].into();
                    println!("[{}, {}, {}] -> {:?}", red, green, blue, urgb_in);
                    let hcv = HCV::<f64>::from(&urgb_in);
                    println!("{:?}", hcv);
                    let urgb_out = URGB::<u8>::try_from(&hcv).unwrap();
                    assert_eq!(urgb_in, urgb_out);
                }
            }
        }
    }
}
