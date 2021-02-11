// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use super::*;
use num_traits_plus::{assert_approx_eq, float_plus::FloatApproxEq};

use crate::{proportion::*, rgb::RGB, CCI};

const NON_ZERO_CHROMAS: [f64; 7] = [0.01, 0.025, 0.5, 0.75, 0.9, 0.99, 1.0];
const VALID_OTHER_SUMS: [f64; 20] = [
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
];
// "second" should never be 0.0 or 1.0
const SECOND_VALUES: [f64; 11] = [0.001, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 0.99];

impl RGBHue {
    fn indices(&self) -> (CCI, CCI, CCI) {
        match self {
            RGBHue::Red => (CCI::Red, CCI::Green, CCI::Blue),
            RGBHue::Green => (CCI::Green, CCI::Red, CCI::Blue),
            RGBHue::Blue => (CCI::Blue, CCI::Red, CCI::Green),
        }
    }
}

impl CMYHue {
    fn indices(&self) -> (CCI, CCI, CCI) {
        match self {
            CMYHue::Magenta => (CCI::Red, CCI::Blue, CCI::Green),
            CMYHue::Yellow => (CCI::Red, CCI::Green, CCI::Blue),
            CMYHue::Cyan => (CCI::Green, CCI::Blue, CCI::Red),
        }
    }
}

impl Sextant {
    fn indices(&self) -> (CCI, CCI, CCI) {
        match self {
            Sextant::RedYellow => (CCI::Red, CCI::Green, CCI::Blue),
            Sextant::RedMagenta => (CCI::Red, CCI::Blue, CCI::Green),
            Sextant::GreenYellow => (CCI::Green, CCI::Red, CCI::Blue),
            Sextant::GreenCyan => (CCI::Green, CCI::Blue, CCI::Red),
            Sextant::BlueMagenta => (CCI::Blue, CCI::Red, CCI::Green),
            Sextant::BlueCyan => (CCI::Blue, CCI::Green, CCI::Red),
        }
    }
}

impl SextantHue {
    fn indices(&self) -> (CCI, CCI, CCI) {
        self.0.indices()
    }
}

impl Hue {
    fn indices(&self) -> (CCI, CCI, CCI) {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.indices(),
            Self::Secondary(cmy_hue) => cmy_hue.indices(),
            Self::Sextant(sextant_hue) => sextant_hue.indices(),
        }
    }
}

#[test]
fn hue_from_rgb() {
    for rgb in &[
        RGB::<f64>::BLACK,
        RGB::WHITE,
        RGB::from([0.5_f64, 0.5_f64, 0.5_f64]),
    ] {
        assert!(Hue::try_from(rgb).is_err());
    }
    for (rgb, hue) in RGB::<f64>::PRIMARIES.iter().zip(Hue::PRIMARIES.iter()) {
        assert_eq!(Hue::try_from(rgb), Ok(*hue));
        assert_eq!(Hue::try_from(&(*rgb * Prop::from(0.5))), Ok(*hue));
    }
    for (rgb, hue) in RGB::<f64>::SECONDARIES.iter().zip(Hue::SECONDARIES.iter()) {
        assert_eq!(Hue::try_from(rgb), Ok(*hue));
        assert_eq!(Hue::try_from(&(*rgb * Prop::from(0.5))), Ok(*hue));
    }
    for (array, sextant, second) in &[
        (
            [Prop::ONE, Prop::from(0.5_f64), Prop::ZERO],
            Sextant::RedYellow,
            Prop::from(0.5),
        ),
        (
            [Prop::ZERO, Prop::from(0.25_f64), Prop::from(0.5_f64)],
            Sextant::BlueCyan,
            Prop::from(0.5),
        ),
        (
            [Prop::from(0.2_f64), Prop::ZERO, Prop::from(0.4_f64)],
            Sextant::BlueMagenta,
            Prop::from(0.5),
        ),
        (
            [Prop::from(0.5_f64), Prop::ZERO, Prop::ONE],
            Sextant::BlueMagenta,
            Prop::from(0.5),
        ),
        (
            [Prop::ONE, Prop::ZERO, Prop::from(0.5_f64)],
            Sextant::RedMagenta,
            Prop::from(0.5),
        ),
        (
            [Prop::from(0.5_f64), Prop::ONE, Prop::ZERO],
            Sextant::GreenYellow,
            Prop::from(0.5),
        ),
        (
            [Prop::ZERO, Prop::ONE, Prop::from(0.5_f64)],
            Sextant::GreenCyan,
            Prop::from(0.5),
        ),
    ] {
        let rgb = RGB::<f64>::from([
            Prop::from(array[0]),
            Prop::from(array[1]),
            Prop::from(array[2]),
        ]);
        let hue = Hue::Sextant(SextantHue(*sextant, *second));
        assert_approx_eq!(Hue::try_from(&rgb).unwrap(), hue, 0.000_000_001);
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
    for (array, sextant, second) in &[
        (
            [Prop::ONE, Prop::from(0.5_f64), Prop::ZERO],
            Sextant::RedYellow,
            Prop::from(0.5_f64),
        ),
        (
            [Prop::ZERO, Prop::from(0.5_f64), Prop::ONE],
            Sextant::BlueCyan,
            Prop::from(0.5_f64),
        ),
        (
            [Prop::from(0.5_f64), Prop::ZERO, Prop::ONE],
            Sextant::BlueMagenta,
            Prop::from(0.5_f64),
        ),
        (
            [Prop::ONE, Prop::ZERO, Prop::from(0.5_f64)],
            Sextant::RedMagenta,
            Prop::from(0.5_f64),
        ),
        (
            [Prop::from(0.5_f64), Prop::ONE, Prop::ZERO],
            Sextant::GreenYellow,
            Prop::from(0.5_f64),
        ),
        (
            [Prop::ZERO, Prop::ONE, Prop::from(0.5_f64)],
            Sextant::GreenCyan,
            Prop::from(0.5_f64),
        ),
    ] {
        let rgb = RGB::<f64>::from(*array);
        let hue = Hue::Sextant(SextantHue(*sextant, *second));
        println!("{:?} {:?} {:?} {:?}", rgb, sextant, second, hue);
        assert_eq!(Hue::try_from(&rgb), Ok(hue));
    }
}

#[test]
fn hue_angle() {
    for (hue, angle) in Hue::PRIMARIES.iter().zip(Degrees::<f64>::PRIMARIES.iter()) {
        assert_eq!(hue.hue_angle(), *angle);
    }
    for (hue, angle) in Hue::SECONDARIES
        .iter()
        .zip(Degrees::<f64>::SECONDARIES.iter())
    {
        assert_eq!(hue.hue_angle(), *angle);
    }
    for (sextant, second, angle) in &[
        (
            Sextant::RedYellow,
            Prop::from(0.5_f64),
            Degrees::<f64>::DEG_30,
        ),
        (
            Sextant::BlueCyan,
            Prop::from(0.5_f64),
            Degrees::<f64>::NEG_DEG_150,
        ),
        (
            Sextant::BlueMagenta,
            Prop::from(0.5_f64),
            Degrees::<f64>::NEG_DEG_90,
        ),
        (
            Sextant::RedMagenta,
            Prop::from(0.5_f64),
            Degrees::<f64>::NEG_DEG_30,
        ),
        (
            Sextant::GreenYellow,
            Prop::from(0.5_f64),
            Degrees::<f64>::DEG_90,
        ),
        (
            Sextant::GreenCyan,
            Prop::from(0.5_f64),
            Degrees::<f64>::DEG_150,
        ),
    ] {
        let hue = Hue::Sextant(SextantHue(*sextant, *second));
        assert_approx_eq!(hue.hue_angle(), *angle, 0.0000001);
    }
}

// TODO: this test needs to be improved
#[test]
fn max_chroma_and_sum_ranges() {
    for hue in &Hue::PRIMARIES {
        assert!(hue.sum_range_for_chroma(Chroma::ZERO).is_none());
        assert_eq!(
            hue.sum_range_for_chroma(Chroma::ONE),
            Some(SumRange((Sum::ONE, Sum::ONE, Sum::ONE)))
        );
        for item in NON_ZERO_CHROMAS.iter() {
            let prop = Prop::from(*item);
            let range = hue.sum_range_for_chroma(Chroma::Either(prop)).unwrap();
            let max_chroma = hue.max_chroma_for_sum(range.shade_min()).unwrap();
            assert_approx_eq!(max_chroma, Chroma::Shade(prop));
            let max_chroma = hue.max_chroma_for_sum(range.tint_max()).unwrap();
            assert_approx_eq!(max_chroma, Chroma::Tint(prop), 0.000_000_000_000_001);
        }
    }
    for hue in &Hue::SECONDARIES {
        assert!(hue.sum_range_for_chroma(Chroma::ZERO).is_none());
        assert_eq!(
            hue.sum_range_for_chroma(Chroma::ONE),
            Some(SumRange((Sum::TWO, Sum::TWO, Sum::TWO)))
        );
        for item in NON_ZERO_CHROMAS.iter() {
            let prop = Prop::from(*item);
            let range = hue.sum_range_for_chroma(Chroma::Either(prop)).unwrap();
            let max_chroma = hue.max_chroma_for_sum(range.shade_min()).unwrap();
            assert_approx_eq!(max_chroma, Chroma::Shade(prop));
            let max_chroma = hue.max_chroma_for_sum(range.tint_max()).unwrap();
            assert_approx_eq!(max_chroma, Chroma::Tint(prop), 0.000000000000001);
        }
    }
    use Sextant::*;
    for sextant in &[
        RedYellow,
        RedMagenta,
        GreenCyan,
        GreenYellow,
        BlueCyan,
        BlueMagenta,
    ] {
        for item in SECOND_VALUES.iter() {
            let other = Prop::from(*item);
            let hue = Hue::Sextant(SextantHue(*sextant, other));
            assert!(hue.sum_range_for_chroma(Chroma::ZERO).is_none());
            assert_eq!(
                hue.sum_range_for_chroma(Chroma::ONE),
                Some(SumRange((
                    Sum::ONE + other,
                    Sum::ONE + other,
                    Sum::ONE + other
                )))
            );
        }
    }
}

#[test]
fn primary_max_chroma_rgbs() {
    for (hue, expected_rgb) in Hue::PRIMARIES.iter().zip(RGB::<f64>::PRIMARIES.iter()) {
        assert_eq!(hue.max_chroma_rgb_for_sum(Sum::ONE).unwrap(), *expected_rgb);
        assert!(hue.max_chroma_rgb_for_sum::<f64>(Sum::ZERO).is_none());
        assert!(hue.max_chroma_rgb_for_sum::<f64>(Sum::THREE).is_none());
        for sum in [
            Sum::from(0.0001_f64),
            Sum::from(0.25_f64),
            Sum::from(0.5_f64),
            Sum::from(0.75_f64),
            Sum::from(0.9999_f64),
        ]
        .iter()
        {
            let mut array = [Prop::ZERO, Prop::ZERO, Prop::ZERO];
            array[hue.indices().0 as usize] = (*sum).into();
            let expected: RGB<f64> = array.into();
            assert_eq!(hue.max_chroma_rgb_for_sum::<f64>(*sum).unwrap(), expected);
        }
        for sum in [
            Sum::from(2.0001_f64),
            Sum::from(2.25_f64),
            Sum::from(2.5_f64),
            Sum::from(2.75_f64),
            Sum::from(2.9999_f64),
        ]
        .iter()
        {
            let mut array = [Prop::ONE, Prop::ONE, Prop::ONE];
            array[hue.indices().1 as usize] = ((*sum - Sum::ONE) / 2).into();
            array[hue.indices().2 as usize] = ((*sum - Sum::ONE) / 2).into();
            let expected: RGB<f64> = array.into();
            assert_eq!(hue.max_chroma_rgb_for_sum::<f64>(*sum).unwrap(), expected);
        }
    }
}

#[test]
fn secondary_max_chroma_rgbs() {
    for (hue, expected_rgb) in Hue::SECONDARIES.iter().zip(RGB::<f64>::SECONDARIES.iter()) {
        assert_approx_eq!(
            hue.max_chroma_rgb_for_sum::<f64>(Sum::from(2.0_f64))
                .unwrap(),
            *expected_rgb
        );
        assert!(hue.max_chroma_rgb_for_sum::<f64>(Sum::ZERO).is_none());
        assert!(hue.max_chroma_rgb_for_sum::<f64>(Sum::THREE).is_none());
        for sum in [
            Sum::from(0.0001_f64),
            Sum::from(0.25_f64),
            Sum::from(0.5_f64),
            Sum::from(0.75_f64),
            Sum::ONE,
            Sum::from(1.5_f64),
            Sum::from(1.9999_f64),
        ]
        .iter()
        {
            let mut array = [Prop::ZERO, Prop::ZERO, Prop::ZERO];
            array[hue.indices().0 as usize] = (*sum / 2).into();
            array[hue.indices().1 as usize] = (*sum / 2).into();
            let expected: RGB<f64> = array.into();
            assert_eq!(hue.max_chroma_rgb_for_sum::<f64>(*sum).unwrap(), expected);
        }
        for sum in [
            Sum::from(2.0001_f64),
            Sum::from(2.25_f64),
            Sum::from(2.5_f64),
            Sum::from(2.75_f64),
            Sum::from(2.9999_f64),
        ]
        .iter()
        {
            let mut array = [Prop::ONE, Prop::ONE, Prop::ONE];
            array[hue.indices().2 as usize] = (*sum - Sum::from(2.0_f64)).into();
            let expected: RGB<f64> = array.into();
            assert_approx_eq!(hue.max_chroma_rgb_for_sum::<f64>(*sum).unwrap(), expected);
        }
    }
}

#[test]
fn other_max_chroma_rgbs() {
    use Sextant::*;
    for sextant in &[
        RedYellow,
        RedMagenta,
        GreenCyan,
        GreenYellow,
        BlueCyan,
        BlueMagenta,
    ] {
        for item in SECOND_VALUES.iter() {
            let second = Prop::from(*item);
            let sextant_hue = SextantHue(*sextant, second);
            let hue = Hue::Sextant(sextant_hue);
            assert!(hue.max_chroma_rgb_for_sum::<f64>(Sum::ZERO).is_none());
            assert!(hue.max_chroma_rgb_for_sum::<f64>(Sum::THREE).is_none());
            for item in VALID_OTHER_SUMS.iter() {
                let sum = Sum::from(*item);
                let rgb = hue.max_chroma_rgb_for_sum::<f64>(sum).unwrap();
                assert_approx_eq!(sum, rgb.sum());
                match Hue::try_from(&rgb).unwrap() {
                    Hue::Sextant(SextantHue(sextant_out, second_out)) => {
                        assert_eq!(sextant_hue.0, sextant_out);
                        assert_approx_eq!(sextant_hue.1, second_out, 0.000_000_1);
                    }
                    _ => panic!("it's gone pure"),
                }
            }
        }
    }
}

#[test]
fn min_max_sum_rgb_for_chroma() {
    for (hue, expected_rgb) in Hue::PRIMARIES.iter().zip(RGB::<f64>::PRIMARIES.iter()) {
        println!("{:?} : {:?}", hue, expected_rgb);
        assert_eq!(
            hue.min_sum_rgb_for_chroma::<f64>(Chroma::ONE),
            *expected_rgb
        );
        assert_eq!(
            hue.max_sum_rgb_for_chroma::<f64>(Chroma::ONE),
            *expected_rgb
        );
        let prop = Prop::from(0.5_f64);
        let chroma = Chroma::Either(prop);
        let shade = hue.min_sum_rgb_for_chroma(chroma);
        let tint = hue.max_sum_rgb_for_chroma(chroma);
        assert!(shade.value() < tint.value());
        assert_approx_eq!(shade.chroma(), chroma, 0.00000000001);
        assert_approx_eq!(tint.chroma(), chroma, 0.00000000001);
        assert_approx_eq!(shade.max_chroma_rgb(), tint.max_chroma_rgb(), 0.0000001);
    }
    for (hue, expected_rgb) in Hue::SECONDARIES.iter().zip(RGB::<f64>::SECONDARIES.iter()) {
        let prop = Prop::from(0.5_f64);
        let chroma = Chroma::Either(prop);
        println!("{:?} : {:?}", hue, expected_rgb);
        assert_eq!(hue.min_sum_rgb_for_chroma(Chroma::ONE), *expected_rgb);
        assert_eq!(hue.max_sum_rgb_for_chroma(Chroma::ONE), *expected_rgb);
        let shade = hue.min_sum_rgb_for_chroma(chroma);
        let tint = hue.max_sum_rgb_for_chroma(chroma);
        assert!(shade.value() < tint.value());
        assert_approx_eq!(shade.chroma(), chroma, 0.00000000001);
        assert_approx_eq!(tint.chroma(), chroma, 0.00000000001);
        assert_approx_eq!(shade.max_chroma_rgb(), tint.max_chroma_rgb(), 0.0000001);
    }
    use Sextant::*;
    for sextant in &[
        RedYellow,
        RedMagenta,
        GreenCyan,
        GreenYellow,
        BlueCyan,
        BlueMagenta,
    ] {
        for item in SECOND_VALUES.iter() {
            let second = Prop::from(*item);
            let hue = Hue::Sextant(SextantHue(*sextant, second));
            assert_eq!(hue.min_sum_rgb_for_chroma::<f64>(Chroma::ZERO), RGB::BLACK);
            assert_eq!(hue.max_sum_rgb_for_chroma::<f64>(Chroma::ZERO), RGB::WHITE);
            for prop in NON_ZERO_CHROMAS.iter().map(|a| Prop::from(*a)) {
                let chroma = Chroma::Either(prop);
                let shade = hue.min_sum_rgb_for_chroma(chroma);
                let tint = hue.max_sum_rgb_for_chroma(chroma);
                assert!(shade.sum() <= tint.sum());
                assert_approx_eq!(shade.chroma(), chroma, 0.00000000001);
                assert_approx_eq!(tint.chroma(), chroma, 0.00000000001);
                assert_approx_eq!(shade.max_chroma_rgb(), tint.max_chroma_rgb(), 0.000_001);
            }
        }
    }
}

#[test]
fn primary_rgb_for_sum_and_chroma() {
    for hue in &Hue::PRIMARIES {
        assert!(hue
            .rgb_for_sum_and_chroma::<f64>(Sum::ZERO, Chroma::ONE)
            .is_none());
        assert!(hue
            .rgb_for_sum_and_chroma::<f64>(Sum::THREE, Chroma::ONE)
            .is_none());
        assert!(hue
            .rgb_for_sum_and_chroma::<f64>(Sum::ZERO, Chroma::ZERO)
            .is_none());
        assert!(hue
            .rgb_for_sum_and_chroma::<f64>(Sum::THREE, Chroma::ZERO)
            .is_none());
        for prop in NON_ZERO_CHROMAS.iter().map(|item| Prop::from(*item)) {
            let chroma = Chroma::Either(prop);
            for sum in VALID_OTHER_SUMS.iter().map(|item| Sum::from(*item)) {
                if let Some(rgb) = hue.rgb_for_sum_and_chroma::<f64>(sum, chroma) {
                    assert_approx_eq!(rgb.sum(), sum, 0.000_000_000_1);
                    assert_approx_eq!(rgb.chroma(), chroma, 0.000_000_000_1);
                    assert_approx_eq!(Hue::try_from(&rgb).unwrap(), hue);
                } else {
                    let range = hue.sum_range_for_chroma(chroma).unwrap();
                    println!("{:?}, {:?}, {:?} : {:?}", *hue, sum, chroma, range);
                    assert!(range.compare_sum(sum).is_failure());
                }
            }
        }
    }
}

#[test]
fn secondary_rgb_for_sum_and_chroma() {
    for hue in &Hue::SECONDARIES {
        assert!(hue
            .rgb_for_sum_and_chroma::<f64>(Sum::ZERO, Chroma::ONE)
            .is_none());
        assert!(hue
            .rgb_for_sum_and_chroma::<f64>(Sum::THREE, Chroma::ONE)
            .is_none());
        assert!(hue
            .rgb_for_sum_and_chroma::<f64>(Sum::ZERO, Chroma::ZERO)
            .is_none());
        assert!(hue
            .rgb_for_sum_and_chroma::<f64>(Sum::THREE, Chroma::ZERO)
            .is_none());
        for prop in NON_ZERO_CHROMAS.iter().map(|item| Prop::from(*item)) {
            let chroma = Chroma::Either(prop);
            for sum in VALID_OTHER_SUMS.iter().map(|item| Sum::from(*item)) {
                if let Some(rgb) = hue.rgb_for_sum_and_chroma::<f64>(sum, chroma) {
                    assert_approx_eq!(rgb.sum(), sum, 0.000_000_1);
                    assert_approx_eq!(rgb.chroma(), chroma, 0.000_000_1);
                    assert_approx_eq!(Hue::try_from(&rgb).unwrap(), hue);
                } else {
                    let range = hue.sum_range_for_chroma(chroma).unwrap();
                    assert!(range.compare_sum(sum).is_failure());
                }
            }
        }
    }
}

#[test]
fn general_rgb_for_sum_and_chroma() {
    use Sextant::*;
    for sextant in &[
        RedYellow,
        RedMagenta,
        GreenCyan,
        GreenYellow,
        BlueCyan,
        BlueMagenta,
    ] {
        for second in SECOND_VALUES.iter().map(|a| Prop::from(*a)) {
            let sextant_hue = SextantHue(*sextant, second);
            let hue = Hue::Sextant(sextant_hue);
            assert!(hue
                .rgb_for_sum_and_chroma::<f64>(Sum::ZERO, Chroma::ONE)
                .is_none());
            assert!(hue
                .rgb_for_sum_and_chroma::<f64>(Sum::THREE, Chroma::ONE)
                .is_none());
            assert!(hue
                .rgb_for_sum_and_chroma::<f64>(Sum::ZERO, Chroma::ZERO)
                .is_none());
            assert!(hue
                .rgb_for_sum_and_chroma::<f64>(Sum::THREE, Chroma::ZERO)
                .is_none());
            for prop in NON_ZERO_CHROMAS.iter().map(|a| Prop::from(*a)) {
                let chroma = Chroma::Either(prop);
                let sum_range = hue.sum_range_for_chroma(chroma).unwrap();
                for sum in VALID_OTHER_SUMS.iter().map(|a| Sum::from(*a)) {
                    if let Some(rgb) = hue.rgb_for_sum_and_chroma::<f64>(sum, chroma) {
                        use SumOrdering::*;
                        match sum_range.compare_sum(sum) {
                            Shade(_, _) => {
                                assert_approx_eq!(rgb.sum(), sum, 0.000_000_001);
                                assert_approx_eq!(rgb.chroma(), chroma, 0.000_000_1);
                                assert_approx_eq!(Hue::try_from(&rgb).unwrap(), hue, 0.000_001);
                            }
                            Tint(_, _) => {
                                assert_approx_eq!(rgb.sum(), sum, 0.000_000_001);
                                assert_approx_eq!(rgb.chroma(), chroma, 0.000_000_1);
                                assert_approx_eq!(Hue::try_from(&rgb).unwrap(), hue, 0.000_000_1);
                            }
                            _ => (),
                        }
                    } else {
                        let range = hue.sum_range_for_chroma(chroma).unwrap();
                        assert!(range.compare_sum(sum).is_failure());
                    }
                }
            }
        }
    }
}
