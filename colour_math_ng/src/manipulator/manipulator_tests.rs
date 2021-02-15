// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
//use num_traits_plus::assert_approx_eq;

use crate::manipulator::{ColourManipulatorBuilder, RotationPolicy};
use crate::{hcv::*, Chroma, Hue, HueConstants, Prop, RGBConstants, Sum, RGB};

#[test]
fn build_manipulator() {
    let manipualor = ColourManipulatorBuilder::new().build();
    assert_eq!(manipualor.clamped, false);
    assert_eq!(manipualor.rotation_policy, RotationPolicy::FavourChroma);
    assert_eq!(manipualor.hcv(), HCV::default());
    assert_eq!(manipualor.saved_hue, Hue::RED);
    let manipualor = ColourManipulatorBuilder::new().clamped(true).build();
    assert_eq!(manipualor.clamped, true);
    assert_eq!(manipualor.rotation_policy, RotationPolicy::FavourChroma);
    assert_eq!(manipualor.hcv(), HCV::default());
    assert_eq!(manipualor.saved_hue, Hue::RED);
    let manipualor = ColourManipulatorBuilder::new()
        .rotation_policy(RotationPolicy::FavourValue)
        .build();
    assert_eq!(manipualor.clamped, false);
    assert_eq!(manipualor.rotation_policy, RotationPolicy::FavourValue);
    assert_eq!(manipualor.hcv(), HCV::default());
    assert_eq!(manipualor.saved_hue, Hue::RED);
    let manipualor = ColourManipulatorBuilder::new()
        .init_rgb(&RGB::<u8>::CYAN)
        .build();
    assert_eq!(manipualor.clamped, false);
    assert_eq!(manipualor.rotation_policy, RotationPolicy::FavourChroma);
    assert_eq!(manipualor.hcv(), HCV::CYAN);
    assert_eq!(manipualor.rgb::<u8>(), RGB::CYAN);
    assert_eq!(manipualor.saved_hue, Hue::CYAN);
    let manipualor = ColourManipulatorBuilder::new()
        .clamped(true)
        .init_hcv(&HCV::YELLOW)
        .rotation_policy(RotationPolicy::FavourValue)
        .build();
    assert_eq!(manipualor.clamped, true);
    assert_eq!(manipualor.rotation_policy, RotationPolicy::FavourValue);
    assert_eq!(manipualor.hcv(), HCV::YELLOW);
    assert_eq!(manipualor.rgb::<u8>(), RGB::YELLOW);
    assert_eq!(manipualor.saved_hue, Hue::YELLOW);
}

#[test]
fn set_get_parameters() {
    let mut manipualor = ColourManipulatorBuilder::new().build();
    let ll_list = [
        0_u8,
        1_u8,
        u8::MAX / 2 - 1,
        u8::MAX / 2,
        u8::MAX / 2 + 1,
        u8::MAX - 1,
        u8::MAX,
    ];
    for red in &ll_list {
        for green in &ll_list {
            for blue in &ll_list {
                let rgb = RGB::<u8>::from([*red, *green, *blue]);
                manipualor.set_colour(&rgb);
                println!(
                    "[{:?}, {:?}, {:?}] -> {:?}",
                    red, green, blue, manipualor.hcv
                );
                assert_eq!(rgb, manipualor.rgb());
            }
        }
    }
    for clamped in &[true, true, false, false, true, false, true, true] {
        manipualor.set_clamped(*clamped);
        assert_eq!(*clamped, manipualor.clamped());
    }
    for rotation_policy in &[
        RotationPolicy::FavourValue,
        RotationPolicy::FavourValue,
        RotationPolicy::FavourChroma,
        RotationPolicy::FavourChroma,
        RotationPolicy::FavourValue,
        RotationPolicy::FavourChroma,
        RotationPolicy::FavourValue,
        RotationPolicy::FavourValue,
    ] {
        manipualor.set_rotation_policy(*rotation_policy);
        assert_eq!(*rotation_policy, manipualor.rotation_policy());
    }
}

#[test]
fn decr_chroma() {
    // clamping should make no difference to chroma decrementing
    for clamped in &[true, false] {
        let mut manipulator = ColourManipulatorBuilder::new().clamped(*clamped).build();
        assert_eq!(manipulator.hcv, HCV::BLACK);
        assert!(!manipulator.decr_chroma(0.1_f64.into()));
        manipulator.set_colour(&RGB::<u64>::YELLOW);
        assert_eq!(manipulator.hcv.chroma, Chroma::ONE);
        let saved_hue = manipulator.hcv.hue;
        let decr = Prop::from(0.1);
        let mut expected = manipulator.hcv.chroma - decr;
        while manipulator.decr_chroma(decr) {
            assert_eq!(manipulator.hcv.chroma, expected);
            expected = manipulator.hcv.chroma - decr;
            assert_eq!(manipulator.hcv.sum, Sum::TWO);
            if manipulator.hcv.chroma > Chroma::ZERO {
                assert_eq!(manipulator.hcv.hue, saved_hue);
            }
        }
        assert!(manipulator.hcv.is_grey());
        assert_eq!(manipulator.hcv.chroma, Chroma::ZERO);
        assert_eq!(manipulator.hcv.sum, Sum::TWO);
        assert_eq!(manipulator.hcv.hue, None);
        assert_eq!(manipulator.saved_hue, saved_hue.unwrap());
    }
}
