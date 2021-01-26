// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::marker::PhantomData;

use crate::chroma::HueData;
use crate::urgb::UnsignedComponent;
use crate::{
    chroma, image, ColourComponent, ColourInterface, HueConstants, HueIfce, IndicesValueOrder,
    RGBConstants, RGB, URGB,
};
use normalised_angles::Degrees;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct HCV<F: ColourComponent> {
    pub(crate) hue_data: Option<HueData<F>>,
    pub(crate) chroma: F,
    pub(crate) sum: F,
}

impl<F: ColourComponent> HCV<F> {
    pub fn hue_angle(&self) -> Option<Degrees<F>> {
        match self.hue_data {
            Some(hue_data) => Some(hue_data.hue_angle()),
            None => None,
        }
    }

    pub fn hue_data(&self) -> Option<HueData<F>> {
        self.hue_data
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
            io: IndicesValueOrder::RED,
        }),
        chroma: F::ONE,
        sum: F::ONE,
    };

    const GREEN: Self = Self {
        hue_data: Some(HueData {
            second: F::ZERO,
            io: IndicesValueOrder::GREEN,
        }),
        chroma: F::ONE,
        sum: F::ONE,
    };

    const BLUE: Self = Self {
        hue_data: Some(HueData {
            second: F::ZERO,
            io: IndicesValueOrder::BLUE,
        }),
        chroma: F::ONE,
        sum: F::ONE,
    };

    const CYAN: Self = Self {
        hue_data: Some(HueData {
            second: F::ONE,
            io: IndicesValueOrder::CYAN,
        }),
        chroma: F::ONE,
        sum: F::TWO,
    };

    const MAGENTA: Self = Self {
        hue_data: Some(HueData {
            second: F::ONE,
            io: IndicesValueOrder::MAGENTA,
        }),
        chroma: F::ONE,
        sum: F::TWO,
    };

    const YELLOW: Self = Self {
        hue_data: Some(HueData {
            second: F::ONE,
            io: IndicesValueOrder::YELLOW,
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

impl<F: ColourComponent + ChromaTolerance> From<&HCV<F>> for RGB<F> {
    fn from(hcv: &HCV<F>) -> Self {
        if let Some(hue_data) = hcv.hue_data {
            if let Some(rgb) = hue_data.rgb_for_sum_and_chroma(hcv.sum, hcv.chroma) {
                rgb
            } else {
                // This can possibly be due floating point arithmetic's inability to properly
                // represent reals resulting in the HCV having a chroma value slightly higher
                // than that which is possible for the hue and sum so we'll check if the RGB
                // with the maximum chroma for the hue and sum is approximately equal to the HCV's
                // chroma and if so use that.
                let rgb = hue_data.max_chroma_rgb_for_sum(hcv.sum);
                if rgb.chroma().approx_eq(&hcv.chroma, Some(F::COMA_TOLERANCE)) {
                    rgb
                } else {
                    panic!("This HCV does not represent a valid colour")
                }
            }
        } else {
            debug_assert_eq!(hcv.chroma, F::ZERO);
            let value = hcv.sum / F::THREE;
            debug_assert!(value >= F::ZERO && value <= F::ONE);
            RGB::from([value, value, value])
        }
    }
}

impl<U: UnsignedComponent, F: ColourComponent + ChromaTolerance> From<&URGB<U>> for HCV<F> {
    fn from(urgb: &URGB<U>) -> Self {
        let rgb: RGB<F> = urgb.into();
        Self::from(&rgb)
    }
}

impl<U: UnsignedComponent, F: ColourComponent + ChromaTolerance> From<&HCV<F>> for URGB<U> {
    fn from(hcv: &HCV<F>) -> Self {
        let rgb: RGB<F> = hcv.into();
        Self::from(rgb)
    }
}

#[derive(Default)]
struct ToMonochrome<F: ColourComponent + ChromaTolerance> {
    phantom_data: PhantomData<F>,
}

impl<F: ColourComponent + ChromaTolerance> image::Transformer<HCV<F>> for ToMonochrome<F> {
    fn transform(&self, pixel: &HCV<F>) -> HCV<F> {
        HCV {
            hue_data: None,
            chroma: F::ZERO,
            sum: pixel.sum,
        }
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
        assert_eq!(RGB::<f64>::from(&HCV::<f64>::RED), RGB::RED);
        assert_eq!(RGB::<f64>::from(&HCV::<f64>::GREEN), RGB::GREEN);
        assert_eq!(RGB::<f64>::from(&HCV::<f64>::BLUE), RGB::BLUE);
        assert_eq!(RGB::<f64>::from(&HCV::<f64>::CYAN), RGB::CYAN);
        assert_eq!(RGB::<f64>::from(&HCV::<f64>::MAGENTA), RGB::MAGENTA);
        assert_eq!(RGB::<f64>::from(&HCV::<f64>::YELLOW), RGB::YELLOW);
        assert_eq!(RGB::<f64>::from(&HCV::<f64>::WHITE), RGB::WHITE);
        assert_eq!(RGB::<f64>::from(&HCV::<f64>::BLACK), RGB::BLACK);
    }

    #[test]
    fn create_urgb_consts() {
        assert_eq!(URGB::<u8>::from(&HCV::<f64>::RED), URGB::RED);
        assert_eq!(URGB::<u8>::from(&HCV::<f64>::GREEN), URGB::GREEN);
        assert_eq!(URGB::<u8>::from(&HCV::<f64>::BLUE), URGB::BLUE);
        assert_eq!(URGB::<u8>::from(&HCV::<f64>::CYAN), URGB::CYAN);
        assert_eq!(URGB::<u8>::from(&HCV::<f64>::MAGENTA), URGB::MAGENTA);
        assert_eq!(URGB::<u8>::from(&HCV::<f64>::YELLOW), URGB::YELLOW);
        assert_eq!(URGB::<u8>::from(&HCV::<f64>::WHITE), URGB::WHITE);
        assert_eq!(URGB::<u8>::from(&HCV::<f64>::BLACK), URGB::BLACK);
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
                    let rgb_out = RGB::<f32>::from(&hcv);
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
                    let rgb_out = RGB::<f64>::from(&hcv);
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
                    let urgb_out = URGB::<u8>::from(&hcv);
                    assert_eq!(urgb_in, urgb_out);
                }
            }
        }
    }
}