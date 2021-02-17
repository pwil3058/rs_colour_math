// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
//use num_traits_plus::assert_approx_eq;

use crate::hue::HueIfce;
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
        let saved_hue = manipulator.hcv.hue();
        let decr = Prop::from(0.1);
        let mut expected = manipulator.hcv.chroma.prop() - decr;
        while manipulator.decr_chroma(decr) {
            assert_eq!(manipulator.hcv.chroma.prop(), expected);
            expected = if manipulator.hcv.chroma.prop() > decr {
                manipulator.hcv.chroma.prop() - decr
            } else {
                Prop::ZERO
            };
            assert_eq!(manipulator.hcv.sum, Sum::TWO);
            if manipulator.hcv.chroma > Chroma::ZERO {
                assert_eq!(manipulator.hcv.hue(), saved_hue);
            }
        }
        assert!(manipulator.hcv.is_grey());
        assert_eq!(manipulator.hcv.chroma, Chroma::ZERO);
        assert_eq!(manipulator.hcv.sum, Sum::TWO);
        assert_eq!(manipulator.hcv.hue(), None);
        assert_eq!(manipulator.saved_hue, saved_hue.unwrap());
    }
}

#[test]
fn incr_chroma_clamped() {
    let mut manipulator = super::ColourManipulatorBuilder::new().clamped(true).build();
    assert_eq!(manipulator.hcv, HCV::BLACK);
    assert!(!manipulator.incr_chroma(0.1_f64.into()));
    // Test where clamping makes a difference and where it doesn't
    for array in &[[0.75_f64, 0.5, 0.0], [0.75, 0.5, 0.75]] {
        let rgb = RGB::from(*array);
        manipulator.set_colour(&rgb);
        let start_sum = manipulator.hcv.sum;
        let saved_hue = manipulator.hcv.hue().unwrap();
        let incr = Prop::from(0.1_f64);
        let mut max_chroma = saved_hue.max_chroma_for_sum(manipulator.hcv.sum).unwrap();
        let mut expected: Prop = (manipulator.hcv.chroma.prop() + incr)
            .min(max_chroma.prop().into())
            .into();
        println!("Max: {:?}", max_chroma);
        println! {"HCV: {:?} incr: {:?} expected: {:?}", manipulator.hcv, incr, expected};
        while manipulator.incr_chroma(incr) {
            println! {"HCV: {:?} incr: {:?} expected: {:?}", manipulator.hcv, incr, expected};
            assert_eq!(manipulator.hcv.chroma.prop(), expected);
            max_chroma = saved_hue.max_chroma_for_sum(manipulator.hcv.sum).unwrap();
            expected = (manipulator.hcv.chroma.prop() + incr)
                .min(max_chroma.prop().into())
                .into();
            assert_eq!(manipulator.hcv.sum, start_sum);
            assert_eq!(manipulator.hcv.hue(), Some(saved_hue));
        }
        assert!(!manipulator.hcv.is_grey());
        assert_eq!(
            manipulator.hcv.chroma,
            saved_hue.max_chroma_for_sum(start_sum).unwrap()
        );
        assert_eq!(manipulator.hcv.sum, start_sum);
        assert_eq!(manipulator.hcv.hue(), Some(saved_hue));
    }
}

#[test]
fn incr_chroma_unclamped() {
    let mut manipulator = super::ColourManipulatorBuilder::new()
        .clamped(false)
        .build();
    assert_eq!(manipulator.hcv, HCV::BLACK);
    assert!(manipulator.incr_chroma(0.1_f64.into()));
    // Test where clamping makes a difference and where it doesn't
    for array in &[[0.75_f64, 0.5, 0.0], [0.75, 0.5, 0.75]] {
        let rgb = RGB::from(*array);
        manipulator.set_colour(&rgb);
        let saved_hue = manipulator.hcv.hue().unwrap();
        let incr: Prop = 0.1_f64.into();
        let mut expected: Prop = (manipulator.hcv.chroma.prop() + incr).min(Sum::ONE).into();
        while manipulator.incr_chroma(incr) {
            assert_eq!(manipulator.hcv.chroma.prop(), expected);
            expected = (manipulator.hcv.chroma.prop() + incr).min(Sum::ONE).into();
            if let Some(range) = saved_hue.sum_range_for_chroma_prop(manipulator.hcv.chroma.prop())
            {
                assert!(range.compare_sum(manipulator.hcv.sum).is_success());
            };
            assert_eq!(manipulator.hcv.hue(), Some(saved_hue));
        }
        assert!(!manipulator.hcv.is_grey());
        assert_eq!(manipulator.hcv.chroma, Chroma::ONE);
        if let Some(range) = saved_hue.sum_range_for_chroma_prop(manipulator.hcv.chroma.prop()) {
            assert!(range.compare_sum(manipulator.hcv.sum).is_success());
        };
        assert_eq!(manipulator.hcv.hue(), Some(saved_hue));
    }
}

#[test]
fn round_trip_chroma() {
    let mut manipulator = super::ColourManipulatorBuilder::new().clamped(true).build();
    manipulator.set_colour(&crate::rgb::RGB::<u64>::CYAN);
    while manipulator.decr_chroma(0.01.into()) {}
    assert!(manipulator.hcv.is_grey());
    while manipulator.incr_chroma(0.01.into()) {}
    assert_eq!(manipulator.rgb::<u64>(), crate::rgb::RGB::<u64>::CYAN);
}
