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

#[cfg(test)]
mod test {
    use crate::rgb::*;
    use crate::ColourComponent;
    use float_cmp::*;

    const NON_ZERO_VALUES: [f64; 7] = [0.000000001, 0.025, 0.5, 0.75, 0.9, 0.99999, 1.0];

    #[test]
    fn calc_other() {
        for val in NON_ZERO_VALUES.iter() {
            for ratio in [0.01, 0.25, 0.5, 0.75, 0.99].iter() {
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
                    assert!(approx_eq!(f64, other, *expected, epsilon = 0.00000000001));
                }
            }
        }
    }
}
