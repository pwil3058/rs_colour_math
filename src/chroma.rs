// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use normalised_angles::Degrees;

use crate::ColourComponent;

pub(crate) fn calc_other_from_angle<F: ColourComponent>(abs_angle: Degrees<F>) -> F {
    if [F::RED_ANGLE, F::GREEN_ANGLE].contains(&abs_angle.degrees()) {
        F::ZERO
    } else if [F::YELLOW_ANGLE, F::CYAN_ANGLE].contains(&abs_angle.degrees()) {
        F::ONE
    } else {
        fn f<F: ColourComponent>(angle: Degrees<F>) -> F {
            // Careful of float not fully representing real numbers
            (angle.sin() / (Degrees::from(F::GREEN_ANGLE) - angle).sin()).min(F::ONE)
        };
        if abs_angle.degrees() <= F::YELLOW_ANGLE {
            f(abs_angle)
        } else if abs_angle.degrees() <= F::GREEN_ANGLE {
            f(Degrees::from(F::GREEN_ANGLE) - abs_angle)
        } else {
            f(abs_angle - Degrees::from(F::GREEN_ANGLE))
        }
    }
}

pub fn calc_other_from_xy<F: ColourComponent>(xy: (F, F)) -> F {
    if xy.0.abs() * F::SQRT_3 > xy.1.abs() {
        let divisor = xy.0.abs() * F::SQRT_3 + xy.1.abs();
        debug_assert!(divisor != F::ZERO);
        let x = xy.0 * F::SQRT_3 / divisor;
        if xy.0 >= F::ZERO {
            (F::ONE - x) * F::TWO
        } else {
            -(F::TWO * x + F::ONE)
        }
    } else {
        F::HALF + xy.0 * F::SIN_120 / xy.1.abs()
    }
}

pub fn calc_other_from_xy_alt<F: ColourComponent>(xy: (F, F)) -> F {
    if xy.1 == F::ZERO {
        if xy.0 > F::ZERO {
            F::ZERO // red
        } else if xy.0 < F::ZERO {
            F::ONE // cyan
        } else {
            panic!("calc_other_from_xy((0.0, 0.0)) is illegal")
        }
    } else {
        let x_sqrt_3 = xy.0.abs() * F::SQRT_3;
        if x_sqrt_3 > xy.1.abs() {
            let divisor = xy.0.abs() * F::SQRT_3 + xy.1.abs();
            let x = xy.0 * F::SQRT_3 / divisor;
            if xy.0 >= F::ZERO {
                (F::ONE - x) * F::TWO
            } else {
                -(x * F::TWO + F::ONE)
            }
        } else if x_sqrt_3 < xy.1.abs() {
            F::HALF + xy.0 * F::SIN_120 / xy.1.abs()
        } else if xy.0 > F::ZERO {
            F::ONE // yellow or magenta
        } else {
            F::ZERO // green or blue
        }
    }
}

pub(crate) fn calc_chroma_correction<F: ColourComponent>(other: F) -> F {
    debug_assert!(other.is_proportion(), "other: {:?}", other);
    // Careful of fact floats only approximate real numbers
    (F::ONE + other * other - other).sqrt().min(F::ONE).recip()
}

pub fn sum_range_for_chroma<F: ColourComponent>(other: F, chroma: F) -> (F, F) {
    debug_assert!(other.is_proportion(), "other: {:?}", other);
    debug_assert!(chroma.is_proportion(), "chroma: {:?}", chroma);
    if chroma == F::ONE {
        (F::ONE + other, F::ONE + other)
    } else {
        let temp = other * chroma;
        (chroma + temp, F::THREE + temp - F::TWO * chroma)
    }
}

pub fn max_chroma_for_sum<F: ColourComponent>(other: F, sum: F) -> F {
    debug_assert!(other.is_proportion(), "other: {:?}", other);
    debug_assert!(sum >= F::ZERO && sum <= F::THREE, "sum: {:?}", sum);
    if sum == F::ZERO || sum == F::THREE {
        F::ZERO
    } else if sum < F::ONE + other {
        sum / (F::ONE + other)
    } else if sum > F::ONE + other {
        (F::THREE - sum) / (F::TWO - other)
    } else {
        F::ONE
    }
}

#[cfg(test)]
mod test {
    use crate::rgb::*;
    use crate::ColourComponent;
    use float_cmp::*;
    use normalised_angles::Degrees;

    const NON_ZERO_VALUES: [f64; 7] = [0.000000001, 0.025, 0.5, 0.75, 0.9, 0.99999, 1.0];
    const OTHER_VALUES: [f64; 13] = [
        0.0, 0.0000001, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 0.99999, 1.0,
    ];

    #[test]
    fn calc_other_from_angle_from_angle() {
        for (angle, expected) in &[
            (f64::RED_ANGLE, 0.0),
            (f64::GREEN_ANGLE, 0.0),
            (f64::BLUE_ANGLE, 0.0),
            (f64::CYAN_ANGLE, 1.0),
            (f64::MAGENTA_ANGLE, 1.0),
            (f64::YELLOW_ANGLE, 1.0),
            (-180.0, 1.0),
            (-165.0, 2.0 / (f64::SQRT_3 + 1.0)),
            (-150.0, 0.5),
            (-135.0, (f64::SQRT_3 - 1.0) / (f64::SQRT_3 + 1.0)),
            (-120.0, 0.0),
            (-105.0, (f64::SQRT_3 - 1.0) / (f64::SQRT_3 + 1.0)),
            (-90.0, 0.5),
            (-75.0, 2.0 / (f64::SQRT_3 + 1.0)),
            (-60.0, 1.0),
            (-45.0, 2.0 / (f64::SQRT_3 + 1.0)),
            (-30.0, 0.5),
            (-15.0, (f64::SQRT_3 - 1.0) / (f64::SQRT_3 + 1.0)),
            (0.0, 0.0),
            (15.0, (f64::SQRT_3 - 1.0) / (f64::SQRT_3 + 1.0)),
            (30.0, 0.5),
            (45.0, 2.0 / (f64::SQRT_3 + 1.0)),
            (60.0, 1.0),
            (75.0, 2.0 / (f64::SQRT_3 + 1.0)),
            (90.0, 0.5),
            (105.0, (f64::SQRT_3 - 1.0) / (f64::SQRT_3 + 1.0)),
            (120.0, 0.0),
            (135.0, (f64::SQRT_3 - 1.0) / (f64::SQRT_3 + 1.0)),
            (150.0, 0.5),
            (165.0, 2.0 / (f64::SQRT_3 + 1.0)),
        ] {
            let hue_angle = Degrees::<f64>::from(*angle);
            let other = super::calc_other_from_angle(hue_angle.abs());
            assert!(other.is_proportion(), "other = {}", other);
            assert!(
                approx_eq!(f64, other, *expected, epsilon = 0.000000000000001),
                "{} :: {} :: {}",
                expected,
                other,
                angle
            );
        }
    }

    #[test]
    fn calc_other_from_xy_from_rgb() {
        for val in NON_ZERO_VALUES.iter() {
            for ratio in NON_ZERO_VALUES.iter() {
                let mod_val = val * *ratio;
                for (array, expected) in &[
                    ([*val, 0.0, 0.0], 0.0),
                    ([0.0, *val, 0.0], 0.0),
                    ([0.0, 0.0, *val], 0.0),
                    ([*val, *val, 0.0], 1.0),
                    ([0.0, *val, *val], 1.0),
                    ([*val, 0.0, *val], 1.0),
                    ([*val, mod_val, 0.0], *ratio),
                    ([0.0, *val, mod_val], *ratio),
                    ([*val, 0.0, mod_val], *ratio),
                    ([mod_val, *val, 0.0], *ratio),
                    ([0.0, mod_val, *val], *ratio),
                    ([mod_val, 0.0, *val], *ratio),
                ] {
                    let rgb = RGB::<f64>::from(*array);
                    let other = super::calc_other_from_xy(rgb.xy());
                    assert!(other.is_proportion(), "other = {}", other);
                    assert!(
                        approx_eq!(f64, other, *expected, epsilon = 0.00000000001),
                        "{} :: {} :: {:?}",
                        expected,
                        other,
                        rgb
                    );
                }
            }
        }
    }

    #[test]
    fn calc_other_from_xy_from_angle() {
        for (angle, expected) in &[
            (f64::RED_ANGLE, 0.0),
            (f64::GREEN_ANGLE, 0.0),
            (f64::BLUE_ANGLE, 0.0),
            (f64::CYAN_ANGLE, 1.0),
            (f64::MAGENTA_ANGLE, 1.0),
            (f64::YELLOW_ANGLE, 1.0),
            (-180.0, 1.0),
            (-165.0, 2.0 / (f64::SQRT_3 + 1.0)),
            (-150.0, 0.5),
            (-135.0, (f64::SQRT_3 - 1.0) / (f64::SQRT_3 + 1.0)),
            (-120.0, 0.0),
            (-105.0, (f64::SQRT_3 - 1.0) / (f64::SQRT_3 + 1.0)),
            (-90.0, 0.5),
            (-75.0, 2.0 / (f64::SQRT_3 + 1.0)),
            (-60.0, 1.0),
            (-45.0, 2.0 / (f64::SQRT_3 + 1.0)),
            (-30.0, 0.5),
            (-15.0, (f64::SQRT_3 - 1.0) / (f64::SQRT_3 + 1.0)),
            (0.0, 0.0),
            (15.0, (f64::SQRT_3 - 1.0) / (f64::SQRT_3 + 1.0)),
            (30.0, 0.5),
            (45.0, 2.0 / (f64::SQRT_3 + 1.0)),
            (60.0, 1.0),
            (75.0, 2.0 / (f64::SQRT_3 + 1.0)),
            (90.0, 0.5),
            (105.0, (f64::SQRT_3 - 1.0) / (f64::SQRT_3 + 1.0)),
            (120.0, 0.0),
            (135.0, (f64::SQRT_3 - 1.0) / (f64::SQRT_3 + 1.0)),
            (150.0, 0.5),
            (165.0, 2.0 / (f64::SQRT_3 + 1.0)),
        ] {
            let hue_angle = Degrees::<f64>::from(*angle);
            let other = super::calc_other_from_xy(hue_angle.xy());
            assert!(other.is_proportion(), "other = {}", other);
            assert!(
                approx_eq!(f64, other, *expected, epsilon = 0.000000000000001),
                "{} :: {} :: {}",
                expected,
                other,
                angle
            );
        }
    }

    #[test]
    fn calc_other_from_xy_from_comparison() {
        for (angle, expected) in &[
            (f64::RED_ANGLE, 0.0),
            (f64::GREEN_ANGLE, 0.0),
            (f64::BLUE_ANGLE, 0.0),
            (f64::CYAN_ANGLE, 1.0),
            (f64::MAGENTA_ANGLE, 1.0),
            (f64::YELLOW_ANGLE, 1.0),
            (-180.0, 1.0),
            (-165.0, 2.0 / (f64::SQRT_3 + 1.0)),
            (-150.0, 0.5),
            (-135.0, (f64::SQRT_3 - 1.0) / (f64::SQRT_3 + 1.0)),
            (-120.0, 0.0),
            (-105.0, (f64::SQRT_3 - 1.0) / (f64::SQRT_3 + 1.0)),
            (-90.0, 0.5),
            (-75.0, 2.0 / (f64::SQRT_3 + 1.0)),
            (-60.0, 1.0),
            (-45.0, 2.0 / (f64::SQRT_3 + 1.0)),
            (-30.0, 0.5),
            (-15.0, (f64::SQRT_3 - 1.0) / (f64::SQRT_3 + 1.0)),
            (0.0, 0.0),
            (15.0, (f64::SQRT_3 - 1.0) / (f64::SQRT_3 + 1.0)),
            (30.0, 0.5),
            (45.0, 2.0 / (f64::SQRT_3 + 1.0)),
            (60.0, 1.0),
            (75.0, 2.0 / (f64::SQRT_3 + 1.0)),
            (90.0, 0.5),
            (105.0, (f64::SQRT_3 - 1.0) / (f64::SQRT_3 + 1.0)),
            (120.0, 0.0),
            (135.0, (f64::SQRT_3 - 1.0) / (f64::SQRT_3 + 1.0)),
            (150.0, 0.5),
            (165.0, 2.0 / (f64::SQRT_3 + 1.0)),
            (180.0, 1.0),
        ] {
            let hue_angle = Degrees::<f64>::from(*angle);
            let other_xy = super::calc_other_from_xy_alt(hue_angle.xy());
            let other_hue = super::calc_other_from_angle(hue_angle.abs());
            assert!(
                approx_eq!(f64, other_xy, other_hue, epsilon = 0.000000000000001),
                "expected: {} :: fm y: {} :: fm hue: {} :: angle: {}",
                expected,
                other_xy,
                other_hue,
                angle,
            );
        }
    }

    #[test]
    fn max_chroma_and_sum_ranges() {
        for other in OTHER_VALUES.iter() {
            assert_eq!(
                super::sum_range_for_chroma::<f64>(*other, 0.0),
                (0.0, 3.0),
                "other: {}",
                other
            );
            assert_eq!(
                super::sum_range_for_chroma::<f64>(*other, 1.0),
                (1.0 + *other, 1.0 + *other),
                "other: {}",
                other
            );
            for chroma in NON_ZERO_VALUES.iter() {
                let (shade, tint) = super::sum_range_for_chroma(*other, *chroma);
                let max_chroma = super::max_chroma_for_sum(*other, shade);
                assert!(
                    approx_eq!(f64, max_chroma, *chroma, epsilon = 0.000000000000001),
                    "chroma {} == {} :: other: {} shade: {}",
                    max_chroma,
                    *chroma,
                    other,
                    shade
                );
                let max_chroma = super::max_chroma_for_sum(*other, tint);
                assert!(
                    approx_eq!(f64, max_chroma, *chroma, epsilon = 0.000000000000001),
                    "chroma {} == {} :: other: {} tint: {}",
                    max_chroma,
                    *chroma,
                    other,
                    tint
                );
            }
        }
    }
}
