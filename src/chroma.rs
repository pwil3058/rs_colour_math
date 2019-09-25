// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::ColourComponent;

pub fn calc_other<F: ColourComponent + std::fmt::Debug>(xy: (F, F)) -> F {
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
        F::HALF + xy.0 * F::SQRT_3 / F::TWO / xy.1.abs()
    }
}

pub fn calc_other_alt<F: ColourComponent>(xy: (F, F)) -> F {
    if xy.1 == F::ZERO {
        if xy.0 > F::ZERO {
            F::ZERO // red
        } else if xy.0 < F::ZERO {
            F::ONE // cyan
        } else {
            panic!("calc_other((0.0, 0.0)) is illegal")
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
            F::HALF + xy.0 * F::SQRT_3 / F::TWO / xy.1.abs()
        } else if xy.0 > F::ZERO {
            F::ONE // yellow or magenta
        } else {
            F::ZERO // green or blue
        }
    }
}

#[cfg(test)]
mod test {
    use crate::rgb::*;
    use crate::ColourComponent;
    use float_cmp::*;
    use normalised_angles::Degrees;

    const NON_ZERO_VALUES: [f64; 7] = [0.000000001, 0.025, 0.5, 0.75, 0.9, 0.99999, 1.0];

    #[test]
    fn calc_other_from_rgb() {
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
                    let other = super::calc_other(rgb.xy());
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
    fn calc_other_from_angle() {
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
            let other = super::calc_other(hue_angle.xy());
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
    fn calc_other_from_comparison() {
        use crate::hue;
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
            let other = super::calc_other(hue_angle.xy());
            assert!(other.is_proportion(), "other = {}", other);
            assert!(
                approx_eq!(f64, other, *expected, epsilon = 0.00000000001),
                "{} :: {} :: {}",
                expected,
                other,
                angle
            );
            let other_hue = hue::calc_other(hue_angle.abs());
            assert!(
                approx_eq!(f64, other, other_hue, epsilon = 0.000000000000001),
                "{} :: {} :: {} :: {}",
                expected,
                other,
                other_hue,
                angle
            );
        }
    }
}
