// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use crate::manipulator::{ColourManipulatorBuilder, RotationPolicy};
use crate::{hcv::*, Hue, HueConstants, RGB};

#[test]
fn build_manipulator() {
    let manipualor = ColourManipulatorBuilder::new().build();
    assert_eq!(manipualor.clamped, false);
    assert_eq!(manipualor.rotation_policy, RotationPolicy::FavourChroma);
    assert_eq!(manipualor.hcv, HCV::default());
    assert_eq!(manipualor.saved_hue, Hue::RED);
    let manipualor = ColourManipulatorBuilder::new().clamped(true).build();
    assert_eq!(manipualor.clamped, true);
    assert_eq!(manipualor.rotation_policy, RotationPolicy::FavourChroma);
    assert_eq!(manipualor.hcv, HCV::default());
    assert_eq!(manipualor.saved_hue, Hue::RED);
    let manipualor = ColourManipulatorBuilder::new()
        .rotation_policy(RotationPolicy::FavourValue)
        .build();
    assert_eq!(manipualor.clamped, false);
    assert_eq!(manipualor.rotation_policy, RotationPolicy::FavourValue);
    assert_eq!(manipualor.hcv, HCV::default());
    assert_eq!(manipualor.saved_hue, Hue::RED);
    let manipualor = ColourManipulatorBuilder::new()
        .init_rgb(&RGB::<u8>::CYAN)
        .build();
    assert_eq!(manipualor.clamped, false);
    assert_eq!(manipualor.rotation_policy, RotationPolicy::FavourChroma);
    assert_eq!(manipualor.hcv, HCV::CYAN);
    assert_eq!(manipualor.saved_hue, Hue::CYAN);
    let manipualor = ColourManipulatorBuilder::new()
        .clamped(true)
        .init_hcv(&HCV::YELLOW)
        .rotation_policy(RotationPolicy::FavourValue)
        .build();
    assert_eq!(manipualor.clamped, true);
    assert_eq!(manipualor.rotation_policy, RotationPolicy::FavourValue);
    assert_eq!(manipualor.hcv, HCV::YELLOW);
    assert_eq!(manipualor.saved_hue, Hue::YELLOW);
}
