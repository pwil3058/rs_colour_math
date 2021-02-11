// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::convert::From;

//use num_traits_plus::assert_approx_eq;

use crate::hcv::*;

#[test]
fn default_hcv_is_black() {
    assert_eq!(HCV::default(), HCV::BLACK);
}

#[test]
fn create_hcv_consts_from_float() {
    assert_eq!(HCV::from(&RGB::<f64>::RED), HCV::RED);
    assert_eq!(HCV::from(&RGB::<f64>::GREEN), HCV::GREEN);
    assert_eq!(HCV::from(&RGB::<f64>::BLUE), HCV::BLUE);
    assert_eq!(HCV::from(&RGB::<f64>::CYAN), HCV::CYAN);
    assert_eq!(HCV::from(&RGB::<f64>::MAGENTA), HCV::MAGENTA);
    assert_eq!(HCV::from(&RGB::<f64>::YELLOW), HCV::YELLOW);
    assert_eq!(HCV::from(&RGB::<f64>::WHITE), HCV::WHITE);
    assert_eq!(HCV::from(&RGB::<f64>::BLACK), HCV::BLACK);
}

#[test]
fn create_hcv_consts_from_unsigned() {
    assert_eq!(HCV::from(&RGB::<u16>::RED), HCV::RED);
    assert_eq!(HCV::from(&RGB::<u16>::GREEN), HCV::GREEN);
    assert_eq!(HCV::from(&RGB::<u16>::BLUE), HCV::BLUE);
    assert_eq!(HCV::from(&RGB::<u16>::CYAN), HCV::CYAN);
    assert_eq!(HCV::from(&RGB::<u16>::MAGENTA), HCV::MAGENTA);
    assert_eq!(HCV::from(&RGB::<u16>::YELLOW), HCV::YELLOW);
    assert_eq!(HCV::from(&RGB::<u16>::WHITE), HCV::WHITE);
    assert_eq!(HCV::from(&RGB::<u16>::BLACK), HCV::BLACK);
}

#[test]
fn from_to_primary() {
    for (rgbf64, rgbu16) in RGB::<f64>::PRIMARIES.iter().zip(&RGB::<u16>::PRIMARIES) {
        let hcv: HCV = rgbf64.into();
        assert_eq!(rgbf64, &RGB::<f64>::from(&hcv));
        assert_eq!(rgbu16, &RGB::<u16>::from(&hcv));
        let hcv: HCV = rgbu16.into();
        assert_eq!(rgbf64, &RGB::<f64>::from(&hcv));
        assert_eq!(rgbu16, &RGB::<u16>::from(&hcv));
    }
}

#[test]
fn from_to_secondary() {
    for (rgbf64, rgbu16) in RGB::<f64>::SECONDARIES.iter().zip(&RGB::<u16>::SECONDARIES) {
        let hcv: HCV = rgbf64.into();
        assert_eq!(rgbf64, &RGB::<f64>::from(&hcv));
        assert_eq!(rgbu16, &RGB::<u16>::from(&hcv));
        let hcv: HCV = rgbu16.into();
        assert_eq!(rgbf64, &RGB::<f64>::from(&hcv));
        assert_eq!(rgbu16, &RGB::<u16>::from(&hcv));
    }
}

#[test]
fn from_to_rgb_f64() {
    let values = vec![0.0_f64, 0.001, 0.01, 0.499, 0.5, 0.99, 0.999, 1.0];
    for red in values.iter().map(|l| Prop::from(*l)) {
        for green in values.iter().map(|l| Prop::from(*l)) {
            for blue in values.iter().map(|l| Prop::from(*l)) {
                let rgb_in: RGB<f64> = [red, green, blue].into();
                println!("[{:?}, {:?}, {:?}] -> {:?}", red, green, blue, rgb_in);
                let hcv = HCV::from(&rgb_in);
                println!("{:?}", hcv);
                let rgb_out = RGB::<f64>::from(&hcv);
                assert_eq!(rgb_in, rgb_out);
            }
        }
    }
}
