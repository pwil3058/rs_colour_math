// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::convert::From;

use super::*;

use num_traits_plus::assert_approx_eq;

use crate::{
    attributes::Chroma,
    hue::{Hue, Sextant::*},
    rgb::RGB,
    ColourBasics, RGBConstants,
};

const NON_ZERO_CHROMA_PROPS: [f64; 7] = [0.01, 0.025, 0.5, 0.75, 0.9, 0.99, 1.0];
const SHADE_TINT_CHROMA_PROPS: [f64; 8] = [0.001, 0.01, 0.025, 0.5, 0.75, 0.9, 0.99, 0.999];
const VALID_HUE_SUMS: [f64; 22] = [
    0.00001,
    0.01,
    0.025,
    0.5,
    0.75,
    0.9,
    0.99999,
    1.0,
    1.000000001,
    1.025,
    1.5,
    1.75,
    1.9,
    1.99999,
    2.0,
    2.000000001,
    2.025,
    2.5,
    2.75,
    2.9,
    2.99,
    2.9999,
];
// "second" should never be 0.0 or 1.0
const SECOND_VALUES: [f64; 12] = [
    0.001, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 0.99, 0.999,
];

impl RGBHue {
    fn indices(&self) -> (usize, usize, usize) {
        match self {
            RGBHue::Red => (0, 1, 2),
            RGBHue::Green => (1, 0, 2),
            RGBHue::Blue => (2, 0, 1),
        }
    }
}

impl CMYHue {
    fn indices(&self) -> (usize, usize, usize) {
        match self {
            CMYHue::Magenta => (0, 2, 1),
            CMYHue::Yellow => (0, 1, 2),
            CMYHue::Cyan => (1, 2, 0),
        }
    }
}

impl Sextant {
    fn indices(&self) -> (usize, usize, usize) {
        match self {
            Sextant::RedYellow => (0, 1, 2),
            Sextant::RedMagenta => (0, 2, 1),
            Sextant::GreenYellow => (1, 0, 2),
            Sextant::GreenCyan => (1, 2, 0),
            Sextant::BlueMagenta => (2, 0, 1),
            Sextant::BlueCyan => (2, 1, 0),
        }
    }
}

impl SextantHue {
    fn indices(&self) -> (usize, usize, usize) {
        self.0.indices()
    }
}

impl Hue {
    fn indices(&self) -> (usize, usize, usize) {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.indices(),
            Self::Secondary(cmy_hue) => cmy_hue.indices(),
            Self::Sextant(sextant_hue) => sextant_hue.indices(),
        }
    }
}

#[test]
fn hue_approx_eq() {
    assert!(Hue::RED.approx_eq(&Hue::RED, None));
    assert!(!Hue::RED.approx_eq(&Hue::BLUE, None));
    assert!(!Hue::RED.approx_eq(&Hue::BLUE, Some(Prop::ONE)));

    assert!(Hue::CYAN.approx_eq(&Hue::CYAN, None));
    assert!(!Hue::CYAN.approx_eq(&Hue::MAGENTA, None));
    assert!(!Hue::CYAN.approx_eq(&Hue::MAGENTA, Some(Prop::ONE)));

    for rgb_hue in Hue::PRIMARIES.iter() {
        for cmy_hue in Hue::SECONDARIES.iter() {
            assert!(!rgb_hue.approx_eq(cmy_hue, None));
            assert!(!rgb_hue.approx_eq(cmy_hue, Some(Prop::ONE)));
            assert!(!cmy_hue.approx_eq(rgb_hue, None));
            assert!(!cmy_hue.approx_eq(rgb_hue, Some(Prop::ONE)));
        }
    }

    assert!(
        !Hue::Sextant(SextantHue(RedYellow, Prop(0x1000000000))).approx_eq(
            &Hue::Sextant(SextantHue(GreenYellow, Prop(0x1000000000))),
            None
        )
    );

    assert!(
        Hue::Sextant(SextantHue(RedYellow, Prop::ALMOST_ONE)).approx_eq(
            &Hue::Sextant(SextantHue(GreenYellow, Prop::ALMOST_ONE)),
            None
        )
    );

    assert!(Hue::Sextant(SextantHue(RedYellow, Prop(1)))
        .approx_eq(&Hue::Sextant(SextantHue(RedMagenta, Prop(1))), None));

    assert!(
        !Hue::Sextant(SextantHue(RedYellow, Prop(0x1000000000))).approx_eq(
            &Hue::Sextant(SextantHue(RedYellow, Prop(0xfffff0000))),
            None
        )
    );
    assert!(
        Hue::Sextant(SextantHue(RedYellow, Prop(0x1000000000))).approx_eq(
            &Hue::Sextant(SextantHue(RedYellow, Prop(0xfffff0000))),
            Some(Prop(0x010000))
        )
    );
}

#[test]
fn hue_from_rgb() {
    for rgb in RGB::<u64>::GREYS.iter() {
        assert!(Hue::try_from(rgb).is_err());
    }
    let fractions = [
        [1_u64, 10],
        [2, 10],
        [3, 10],
        [4, 10],
        [5, 10],
        [6, 10],
        [7, 10],
        [8, 10],
        [9, 10],
    ];
    let rgb_iter = RGB::<u64>::PRIMARIES
        .iter()
        .chain(RGB::<u64>::SECONDARIES.iter());
    let hue_iter = Hue::PRIMARIES.iter().chain(Hue::SECONDARIES.iter());
    for (rgb, hue) in rgb_iter.zip(hue_iter) {
        assert_eq!(Hue::try_from(rgb), Ok(*hue));
        for fraction in &fractions {
            assert_eq!(Hue::try_from(&(*rgb * Prop::from(*fraction))), Ok(*hue));
        }
    }
    for (rgb, hue) in RGB::<u64>::IN_BETWEENS.iter().zip(Hue::IN_BETWEENS.iter()) {
        assert_eq!(Hue::try_from(rgb), Ok(*hue));
        for fraction in &fractions {
            assert_approx_eq!(
                Hue::try_from(&(*rgb * Prop::from(*fraction))).unwrap(),
                *hue
            );
        }
    }
}

#[test]
fn hue_max_chroma_rgb() {
    for (hue, rgb) in Hue::PRIMARIES.iter().zip(RGB::<f64>::PRIMARIES.iter()) {
        assert_eq!(hue.max_chroma_rgb(), *rgb);
    }
    for (hue, rgb) in Hue::SECONDARIES.iter().zip(RGB::<f64>::SECONDARIES.iter()) {
        assert_eq!(hue.max_chroma_rgb(), *rgb);
    }
    for (hue, rgb) in Hue::IN_BETWEENS.iter().zip(RGB::<f64>::IN_BETWEENS.iter()) {
        assert_eq!(hue.max_chroma_rgb(), *rgb);
    }
}

#[test]
fn hue_to_from_angle() {
    for (angle, hue) in Angle::PRIMARIES.iter().zip(Hue::PRIMARIES.iter()) {
        assert_eq!(Hue::from(*angle), *hue);
        assert_eq!(hue.angle(), *angle);
    }
    for (angle, hue) in Angle::SECONDARIES.iter().zip(Hue::SECONDARIES.iter()) {
        assert_eq!(Hue::from(*angle), *hue);
        assert_eq!(hue.angle(), *angle);
    }
    for (angle, hue) in Angle::IN_BETWEENS.iter().zip(Hue::IN_BETWEENS.iter()) {
        assert_eq!(Hue::from(*angle), *hue);
        assert_eq!(hue.angle(), *angle);
    }
    let second = Prop::from(0.2679491924311227_f64);
    use Sextant::*;
    for (angle, sextant) in &[
        (Angle::from((15, 0, 0)), RedYellow),
        (Angle::from((105, 0, 0)), GreenYellow),
        (Angle::from((135, 0, 0)), GreenCyan),
        (-Angle::from((15, 0, 0)), RedMagenta),
        (-Angle::from((105, 0, 0)), BlueMagenta),
        (-Angle::from((135, 0, 0)), BlueCyan),
    ] {
        let hue = Hue::Sextant(SextantHue(*sextant, second));
        assert_approx_eq!(Hue::from(*angle), hue, Prop(0x0000000000010000));
        assert_approx_eq!(hue.angle(), *angle, 0x0000000000000100);
    }
}

#[test]
fn hue_add_sub_angle() {
    for hue in Hue::PRIMARIES
        .iter()
        .chain(Hue::SECONDARIES.iter())
        .chain(Hue::IN_BETWEENS.iter())
    {
        for angle in &[
            Angle::from(15),
            Angle::from(-15),
            Angle::from(135),
            Angle::from(-135),
        ] {
            assert_approx_eq!((*hue + *angle).angle(), hue.angle() + *angle, 0x100);
            assert_approx_eq!((*hue - *angle).angle(), hue.angle() - *angle, 0x100);
        }
    }
}

#[test]
fn rgb_ordered_triplet() {
    let light_levels: [Prop; 17] = [
        Prop::ZERO,
        Prop(1),
        Prop(2),
        Prop(3),
        Prop::ONE / 100,
        Prop::ONE / 10,
        Prop::ONE / 5,
        Prop::ONE / 3,
        Prop::ONE / 2,
        (Prop::ONE / 10 * 7).into(),
        (Prop::ONE / 10 * 8).into(),
        (Prop::ONE / 10 * 9).into(),
        (Prop::ONE / 100 * 99).into(),
        Prop::ONE - Prop(3),
        Prop::ONE - Prop(2),
        Prop::ONE - Prop(1),
        Prop::ONE,
    ];
    // Valid primary and secondary hue colours
    for first in &light_levels[1..] {
        for others in &light_levels {
            if others < first {
                for rgb_hue in &[RGBHue::Red, RGBHue::Green, RGBHue::Blue] {
                    let rgb = match rgb_hue {
                        RGBHue::Red => RGB::<u64>::from([*first, *others, *others]),
                        RGBHue::Green => RGB::<u64>::from([*others, *first, *others]),
                        RGBHue::Blue => RGB::<u64>::from([*others, *others, *first]),
                    };
                    let array = rgb_hue
                        .rgb_ordered_triplet(rgb.sum(), rgb.chroma_prop())
                        .expect("should be legal");
                    assert_eq!(RGB::<u64>::from(array), rgb);
                }
                for cmy_hue in &[CMYHue::Cyan, CMYHue::Magenta, CMYHue::Yellow] {
                    let rgb = match cmy_hue {
                        CMYHue::Cyan => RGB::<u64>::from([*others, *first, *first]),
                        CMYHue::Magenta => RGB::<u64>::from([*first, *others, *first]),
                        CMYHue::Yellow => RGB::<u64>::from([*first, *first, *others]),
                    };
                    let array = cmy_hue
                        .rgb_ordered_triplet(rgb.sum(), rgb.chroma_prop())
                        .expect("should be legal");
                    assert_eq!(RGB::<u64>::from(array), rgb);
                }
                let second = others;
                for third in &light_levels {
                    if third < second {
                        use Sextant::*;
                        for sextant in &[
                            RedYellow,
                            RedMagenta,
                            GreenCyan,
                            GreenYellow,
                            BlueMagenta,
                            BlueCyan,
                        ] {
                            let rgb = match sextant {
                                RedMagenta => RGB::<u64>::from([*first, *third, *second]),
                                RedYellow => RGB::<u64>::from([*first, *second, *third]),
                                GreenYellow => RGB::<u64>::from([*second, *first, *third]),
                                GreenCyan => RGB::<u64>::from([*third, *first, *second]),
                                BlueCyan => RGB::<u64>::from([*third, *second, *first]),
                                BlueMagenta => RGB::<u64>::from([*second, *third, *first]),
                            };
                            let hue = match sextant {
                                RedMagenta => Hue::try_from([*first, *third, *second]).unwrap(),
                                RedYellow => Hue::try_from([*first, *second, *third]).unwrap(),
                                GreenYellow => Hue::try_from([*second, *first, *third]).unwrap(),
                                GreenCyan => Hue::try_from([*third, *first, *second]).unwrap(),
                                BlueCyan => Hue::try_from([*third, *second, *first]).unwrap(),
                                BlueMagenta => Hue::try_from([*second, *third, *first]).unwrap(),
                            };
                            match hue {
                                Hue::Sextant(sextant_hue) => {
                                    let array = sextant_hue
                                        .rgb_ordered_triplet(rgb.sum(), rgb.chroma_prop())
                                        .expect("should be legal");
                                    assert_eq!(RGB::<u64>::from(array), rgb);
                                    let rgb = sextant_hue.max_chroma_rgb();
                                    // make sure we hit Chroma::Neither at least once
                                    let array = sextant_hue
                                        .rgb_ordered_triplet(rgb.sum(), rgb.chroma_prop())
                                        .expect("should be legal");
                                    assert_eq!(RGB::<u64>::from(array), rgb);
                                }
                                _ => panic!("should have been a SextantHue"),
                            }
                        }
                    }
                }
            }
        }
    }
    for red in light_levels.iter() {
        for green in light_levels.iter() {
            for blue in light_levels.iter() {
                let array = [*red, *green, *blue];
                if let Ok(hue) = Hue::try_from(array) {
                    let rgb = RGB::<u64>::from(array);
                    let sum = rgb.sum();
                    let chroma = rgb.chroma();
                    let array_out = hue.rgb_ordered_triplet(sum, chroma.into_prop()).unwrap();
                    assert_eq!(array, array_out);
                } else {
                    assert!(RGB::<u64>::from(array).is_grey());
                }
            }
        }
    }
}

#[test]
fn darkest_rgb_for_chroma_prop() {
    for rgb_hue in RGBHue::HUES.iter() {
        for c_prop in NON_ZERO_CHROMA_PROPS.iter().map(|v| Prop::from(*v)).rev() {
            let min_sum = rgb_hue.min_sum_for_chroma_prop(c_prop).unwrap();
            let array = rgb_hue.rgb_ordered_triplet(min_sum, c_prop).unwrap();
            let rgb = RGB::<u64>::from(array);
            assert_eq!(min_sum, rgb.sum());
            assert_eq!(c_prop, rgb.chroma_prop());
            let hue = Hue::try_from(rgb).unwrap();
            assert_eq!(hue, Hue::Primary(*rgb_hue));
        }
    }
    for cmy_hue in CMYHue::HUES.iter() {
        for c_prop in NON_ZERO_CHROMA_PROPS.iter().map(|v| Prop::from(*v)).rev() {
            let min_sum = cmy_hue.min_sum_for_chroma_prop(c_prop).unwrap();
            let array = cmy_hue.rgb_ordered_triplet(min_sum, c_prop).unwrap();
            let rgb = RGB::<u64>::from(array);
            assert_eq!(min_sum, rgb.sum());
            assert_eq!(c_prop, rgb.chroma_prop());
            let hue = Hue::try_from(rgb).unwrap();
            assert_eq!(hue, Hue::Secondary(*cmy_hue));
        }
    }
    for sextant in Sextant::SEXTANTS.iter() {
        for second in SECOND_VALUES.iter().map(|v| Prop::from(*v)) {
            let sextant_hue = SextantHue(*sextant, second);
            for c_prop in NON_ZERO_CHROMA_PROPS.iter().map(|v| Prop::from(*v)).rev() {
                let min_sum = sextant_hue.min_sum_for_chroma_prop(c_prop).unwrap();
                match sextant_hue
                    .try_rgb_ordered_triplet(min_sum, c_prop)
                    .unwrap()
                {
                    Ok(array) => {
                        let rgb = RGB::<u64>::from(array);
                        assert_eq!(min_sum, rgb.sum());
                        assert_eq!(c_prop, rgb.chroma_prop());
                        let hue = Hue::try_from(rgb).unwrap();
                        assert_eq!(hue, Hue::Sextant(sextant_hue));
                    }
                    Err(array) => {
                        let rgb = RGB::<u64>::from(array);
                        assert_eq!(min_sum, rgb.sum());
                        assert_eq!(c_prop, rgb.chroma_prop());
                        let hue = Hue::try_from(rgb).unwrap();
                        assert_approx_eq!(hue, Hue::Sextant(sextant_hue), Prop(0x10000));
                    }
                }
            }
        }
    }
}

#[test]
fn lightest_rgb_for_chroma_prop() {
    for rgb_hue in RGBHue::HUES.iter() {
        for c_prop in NON_ZERO_CHROMA_PROPS.iter().map(|v| Prop::from(*v)).rev() {
            let max_sum = rgb_hue.max_sum_for_chroma_prop(c_prop).unwrap();
            let array = rgb_hue.rgb_ordered_triplet(max_sum, c_prop).unwrap();
            let rgb = RGB::<u64>::from(array);
            assert_eq!(max_sum, rgb.sum());
            assert_eq!(c_prop, rgb.chroma_prop());
            let hue = Hue::try_from(rgb).unwrap();
            assert_eq!(hue, Hue::Primary(*rgb_hue));
        }
    }
    for cmy_hue in CMYHue::HUES.iter() {
        for c_prop in NON_ZERO_CHROMA_PROPS.iter().map(|v| Prop::from(*v)).rev() {
            let max_sum = cmy_hue.max_sum_for_chroma_prop(c_prop).unwrap();
            let array = cmy_hue.rgb_ordered_triplet(max_sum, c_prop).unwrap();
            let rgb = RGB::<u64>::from(array);
            assert_eq!(max_sum, rgb.sum());
            assert_eq!(c_prop, rgb.chroma_prop());
            let hue = Hue::try_from(rgb).unwrap();
            assert_eq!(hue, Hue::Secondary(*cmy_hue));
        }
    }
    for sextant in Sextant::SEXTANTS.iter() {
        for second in SECOND_VALUES.iter().map(|v| Prop::from(*v)) {
            let sextant_hue = SextantHue(*sextant, second);
            for c_prop in NON_ZERO_CHROMA_PROPS.iter().map(|v| Prop::from(*v)).rev() {
                let max_sum = sextant_hue.max_sum_for_chroma_prop(c_prop).unwrap();
                match sextant_hue
                    .try_rgb_ordered_triplet(max_sum, c_prop)
                    .unwrap()
                {
                    Ok(array) => {
                        let rgb = RGB::<u64>::from(array);
                        assert_eq!(max_sum, rgb.sum());
                        assert_eq!(c_prop, rgb.chroma_prop());
                        let hue = Hue::try_from(rgb).unwrap();
                        assert_eq!(hue, Hue::Sextant(sextant_hue));
                    }
                    Err(array) => {
                        let rgb = RGB::<u64>::from(array);
                        assert_eq!(max_sum, rgb.sum());
                        assert_eq!(c_prop, rgb.chroma_prop());
                        let hue = Hue::try_from(rgb).unwrap();
                        assert_approx_eq!(hue, Hue::Sextant(sextant_hue), Prop(0x10000));
                    }
                }
            }
        }
    }
}

// TODO: this test needs to be improved
#[test]
fn max_chroma_and_sum_ranges() {
    for hue in &Hue::PRIMARIES {
        assert!(hue.sum_range_for_chroma_prop(Prop::ZERO).is_none());
        assert_eq!(
            hue.sum_range_for_chroma_prop(Prop::ONE),
            Some((UFDRNumber::ONE, UFDRNumber::ONE))
        );
        for item in NON_ZERO_CHROMA_PROPS.iter() {
            let prop = Prop::from(*item);
            match prop {
                Prop::ONE => {
                    let range = hue.sum_range_for_chroma_prop(prop).unwrap();
                    assert_eq!(range.0, range.1);
                    let max_chroma = hue.max_chroma_for_sum(range.0).unwrap();
                    assert_eq!(max_chroma, Chroma::ONE);
                }
                prop => {
                    let range = hue.sum_range_for_chroma_prop(prop).unwrap();
                    let max_chroma = hue.max_chroma_for_sum(range.0).unwrap();
                    assert_approx_eq!(max_chroma, Chroma::Shade(prop), Prop(0xF));
                    let max_chroma = hue.max_chroma_for_sum(range.1).unwrap();
                    assert_approx_eq!(max_chroma, Chroma::Tint(prop), Prop(0xF));
                }
            }
        }
    }
    for hue in &Hue::SECONDARIES {
        assert!(hue.sum_range_for_chroma_prop(Prop::ZERO).is_none());
        assert_eq!(
            hue.sum_range_for_chroma_prop(Prop::ONE),
            Some((UFDRNumber::TWO, UFDRNumber::TWO))
        );
        for item in NON_ZERO_CHROMA_PROPS.iter() {
            let prop = Prop::from(*item);
            match prop {
                Prop::ONE => {
                    let range = hue.sum_range_for_chroma_prop(prop).unwrap();
                    assert_eq!(range.0, range.1);
                    let max_chroma = hue.max_chroma_for_sum(range.0).unwrap();
                    assert_eq!(max_chroma, Chroma::ONE);
                }
                prop => {
                    let range = hue.sum_range_for_chroma_prop(prop).unwrap();
                    let max_chroma = hue.max_chroma_for_sum(range.0).unwrap();
                    assert_approx_eq!(max_chroma, Chroma::Shade(prop), Prop(0xF));
                    let max_chroma = hue.max_chroma_for_sum(range.1).unwrap();
                    assert_approx_eq!(max_chroma, Chroma::Tint(prop), Prop(0xF));
                }
            }
        }
    }
    for sextant in Sextant::SEXTANTS.iter() {
        for item in SECOND_VALUES.iter() {
            let other = Prop::from(*item);
            let hue = Hue::Sextant(SextantHue(*sextant, other));
            assert!(hue.sum_range_for_chroma_prop(Prop::ZERO).is_none());
            assert_eq!(
                hue.sum_range_for_chroma_prop(Prop::ONE),
                Some((UFDRNumber::ONE + other, UFDRNumber::ONE + other,))
            );
        }
    }
}

#[test]
fn primary_max_chroma_rgbs() {
    for (hue, expected_rgb) in Hue::PRIMARIES.iter().zip(RGB::<f64>::PRIMARIES.iter()) {
        assert_eq!(
            hue.max_chroma_rgb_for_sum(UFDRNumber::ONE).unwrap(),
            *expected_rgb
        );
        assert!(hue
            .max_chroma_rgb_for_sum::<f64>(UFDRNumber::ZERO)
            .is_none());
        assert!(hue
            .max_chroma_rgb_for_sum::<f64>(UFDRNumber::THREE)
            .is_none());
        for sum in [
            UFDRNumber::from(0.0001_f64),
            UFDRNumber::from(0.25_f64),
            UFDRNumber::from(0.5_f64),
            UFDRNumber::from(0.75_f64),
            UFDRNumber::from(0.9999_f64),
        ]
        .iter()
        {
            let mut array = [Prop::ZERO, Prop::ZERO, Prop::ZERO];
            array[hue.indices().0 as usize] = (*sum).into();
            let expected: RGB<f64> = array.into();
            assert_eq!(hue.max_chroma_rgb_for_sum::<f64>(*sum).unwrap(), expected);
        }
        for sum in [
            UFDRNumber::from(2.0001_f64),
            UFDRNumber::from(2.25_f64),
            UFDRNumber::from(2.5_f64),
            UFDRNumber::from(2.75_f64),
            UFDRNumber::from(2.9999_f64),
        ]
        .iter()
        {
            let mut array = [Prop::ONE, Prop::ONE, Prop::ONE];
            array[hue.indices().1 as usize] = ((*sum - UFDRNumber::ONE) / 2).into();
            array[hue.indices().2 as usize] = ((*sum - UFDRNumber::ONE) / 2).into();
            let expected: RGB<f64> = array.into();
            assert_eq!(hue.max_chroma_rgb_for_sum::<f64>(*sum).unwrap(), expected);
        }
    }
}

#[test]
fn secondary_max_chroma_rgbs() {
    for (hue, expected_rgb) in Hue::SECONDARIES.iter().zip(RGB::<u64>::SECONDARIES.iter()) {
        assert_approx_eq!(
            hue.max_chroma_rgb_for_sum::<u64>(UFDRNumber::TWO).unwrap(),
            *expected_rgb
        );
        assert!(hue
            .max_chroma_rgb_for_sum::<f64>(UFDRNumber::ZERO)
            .is_none());
        assert!(hue
            .max_chroma_rgb_for_sum::<f64>(UFDRNumber::THREE)
            .is_none());
        for sum in [
            UFDRNumber::from(0.0001_f64),
            UFDRNumber::from(0.25_f64),
            UFDRNumber::from(0.5_f64),
            UFDRNumber::from(0.75_f64),
            UFDRNumber::ONE,
            UFDRNumber::from(1.5_f64),
            UFDRNumber::from(1.9999_f64),
        ]
        .iter()
        {
            let mut array = [Prop::ZERO, Prop::ZERO, Prop::ZERO];
            array[hue.indices().0 as usize] = (*sum / 2).into();
            array[hue.indices().1 as usize] = (*sum / 2).into();
            let expected: RGB<u64> = array.into();
            assert_eq!(hue.max_chroma_rgb_for_sum::<u64>(*sum).unwrap(), expected);
        }
        for sum in [
            UFDRNumber::from(2.0001_f64),
            UFDRNumber::from(2.25_f64),
            UFDRNumber::from(2.5_f64),
            UFDRNumber::from(2.75_f64),
            UFDRNumber::from(2.9999_f64),
        ]
        .iter()
        {
            let mut array = [Prop::ONE, Prop::ONE, Prop::ONE];
            array[hue.indices().2 as usize] = (*sum - UFDRNumber::TWO).into();
            let expected: RGB<u64> = array.into();
            assert_approx_eq!(hue.max_chroma_rgb_for_sum::<u64>(*sum).unwrap(), expected);
        }
    }
}

//#[test]
fn _other_max_chroma_rgbs() {
    for sextant in Sextant::SEXTANTS.iter() {
        for item in SECOND_VALUES.iter() {
            let second = Prop::from(*item);
            let sextant_hue = SextantHue(*sextant, second);
            let hue = Hue::Sextant(sextant_hue);
            assert!(hue
                .max_chroma_rgb_for_sum::<f64>(UFDRNumber::ZERO)
                .is_none());
            assert!(hue
                .max_chroma_rgb_for_sum::<f64>(UFDRNumber::THREE)
                .is_none());
            for item in VALID_HUE_SUMS.iter() {
                let sum = UFDRNumber::from(*item);
                let rgb = hue.max_chroma_rgb_for_sum::<u64>(sum).unwrap();
                assert_approx_eq!(sum, rgb.sum());
                assert_approx_eq!(rgb.chroma(), hue.max_chroma_for_sum(sum).unwrap());
                match Hue::try_from(&rgb).unwrap() {
                    Hue::Sextant(SextantHue(sextant_out, second_out)) => {
                        assert_eq!(sextant_hue.0, sextant_out);
                        assert_approx_eq!(sextant_hue.1, second_out, Prop(0x153));
                    }
                    _ => panic!("it's gone pure"),
                }
            }
        }
    }
}

#[test]
fn lightest_darkest_hcv_for_chroma() {
    let hues: Vec<Hue> = Hue::PRIMARIES
        .iter()
        .chain(Hue::SECONDARIES.iter())
        .chain(Hue::IN_BETWEENS.iter())
        .cloned()
        .collect();
    let hcvs: Vec<HCV> = HCV::PRIMARIES
        .iter()
        .chain(HCV::SECONDARIES.iter())
        .chain(HCV::IN_BETWEENS.iter())
        .cloned()
        .collect();
    for (hue, hcv) in hues.iter().zip(hcvs.iter()) {
        assert_eq!(hue.lightest_hcv_for_chroma(Chroma::ZERO), None);
        assert_eq!(hue.darkest_hcv_for_chroma(Chroma::ZERO), None);
        assert_eq!(hue.lightest_hcv_for_chroma(Chroma::ONE), Some(Ok(*hcv)));
        assert_eq!(hue.darkest_hcv_for_chroma(Chroma::ONE), Some(Ok(*hcv)));
        for prop in SHADE_TINT_CHROMA_PROPS.iter().map(|f| Prop::from(*f)) {
            let shade_chroma = Chroma::Shade(prop);
            let darkest_shade = match hue.darkest_hcv_for_chroma(shade_chroma).unwrap() {
                Ok(hcv) => {
                    assert_eq!(hcv.hue, Some(*hue));
                    assert_eq!(hcv.chroma(), shade_chroma);
                    hcv
                }
                Err(hcv) => {
                    assert_approx_eq!(hcv.hue.unwrap(), *hue);
                    assert_eq!(hcv.chroma(), shade_chroma);
                    hcv
                }
            };
            let lightest_shade = match hue.lightest_hcv_for_chroma(shade_chroma).unwrap() {
                Ok(hcv) => {
                    assert_eq!(hcv.hue, Some(*hue));
                    assert_eq!(hcv.chroma(), shade_chroma);
                    hcv
                }
                Err(hcv) => {
                    assert_approx_eq!(hcv.hue.unwrap(), *hue);
                    assert_eq!(hcv.chroma(), shade_chroma);
                    hcv
                }
            };
            assert!(darkest_shade.sum < lightest_shade.sum);
            let tint_chroma = Chroma::Tint(prop);
            let darkest_tint = match hue.darkest_hcv_for_chroma(tint_chroma).unwrap() {
                Ok(hcv) => {
                    assert_eq!(hcv.hue, Some(*hue));
                    assert_eq!(hcv.chroma(), tint_chroma);
                    hcv
                }
                Err(hcv) => {
                    assert_approx_eq!(hcv.hue.unwrap(), *hue);
                    assert_eq!(hcv.chroma(), tint_chroma);
                    hcv
                }
            };
            let lightest_tint = match hue.lightest_hcv_for_chroma(tint_chroma).unwrap() {
                Ok(hcv) => {
                    assert_eq!(hcv.hue, Some(*hue));
                    assert_eq!(hcv.chroma(), tint_chroma);
                    hcv
                }
                Err(hcv) => {
                    assert_approx_eq!(hcv.hue.unwrap(), *hue);
                    assert_eq!(hcv.chroma(), tint_chroma);
                    hcv
                }
            };
            assert!(darkest_tint.sum < lightest_tint.sum);
            assert!(lightest_shade.sum < darkest_tint.sum);
        }
    }
    for sextant in Sextant::SEXTANTS.iter() {
        for item in SECOND_VALUES.iter() {
            let second = Prop::from(*item);
            let hue = Hue::Sextant(SextantHue(*sextant, second));
            assert_eq!(hue.darkest_hcv_for_chroma(Chroma::ZERO), None);
            assert_eq!(hue.lightest_hcv_for_chroma(Chroma::ZERO), None);
            for prop in SHADE_TINT_CHROMA_PROPS.iter().map(|a| Prop::from(*a)) {
                let shade_chroma = Chroma::Shade(prop);
                let darkest_shade = match hue.darkest_hcv_for_chroma(shade_chroma).unwrap() {
                    Ok(hcv) => {
                        assert_eq!(hcv.hue, Some(hue));
                        assert_eq!(hcv.chroma(), shade_chroma);
                        hcv
                    }
                    Err(hcv) => {
                        assert_approx_eq!(hcv.hue.unwrap(), hue, Prop(0xF0000));
                        assert_eq!(hcv.chroma(), shade_chroma);
                        hcv
                    }
                };
                let lightest_shade = match hue.lightest_hcv_for_chroma(shade_chroma).unwrap() {
                    Ok(hcv) => {
                        assert_eq!(hcv.hue, Some(hue));
                        assert_eq!(hcv.chroma(), shade_chroma);
                        hcv
                    }
                    Err(hcv) => {
                        assert_approx_eq!(hcv.hue.unwrap(), hue, Prop(0xF0000));
                        assert_eq!(hcv.chroma_prop(), shade_chroma.into_prop());
                        hcv
                    }
                };
                assert!(darkest_shade.sum < lightest_shade.sum);
                let tint_chroma = Chroma::Tint(prop);
                let darkest_tint = match hue.darkest_hcv_for_chroma(tint_chroma).unwrap() {
                    Ok(hcv) => {
                        assert_eq!(hcv.hue, Some(hue));
                        assert_eq!(hcv.chroma(), tint_chroma);
                        hcv
                    }
                    Err(hcv) => {
                        assert_approx_eq!(hcv.hue.unwrap(), hue, Prop(0xF0000));
                        assert_eq!(hcv.chroma_prop(), tint_chroma.into_prop());
                        hcv
                    }
                };
                let lightest_tint = match hue.lightest_hcv_for_chroma(tint_chroma).unwrap() {
                    Ok(hcv) => {
                        assert_eq!(hcv.hue, Some(hue));
                        assert_eq!(hcv.chroma(), tint_chroma);
                        hcv
                    }
                    Err(hcv) => {
                        assert_approx_eq!(hcv.hue.unwrap(), hue, Prop(0xF0000));
                        assert_eq!(hcv.chroma(), tint_chroma);
                        hcv
                    }
                };
                assert!(darkest_tint.sum < lightest_tint.sum);
                assert!(lightest_shade.sum < darkest_tint.sum);
            }
        }
    }
}

#[test]
fn lightest_darkest_rgb_for_chroma() {
    let hues: Vec<Hue> = Hue::PRIMARIES
        .iter()
        .chain(Hue::SECONDARIES.iter())
        .chain(Hue::IN_BETWEENS.iter())
        .cloned()
        .collect();
    let hcvs: Vec<HCV> = HCV::PRIMARIES
        .iter()
        .chain(HCV::SECONDARIES.iter())
        .chain(HCV::IN_BETWEENS.iter())
        .cloned()
        .collect();
    for (hue, hcv) in hues.iter().zip(hcvs.iter()) {
        assert_eq!(hue.darkest_rgb_for_chroma::<u64>(Chroma::ZERO), None);
        assert_eq!(hue.lightest_rgb_for_chroma::<u64>(Chroma::ZERO), None);
        assert_eq!(
            hue.darkest_rgb_for_chroma::<u64>(Chroma::ONE),
            Some(Ok(RGB::<u64>::from(hcv)))
        );
        assert_eq!(
            hue.lightest_rgb_for_chroma::<u64>(Chroma::ONE),
            Some(Ok(RGB::<u64>::from(hcv)))
        );
        for prop in SHADE_TINT_CHROMA_PROPS.iter().map(|f| Prop::from(*f)) {
            let shade_chroma = Chroma::Shade(prop);
            let darkest_shade = match hue.darkest_rgb_for_chroma::<u64>(shade_chroma).unwrap() {
                Ok(darkest_shade) => {
                    assert_eq!(darkest_shade.hue(), Some(*hue));
                    darkest_shade
                }
                Err(darkest_shade) => {
                    assert_approx_eq!(darkest_shade.hue().unwrap(), *hue);
                    darkest_shade
                }
            };
            assert_eq!(darkest_shade.chroma(), shade_chroma);
            let lightest_shade = match hue.lightest_rgb_for_chroma::<u64>(shade_chroma).unwrap() {
                Ok(lightest_shade) => {
                    assert_eq!(lightest_shade.hue(), Some(*hue));
                    lightest_shade
                }
                Err(lightest_shade) => {
                    assert_approx_eq!(lightest_shade.hue().unwrap(), *hue);
                    lightest_shade
                }
            };
            assert_eq!(lightest_shade.chroma(), shade_chroma);
            assert!(darkest_shade.sum() < lightest_shade.sum());
            let tint_chroma = Chroma::Tint(prop);
            let darkest_tint = match hue.darkest_rgb_for_chroma::<u64>(tint_chroma).unwrap() {
                Ok(darkest_tint) => {
                    assert_eq!(darkest_tint.hue(), Some(*hue));
                    darkest_tint
                }
                Err(darkest_tint) => {
                    assert_approx_eq!(darkest_tint.hue().unwrap(), *hue);
                    darkest_tint
                }
            };
            assert_eq!(darkest_tint.chroma(), tint_chroma);
            let lightest_tint = match hue.lightest_rgb_for_chroma::<u64>(tint_chroma).unwrap() {
                Ok(lightest_tint) => {
                    assert_eq!(lightest_tint.hue(), Some(*hue));
                    lightest_tint
                }
                Err(lightest_tint) => {
                    assert_approx_eq!(lightest_tint.hue().unwrap(), *hue);
                    lightest_tint
                }
            };
            assert_eq!(lightest_tint.chroma(), tint_chroma);
            assert!(darkest_tint.sum() < lightest_tint.sum());
            assert!(lightest_shade.sum() < darkest_tint.sum());
        }
    }
    for sextant in Sextant::SEXTANTS.iter() {
        for item in SECOND_VALUES.iter() {
            let second = Prop::from(*item);
            let hue = Hue::Sextant(SextantHue(*sextant, second));
            assert_eq!(hue.darkest_rgb_for_chroma::<u64>(Chroma::ZERO), None);
            assert_eq!(hue.lightest_rgb_for_chroma::<u64>(Chroma::ZERO), None);
            assert_eq!(
                hue.darkest_rgb_for_chroma::<u64>(Chroma::ONE),
                Some(Ok(hue.max_chroma_rgb::<u64>()))
            );
            assert_eq!(
                hue.lightest_rgb_for_chroma::<u64>(Chroma::ONE),
                Some(Ok(hue.max_chroma_rgb::<u64>()))
            );
            for prop in SHADE_TINT_CHROMA_PROPS.iter().map(|f| Prop::from(*f)) {
                let shade_chroma = Chroma::Shade(prop);
                let darkest_shade = match hue.darkest_rgb_for_chroma::<u64>(shade_chroma).unwrap() {
                    Ok(darkest_shade) => {
                        assert_eq!(darkest_shade.hue(), Some(hue));
                        darkest_shade
                    }
                    Err(darkest_shade) => {
                        assert_approx_eq!(darkest_shade.hue().unwrap(), hue, Prop(0xF0000));
                        darkest_shade
                    }
                };
                assert_eq!(darkest_shade.chroma(), shade_chroma);
                let lightest_shade = match hue.lightest_rgb_for_chroma::<u64>(shade_chroma).unwrap()
                {
                    Ok(lightest_shade) => {
                        assert_eq!(lightest_shade.hue(), Some(hue));
                        assert_eq!(lightest_shade.chroma(), shade_chroma);
                        lightest_shade
                    }
                    Err(lightest_shade) => {
                        assert_approx_eq!(lightest_shade.hue().unwrap(), hue, Prop(0xF0000));
                        assert_eq!(lightest_shade.chroma_prop(), shade_chroma.into_prop());
                        lightest_shade
                    }
                };
                assert!(darkest_shade.sum() < lightest_shade.sum());
                let tint_chroma = Chroma::Tint(prop);
                let darkest_tint = match hue.darkest_rgb_for_chroma::<u64>(tint_chroma).unwrap() {
                    Ok(darkest_tint) => {
                        assert_eq!(darkest_tint.hue(), Some(hue));
                        assert_eq!(darkest_tint.chroma(), tint_chroma);
                        darkest_tint
                    }
                    Err(darkest_tint) => {
                        assert_approx_eq!(darkest_tint.hue().unwrap(), hue, Prop(0xF0000));
                        assert_eq!(darkest_tint.chroma_prop(), tint_chroma.into_prop());
                        darkest_tint
                    }
                };
                let lightest_tint = match hue.lightest_rgb_for_chroma::<u64>(tint_chroma).unwrap() {
                    Ok(lightest_tint) => {
                        assert_eq!(lightest_tint.hue(), Some(hue));
                        lightest_tint
                    }
                    Err(lightest_tint) => {
                        assert_approx_eq!(lightest_tint.hue().unwrap(), hue, Prop(0xF0000));
                        lightest_tint
                    }
                };
                assert_eq!(lightest_tint.chroma(), tint_chroma);
                assert!(darkest_tint.sum() < lightest_tint.sum());
                assert!(lightest_shade.sum() < darkest_tint.sum());
            }
        }
    }
}

// //#[test]
// fn _min_max_rgb_for_chroma() {
//     for (hue, expected_rgb) in Hue::PRIMARIES.iter().zip(RGB::<f64>::PRIMARIES.iter()) {
//         assert_eq!(
//             hue.darkest_rgb_for_chroma::<f64>(Chroma::ONE),
//             Some(*expected_rgb)
//         );
//         assert_eq!(
//             hue.lightest_rgb_for_chroma::<f64>(Chroma::ONE),
//             Some(*expected_rgb)
//         );
//         let prop = Prop::from(0.5_f64);
//         let shade_chroma = Chroma::Shade(prop);
//         let shade = hue.darkest_rgb_for_chroma::<u64>(shade_chroma).unwrap();
//         let tint_chroma = Chroma::Tint(prop);
//         let tint = hue.lightest_rgb_for_chroma::<u64>(tint_chroma).unwrap();
//         assert!(shade.value() < tint.value());
//         assert_approx_eq!(shade.chroma(), shade_chroma, Prop(0xF));
//         assert_approx_eq!(tint.chroma(), tint_chroma, Prop(0xF));
//         assert_approx_eq!(
//             shade.max_chroma_rgb(),
//             tint.max_chroma_rgb(),
//             0.0000001.into()
//         );
//     }
//     for (hue, expected_rgb) in Hue::SECONDARIES.iter().zip(RGB::<u64>::SECONDARIES.iter()) {
//         assert_eq!(
//             hue.darkest_rgb_for_chroma::<u64>(Chroma::ONE),
//             Some(*expected_rgb)
//         );
//         assert_eq!(
//             hue.lightest_rgb_for_chroma::<u64>(Chroma::ONE),
//             Some(*expected_rgb)
//         );
//         let prop = Prop::from(0.5_f64);
//         let shade_chroma = Chroma::Shade(prop);
//         let shade = hue.darkest_rgb_for_chroma::<u64>(shade_chroma).unwrap();
//         let tint_chroma = Chroma::Tint(prop);
//         let tint = hue.lightest_rgb_for_chroma::<u64>(tint_chroma).unwrap();
//         assert!(shade.value() < tint.value());
//         assert_approx_eq!(shade.chroma(), shade_chroma, Prop(0xF));
//         assert_approx_eq!(tint.chroma(), tint_chroma, Prop(0xF));
//         assert_approx_eq!(
//             shade.max_chroma_rgb(),
//             tint.max_chroma_rgb(),
//             0.0000001.into()
//         );
//     }
//     for sextant in Sextant::SEXTANTS.iter() {
//         for item in SECOND_VALUES.iter() {
//             let second = Prop::from(*item);
//             let hue = Hue::Sextant(SextantHue(*sextant, second));
//             assert_eq!(hue.darkest_rgb_for_chroma::<u64>(Chroma::ZERO), None);
//             assert_eq!(hue.lightest_rgb_for_chroma::<u64>(Chroma::ZERO), None);
//             for prop in _SHADE_TINT_CHROMA_PROPS.iter().map(|a| Prop::from(*a)) {
//                 let shade_chroma = Chroma::Shade(prop);
//                 let shade = hue.darkest_rgb_for_chroma::<u64>(shade_chroma).unwrap();
//                 let tint_chroma = Chroma::Tint(prop);
//                 let tint = hue.lightest_rgb_for_chroma::<u64>(tint_chroma).unwrap();
//                 assert!(shade.value() < tint.value());
//                 assert_approx_eq!(shade.chroma(), shade_chroma, Prop(0xF));
//                 assert_approx_eq!(tint.chroma(), tint_chroma, Prop(0xF));
//                 assert_approx_eq!(
//                     shade.max_chroma_rgb(),
//                     tint.max_chroma_rgb(),
//                     0.000_001.into()
//                 );
//             }
//         }
//     }
// }

#[test]
fn try_rgb_for_sum_and_chroma_prop() {
    for hue in Hue::PRIMARIES
        .iter()
        .chain(Hue::SECONDARIES.iter())
        .chain(Hue::IN_BETWEENS.iter())
    {
        assert!(hue
            .try_rgb_for_sum_and_chroma_prop::<u64>(UFDRNumber::ZERO, Prop::ONE)
            .is_none());
        assert!(hue
            .try_rgb_for_sum_and_chroma_prop::<u64>(UFDRNumber::THREE, Prop::ONE)
            .is_none());
        assert!(hue
            .try_rgb_for_sum_and_chroma_prop::<u64>(UFDRNumber::ZERO, Prop::ZERO)
            .is_none());
        assert!(hue
            .try_rgb_for_sum_and_chroma_prop::<u64>(UFDRNumber::THREE, Prop::ZERO)
            .is_none());
        for c_prop in NON_ZERO_CHROMA_PROPS.iter().map(|item| Prop::from(*item)) {
            for sum in VALID_HUE_SUMS.iter().map(|item| UFDRNumber::from(*item)) {
                if let Some(result) = hue.try_rgb_for_sum_and_chroma_prop::<u64>(sum, c_prop) {
                    match result {
                        Ok(rgb) => {
                            assert_eq!(rgb.hue(), Some(*hue));
                            assert_eq!(rgb.chroma_prop(), c_prop);
                            assert_eq!(rgb.sum(), sum);
                            assert_eq!(rgb, RGB::from(HCV::from(rgb)));
                        }
                        Err(rgb) => {
                            assert_approx_eq!(rgb.hue().unwrap(), *hue);
                            assert_eq!(rgb.chroma_prop(), c_prop);
                            assert_eq!(rgb.sum(), sum);
                            assert_eq!(rgb, RGB::from(HCV::from(rgb)));
                        }
                    }
                } else {
                    if let Some(range) = hue.sum_range_for_chroma_prop(c_prop) {
                        assert!(sum < range.0 || sum > range.1);
                    } else {
                        assert!(c_prop == Prop::ZERO || !sum.is_hue_valid());
                    }
                }
            }
        }
    }
    // Now give sextant hues a good work over
    for sextant in Sextant::SEXTANTS.iter() {
        for hue in SECOND_VALUES
            .iter()
            .map(|i| Hue::Sextant(SextantHue(*sextant, Prop::from(*i))))
        {
            for c_prop in NON_ZERO_CHROMA_PROPS.iter().map(|item| Prop::from(*item)) {
                for sum in VALID_HUE_SUMS.iter().map(|item| UFDRNumber::from(*item)) {
                    if let Some(result) = hue.try_rgb_for_sum_and_chroma_prop::<u64>(sum, c_prop) {
                        match result {
                            Ok(rgb) => {
                                assert_eq!(rgb.hue(), Some(hue));
                                assert_eq!(rgb.chroma_prop(), c_prop);
                                assert_eq!(rgb.sum(), sum);
                                assert_eq!(rgb, RGB::from(HCV::from(rgb)));
                            }
                            Err(rgb) => {
                                assert_approx_eq!(rgb.hue().unwrap(), hue, Prop(0x100000));
                                assert_eq!(rgb.chroma_prop(), c_prop);
                                assert_eq!(rgb.sum(), sum);
                                assert_eq!(rgb, RGB::from(HCV::from(rgb)));
                            }
                        }
                    } else {
                        if let Some(range) = hue.sum_range_for_chroma_prop(c_prop) {
                            assert!(sum < range.0 || sum > range.1);
                        } else {
                            assert!(c_prop == Prop::ZERO || !sum.is_hue_valid());
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn try_hcv_for_sum_and_chroma_prop() {
    for hue in Hue::PRIMARIES
        .iter()
        .chain(Hue::SECONDARIES.iter())
        .chain(Hue::IN_BETWEENS.iter())
    {
        assert!(hue
            .try_hcv_for_sum_and_chroma_prop(UFDRNumber::ZERO, Prop::ONE)
            .is_none());
        assert!(hue
            .try_hcv_for_sum_and_chroma_prop(UFDRNumber::THREE, Prop::ONE)
            .is_none());
        assert!(hue
            .try_hcv_for_sum_and_chroma_prop(UFDRNumber::ZERO, Prop::ZERO)
            .is_none());
        assert!(hue
            .try_hcv_for_sum_and_chroma_prop(UFDRNumber::THREE, Prop::ZERO)
            .is_none());
        for c_prop in NON_ZERO_CHROMA_PROPS.iter().map(|item| Prop::from(*item)) {
            for sum in VALID_HUE_SUMS.iter().map(|item| UFDRNumber::from(*item)) {
                if let Some(result) = hue.try_hcv_for_sum_and_chroma_prop(sum, c_prop) {
                    match result {
                        Ok(hcv) => {
                            assert_eq!(hcv.hue(), Some(*hue));
                            assert_eq!(hcv.chroma_prop(), c_prop);
                            assert_eq!(hcv.sum, sum);
                            assert_eq!(hcv, HCV::from(RGB::<u64>::from(hcv)));
                        }
                        Err(hcv) => {
                            assert_approx_eq!(hcv.hue().unwrap(), *hue);
                            assert_eq!(hcv.chroma_prop(), c_prop);
                            assert_eq!(hcv.sum, sum);
                            assert_eq!(hcv, HCV::from(RGB::<u64>::from(hcv)));
                        }
                    }
                } else {
                    if let Some(range) = hue.sum_range_for_chroma_prop(c_prop) {
                        assert!(sum < range.0 || sum > range.1);
                    } else {
                        assert!(c_prop == Prop::ZERO || !sum.is_hue_valid());
                    }
                }
            }
        }
    }
    // Now give sextant hues a good work over
    for sextant in Sextant::SEXTANTS.iter() {
        for hue in SECOND_VALUES
            .iter()
            .map(|i| Hue::Sextant(SextantHue(*sextant, Prop::from(*i))))
        {
            for c_prop in NON_ZERO_CHROMA_PROPS.iter().map(|item| Prop::from(*item)) {
                for sum in VALID_HUE_SUMS.iter().map(|item| UFDRNumber::from(*item)) {
                    if let Some(result) = hue.try_hcv_for_sum_and_chroma_prop(sum, c_prop) {
                        match result {
                            Ok(hcv) => {
                                assert_eq!(hcv.hue(), Some(hue));
                                assert_eq!(hcv.chroma_prop(), c_prop);
                                assert_eq!(hcv.sum, sum);
                                assert_eq!(hcv, HCV::from(RGB::<u64>::from(hcv)));
                            }
                            Err(hcv) => {
                                assert_approx_eq!(hcv.hue().unwrap(), hue, Prop(0x100000));
                                assert_eq!(hcv.chroma_prop(), c_prop);
                                assert_eq!(hcv.sum, sum);
                                assert_eq!(hcv, HCV::from(RGB::<u64>::from(hcv)));
                            }
                        }
                    } else {
                        if let Some(range) = hue.sum_range_for_chroma_prop(c_prop) {
                            assert!(sum < range.0 || sum > range.1);
                        } else {
                            assert!(c_prop == Prop::ZERO || !sum.is_hue_valid());
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn primary_try_rgb_for_sum_and_chroma() {
    for hue in &Hue::PRIMARIES {
        assert!(hue
            .try_rgb_for_sum_and_chroma::<u64>(UFDRNumber::ZERO, Chroma::ONE)
            .is_none());
        assert!(hue
            .try_rgb_for_sum_and_chroma::<u64>(UFDRNumber::THREE, Chroma::ONE)
            .is_none());
        assert!(hue
            .try_rgb_for_sum_and_chroma::<u64>(UFDRNumber::ZERO, Chroma::ZERO)
            .is_none());
        assert!(hue
            .try_rgb_for_sum_and_chroma::<u64>(UFDRNumber::THREE, Chroma::ZERO)
            .is_none());
        for prop in NON_ZERO_CHROMA_PROPS.iter().map(|item| Prop::from(*item)) {
            for sum in VALID_HUE_SUMS.iter().map(|item| UFDRNumber::from(*item)) {
                let chroma = Chroma::from((prop, *hue, sum));
                if let Some(result) = hue.try_rgb_for_sum_and_chroma::<u64>(sum, chroma) {
                    match result {
                        Ok(rgb) => {
                            assert_eq!(rgb.hue(), Some(*hue));
                            assert_eq!(rgb.sum(), sum);
                            assert_eq!(rgb.chroma(), chroma);
                            assert_eq!(rgb, RGB::from(HCV::from(rgb)));
                        }
                        Err(rgb) => {
                            assert_approx_eq!(rgb.hue().unwrap(), *hue);
                            assert_eq!(rgb.sum(), sum);
                            // NB: near the swap over point sum errors can cause a shift in Chroma variant
                            if sum.approx_eq(&hue.sum_for_max_chroma(), Some(Prop(0x100))) {
                                assert_eq!(rgb.chroma_prop(), chroma.into_prop());
                            } else {
                                assert_eq!(rgb.chroma(), chroma);
                            }
                            assert_eq!(rgb, RGB::from(HCV::from(rgb)));
                        }
                    }
                } else {
                    let range = hue.sum_range_for_chroma(chroma).unwrap();
                    assert!(sum < range.0 || sum > range.1);
                }
            }
        }
    }
}

#[test]
fn secondary_try_rgb_for_sum_and_chroma() {
    for hue in &Hue::SECONDARIES {
        assert!(hue
            .try_rgb_for_sum_and_chroma::<u64>(UFDRNumber::ZERO, Chroma::ONE)
            .is_none());
        assert!(hue
            .try_rgb_for_sum_and_chroma::<u64>(UFDRNumber::THREE, Chroma::ONE)
            .is_none());
        assert!(hue
            .try_rgb_for_sum_and_chroma::<u64>(UFDRNumber::ZERO, Chroma::ZERO)
            .is_none());
        assert!(hue
            .try_rgb_for_sum_and_chroma::<u64>(UFDRNumber::THREE, Chroma::ZERO)
            .is_none());
        for prop in NON_ZERO_CHROMA_PROPS.iter().map(|item| Prop::from(*item)) {
            for sum in VALID_HUE_SUMS.iter().map(|item| UFDRNumber::from(*item)) {
                let chroma = Chroma::from((prop, *hue, sum));
                if let Some(result) = hue.try_rgb_for_sum_and_chroma::<u64>(sum, chroma) {
                    match result {
                        Ok(rgb) => {
                            assert_eq!(rgb.hue(), Some(*hue));
                            assert_eq!(rgb.sum(), sum);
                            assert_eq!(rgb.chroma(), chroma);
                            assert_eq!(rgb, RGB::from(HCV::from(rgb)));
                        }
                        Err(rgb) => {
                            assert_approx_eq!(rgb.hue().unwrap(), *hue);
                            assert_eq!(rgb.sum(), sum);
                            // NB: near the swap over point sum errors can cause a shift in Chroma variant
                            if sum.approx_eq(&hue.sum_for_max_chroma(), Some(Prop(0x100))) {
                                assert_eq!(rgb.chroma_prop(), chroma.into_prop());
                            } else {
                                assert_eq!(rgb.chroma(), chroma);
                            }
                            assert_eq!(rgb, RGB::from(HCV::from(rgb)));
                        }
                    }
                } else {
                    let range = hue.sum_range_for_chroma(chroma).unwrap();
                    assert!(sum < range.0 || sum > range.1);
                }
            }
        }
    }
}

#[test]
fn general_try_rgb_for_sum_and_chroma() {
    for sextant in Sextant::SEXTANTS.iter() {
        for second in SECOND_VALUES.iter().map(|a| Prop::from(*a)) {
            let sextant_hue = SextantHue(*sextant, second);
            let hue = Hue::Sextant(sextant_hue);
            assert!(hue
                .try_rgb_for_sum_and_chroma::<u64>(UFDRNumber::ZERO, Chroma::ONE)
                .is_none());
            assert!(hue
                .try_rgb_for_sum_and_chroma::<u64>(UFDRNumber::THREE, Chroma::ONE)
                .is_none());
            assert!(hue
                .try_rgb_for_sum_and_chroma::<u64>(UFDRNumber::ZERO, Chroma::ZERO)
                .is_none());
            assert!(hue
                .try_rgb_for_sum_and_chroma::<u64>(UFDRNumber::THREE, Chroma::ZERO)
                .is_none());
            for prop in NON_ZERO_CHROMA_PROPS.iter().map(|a| Prop::from(*a)) {
                for sum in VALID_HUE_SUMS.iter().map(|a| UFDRNumber::from(*a)) {
                    let chroma = Chroma::from((prop, hue, sum));
                    if let Some(result) = hue.try_rgb_for_sum_and_chroma::<u64>(sum, chroma) {
                        match result {
                            Ok(rgb) => {
                                assert_eq!(rgb.hue(), Some(hue));
                                assert_eq!(rgb.sum(), sum);
                                assert_eq!(rgb.chroma(), chroma);
                                assert_eq!(rgb, RGB::from(HCV::from(rgb)));
                            }
                            Err(rgb) => {
                                assert_approx_eq!(rgb.hue().unwrap(), hue, Prop(0x100000));
                                assert_eq!(rgb.sum(), sum);
                                // NB: near the swap over point sum errors can cause a shift in Chroma variant
                                if sum.approx_eq(&hue.sum_for_max_chroma(), Some(Prop(0x100))) {
                                    assert_eq!(rgb.chroma_prop(), chroma.into_prop());
                                } else {
                                    assert_eq!(rgb.chroma(), chroma);
                                }
                                assert_eq!(rgb, RGB::from(HCV::from(rgb)));
                            }
                        }
                    } else {
                        let range = hue.sum_range_for_chroma(chroma).unwrap();
                        assert!(sum < range.0 || sum > range.1);
                    }
                }
            }
        }
    }
}
