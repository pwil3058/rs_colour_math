// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::convert::From;

use num_traits_plus::assert_approx_eq;

use crate::{attributes::Warmth, hcv::*};

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
fn round_trip_from_to_rgb_constants() {
    for rgb in RGB::<u64>::PRIMARIES.iter().chain(
        RGB::<u64>::SECONDARIES
            .iter()
            .chain(RGB::<u64>::IN_BETWEENS.iter()),
    ) {
        let [red, green, blue] = <[Prop; 3]>::from(rgb);
        let rgb_in: RGB<u64> = [red, green, blue].into();
        let hcv = HCV::from(&rgb_in);
        let rgb_out = RGB::<u64>::from(&hcv);
        assert_eq!(rgb_in, rgb_out);
        let rgb_in: RGB<u8> = [red, green, blue].into();
        let hcv = HCV::from(&rgb_in);
        let rgb_out = RGB::<u8>::from(&hcv);
        assert_eq!(rgb_in, rgb_out);
        let rgb_in: RGB<f64> = [red, green, blue].into();
        let hcv = HCV::from(&rgb_in);
        let rgb_out = RGB::<f64>::from(&hcv);
        assert_eq!(rgb_in, rgb_out);
    }
}

#[test]
fn round_trip_from_to_rgb() {
    let values = vec![0.0_f64, 0.001, 0.01, 0.499, 0.5, 0.99, 0.999, 1.0];
    for red in values.iter().map(|l| Prop::from(*l)) {
        for green in values.iter().map(|l| Prop::from(*l)) {
            for blue in values.iter().map(|l| Prop::from(*l)) {
                let rgb_in: RGB<u64> = [red, green, blue].into();
                let hcv = HCV::from(&rgb_in);
                let rgb_out = RGB::<u64>::from(&hcv);
                assert_eq!(rgb_in, rgb_out);
                let rgb_in: RGB<u8> = [red, green, blue].into();
                let hcv = HCV::from(&rgb_in);
                let rgb_out = RGB::<u8>::from(&hcv);
                assert_eq!(rgb_in, rgb_out);
                let rgb_in: RGB<f64> = [red, green, blue].into();
                let hcv = HCV::from(&rgb_in);
                let rgb_out = RGB::<f64>::from(&hcv);
                assert_eq!(rgb_in, rgb_out);
            }
        }
    }
}

#[test]
fn warmth() {
    assert_eq!(HCV::RED.warmth(), Warmth::ONE);
    assert_eq!(HCV::BLUE.warmth(), (Prop::ONE / 2).into());
    assert_eq!(HCV::CYAN.warmth(), (Prop::ONE / 3).into());
    assert_eq!(HCV::YELLOW.warmth(), (Prop::ONE * 5 / 6).into());
    assert_eq!(HCV::WHITE.warmth(), Warmth::ZERO);
    assert_eq!(HCV::BLACK.warmth(), (Prop::ONE / 2).into());
    assert_approx_eq!(
        RGB::<u8>::from([Prop::ONE, Prop::ONE / 2, Prop::ONE / 2]).warmth(),
        (Prop::ONE * 5 / 12).into(),
        0x100000000000000
    )
}

#[test]
fn hcv_add_sub_angle() {
    for hcv in HCV::PRIMARIES
        .iter()
        .chain(HCV::SECONDARIES.iter())
        .chain(HCV::IN_BETWEENS.iter())
    {
        for angle in &[
            Angle::from(15),
            Angle::from(-15),
            Angle::from(135),
            Angle::from(-135),
        ] {
            assert_approx_eq!(
                (*hcv + *angle).hue_angle().unwrap(),
                hcv.hue_angle().unwrap() + *angle,
                0x100
            );
            assert_approx_eq!(
                (*hcv - *angle).hue_angle().unwrap(),
                hcv.hue_angle().unwrap() - *angle,
                0x100
            );
            assert_eq!((*hcv + *angle).chroma(), hcv.chroma());
            assert_eq!((*hcv - *angle).chroma(), hcv.chroma());
        }
    }
}
