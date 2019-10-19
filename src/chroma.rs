// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::convert::TryFrom;

use normalised_angles::*;

use crate::{rgb::RGB, ColourAngle, ColourComponent};
use std::path::Prefix::DeviceNS;

pub(crate) fn calc_other_from_angle<F: ColourComponent>(abs_angle: Degrees<F>) -> F {
    if [Degrees::RED_ANGLE, Degrees::GREEN_ANGLE].contains(&abs_angle) {
        F::ZERO
    } else if [Degrees::YELLOW_ANGLE, Degrees::CYAN_ANGLE].contains(&abs_angle) {
        F::ONE
    } else {
        fn f<F: ColourComponent>(angle: Degrees<F>) -> F {
            // Careful of float not fully representing real numbers
            (angle.sin() / (Degrees::GREEN_ANGLE - angle).sin()).min(F::ONE)
        };
        if abs_angle <= Degrees::YELLOW_ANGLE {
            f(abs_angle)
        } else if abs_angle <= Degrees::GREEN_ANGLE {
            f(Degrees::GREEN_ANGLE - abs_angle)
        } else {
            f(abs_angle - Degrees::GREEN_ANGLE)
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HueData<F: ColourComponent> {
    // TODO: un pub HueData fields
    pub(crate) second: F,
    pub(crate) io: [u8; 3],
}

impl<F: ColourComponent> From<Degrees<F>> for HueData<F> {
    fn from(angle: Degrees<F>) -> Self {
        let (second, io) = if angle == Degrees::RED_ANGLE {
            (F::ZERO, [0, 1, 2])
        } else if angle == Degrees::GREEN_ANGLE {
            (F::ZERO, [1, 2, 0])
        } else if angle == Degrees::BLUE_ANGLE {
            (F::ZERO, [2, 0, 1])
        } else if angle == Degrees::CYAN_ANGLE || angle == -Degrees::CYAN_ANGLE {
            (F::ONE, [2, 1, 0])
        } else if angle == Degrees::MAGENTA_ANGLE {
            (F::ONE, [0, 2, 1])
        } else if angle == Degrees::YELLOW_ANGLE {
            (F::ONE, [1, 0, 2])
        } else {
            fn f<F: ColourComponent>(angle: Degrees<F>) -> F {
                // Careful of float not fully representing real numbers
                (angle.sin() / (Degrees::GREEN_ANGLE - angle).sin()).min(F::ONE)
            };
            if angle >= Degrees::DEG_0 {
                if angle <= Degrees::YELLOW_ANGLE {
                    (f(angle), [0, 1, 2])
                } else if angle <= Degrees::GREEN_ANGLE {
                    (f(Degrees::GREEN_ANGLE - angle), [1, 0, 2])
                } else {
                    (f(angle - Degrees::GREEN_ANGLE), [1, 2, 0])
                }
            } else if angle >= Degrees::MAGENTA_ANGLE {
                (f(-angle), [0, 2, 1])
            } else if angle >= Degrees::BLUE_ANGLE {
                (f(Degrees::GREEN_ANGLE + angle), [2, 0, 1])
            } else {
                (f(-angle - Degrees::GREEN_ANGLE), [2, 1, 0])
            }
        };
        Self { second, io }
    }
}

impl<F: ColourComponent> TryFrom<(F, F)> for HueData<F> {
    type Error = &'static str;

    fn try_from(xy: (F, F)) -> Result<Self, Self::Error> {
        if xy.1 == F::ZERO {
            if xy.0 > F::ZERO {
                Ok(Self {
                    second: F::ZERO,
                    io: [0, 1, 2],
                }) // red
            } else if xy.0 < F::ZERO {
                Ok(Self {
                    second: F::ONE,
                    io: [1, 2, 0],
                }) // cyan
            } else {
                Err("Greys have no hue and, ergo, can't generate HueData")
            }
        } else {
            let x_sqrt_3 = xy.0.abs() * F::SQRT_3;
            if x_sqrt_3 > xy.1.abs() {
                let divisor = xy.0.abs() * F::SQRT_3 + xy.1.abs();
                let x = xy.0 * F::SQRT_3 / divisor;
                if xy.0 >= F::ZERO {
                    Ok(Self {
                        second: (F::ONE - x) * F::TWO,
                        io: if xy.1 > F::ZERO { [0, 1, 2] } else { [0, 2, 1] },
                    })
                } else {
                    Ok(Self {
                        second: -(x * F::TWO + F::ONE),
                        io: if xy.1 > F::ZERO { [1, 2, 0] } else { [2, 1, 0] },
                    })
                }
            } else if x_sqrt_3 < xy.1.abs() {
                Ok(Self {
                    second: F::HALF + xy.0 * F::SIN_120 / xy.1.abs(),
                    io: if xy.1 > F::ZERO { [1, 0, 2] } else { [2, 0, 1] },
                })
            } else if xy.0 > F::ZERO {
                Ok(Self {
                    second: F::ONE,
                    io: if xy.1 > F::ZERO { [0, 1, 2] } else { [0, 2, 1] },
                }) // yellow and magenta
            } else {
                Ok(Self {
                    second: F::ZERO,
                    io: if xy.1 > F::ZERO { [1, 0, 2] } else { [2, 0, 1] },
                }) // green and blue
            }
        }
    }
}

impl<F: ColourComponent> std::default::Default for HueData<F> {
    fn default() -> Self {
        Self {
            second: F::ZERO,
            io: [0, 1, 2],
        }
    }
}

impl<F: ColourComponent> HueData<F> {
    pub fn hue_angle(&self) -> Degrees<F> {
        if self.second == F::ZERO {
            match self.io[0] {
                0 => Degrees::RED_ANGLE,
                1 => Degrees::GREEN_ANGLE,
                2 => Degrees::BLUE_ANGLE,
                _ => panic!("illegal colour component index: {}", self.io[0]),
            }
        } else if self.second == F::ONE {
            match self.io[2] {
                0 => Degrees::CYAN_ANGLE,
                1 => Degrees::MAGENTA_ANGLE,
                2 => Degrees::YELLOW_ANGLE,
                _ => panic!("illegal colour component index: {}", self.io[0]),
            }
        } else {
            let sin = F::SQRT_3 * self.second
                / F::TWO
                / (F::ONE - self.second + self.second.powi(2)).sqrt();
            let angle = Degrees::asin(sin);
            match self.io {
                [0, 1, 2] => angle,
                [1, 0, 2] => Degrees::GREEN_ANGLE - angle,
                [1, 2, 0] => Degrees::GREEN_ANGLE + angle,
                [0, 2, 1] => -angle,
                [2, 0, 1] => Degrees::BLUE_ANGLE + angle,
                [2, 1, 0] => Degrees::BLUE_ANGLE - angle,
                _ => panic!("illegal colour component indices: {:?}", self.io),
            }
        }
    }

    pub fn chroma_correction(&self) -> F {
        calc_chroma_correction(self.second)
    }

    pub fn max_chroma_rgb(&self) -> RGB<F> {
        let mut array = [F::ZERO, F::ZERO, F::ZERO];
        array[self.io[0] as usize] = F::ONE;
        array[self.io[1] as usize] = self.second;
        array.into()
    }

    pub fn max_chroma_rgb_for_sum(&self, sum: F) -> RGB<F> {
        debug_assert!(sum >= F::ZERO && sum <= F::THREE, "sum: {:?}", sum);
        let mut array: [F; 3] = [F::ZERO, F::ZERO, F::ZERO];
        if sum == F::ZERO {
            // Nothing to do
        } else if sum == F::THREE {
            array = [F::ONE, F::ONE, F::ONE];
        } else if self.second == F::ZERO {
            // pure red, green or blue
            if sum <= F::ONE {
                array[self.io[0] as usize] = sum;
            } else {
                array[self.io[0] as usize] = F::ONE;
                array[self.io[1] as usize] = ((sum - F::ONE) / F::TWO).min(F::ONE);
                array[self.io[2] as usize] = array[self.io[1] as usize];
            }
        } else if self.second == F::ONE {
            // pure cyan, magenta or yellow
            if sum <= F::TWO {
                array[self.io[0] as usize] = (sum / F::TWO).min(F::ONE);
                array[self.io[1] as usize] = array[self.io[0] as usize];
            } else {
                array[self.io[0] as usize] = F::ONE;
                array[self.io[1] as usize] = F::ONE;
                array[self.io[2] as usize] = (sum - F::TWO).min(F::ONE);
            }
        } else if sum < F::ONE + self.second {
            let divisor = F::ONE + self.second;
            array[self.io[0] as usize] = (sum / divisor).min(F::ONE);
            array[self.io[1] as usize] = sum * self.second / divisor;
        } else if sum > F::ONE + self.second {
            let chroma = (F::THREE - sum) / (F::TWO - self.second);
            let oc = self.second * chroma;
            array[self.io[0] as usize] = ((sum + F::TWO * chroma - oc) / F::THREE).min(F::ONE);
            array[self.io[1] as usize] = (sum + F::TWO * oc - chroma) / F::THREE;
            array[self.io[2] as usize] = (sum - oc - chroma) / F::THREE;
        } else {
            array[self.io[0] as usize] = F::ONE;
            array[self.io[1] as usize] = self.second;
        };
        array.into()
    }

    pub fn min_sum_rgb_for_chroma(&self, chroma: F) -> RGB<F> {
        debug_assert!(chroma.is_proportion(), "chroma: {:?}", chroma);
        let mut array: [F; 3] = [F::ZERO, F::ZERO, F::ZERO];
        if chroma == F::ZERO {
            // do nothing
        } else if chroma == F::ONE {
            array[self.io[0] as usize] = F::ONE;
            array[self.io[1] as usize] = self.second;
        } else if self.second == F::ZERO {
            array[self.io[0] as usize] = chroma;
        } else if self.second == F::ONE {
            array[self.io[0] as usize] = chroma;
            array[self.io[1] as usize] = chroma;
        } else {
            array[self.io[0] as usize] = chroma;
            array[self.io[1] as usize] = chroma * self.second;
        };
        array.into()
    }

    pub fn max_sum_rgb_for_chroma(&self, chroma: F) -> RGB<F> {
        debug_assert!(chroma.is_proportion(), "chroma: {:?}", chroma);
        let mut array: [F; 3] = [F::ZERO, F::ZERO, F::ZERO];
        if chroma == F::ZERO {
            array = [F::ONE, F::ONE, F::ONE];
        } else if chroma == F::ONE {
            array[self.io[0] as usize] = F::ONE;
            array[self.io[1] as usize] = self.second;
        } else if self.second == F::ZERO {
            array[self.io[0] as usize] = F::ONE;
            array[self.io[1] as usize] = F::ONE - chroma;
            array[self.io[2] as usize] = array[self.io[1] as usize];
        } else if self.second == F::ONE {
            array[self.io[0] as usize] = F::ONE;
            array[self.io[1] as usize] = F::ONE;
            array[self.io[2] as usize] = F::ONE - chroma;
        } else {
            array[self.io[0] as usize] = F::ONE;
            array[self.io[2] as usize] = F::ONE - chroma;
            array[self.io[1] as usize] = chroma * self.second + array[self.io[2] as usize];
        };
        array.into()
    }

    pub fn rgb_for_sum_and_chroma(&self, sum: F, chroma: F) -> Option<RGB<F>> {
        debug_assert!(chroma.is_proportion(), "chroma: {:?}", chroma);
        debug_assert!(sum >= F::ZERO && sum <= F::THREE, "sum: {:?}", sum);
        let mut array: [F; 3] = [F::ZERO, F::ZERO, F::ZERO];
        if sum == F::ZERO {
            if chroma > F::ZERO {
                return None;
            }
        } else if sum == F::THREE {
            if chroma > F::ZERO {
                return None;
            } else {
                array = [F::ONE, F::ONE, F::ONE];
            }
        } else if chroma == F::ZERO {
            let value = sum / F::THREE;
            array = [value, value, value];
        } else if chroma == F::ONE {
            if sum == F::ONE + self.second {
                array[self.io[0] as usize] = F::ONE;
                array[self.io[1] as usize] = self.second;
            } else {
                return None;
            }
        } else if self.second == F::ZERO {
            // pure red, green or blue
            array[self.io[0] as usize] = (sum + F::TWO * chroma) / F::THREE;
            array[self.io[1] as usize] = (sum - chroma) / F::THREE;
            array[self.io[2] as usize] = array[self.io[1] as usize];
        } else if self.second == F::ONE {
            // pure cyan, magenta or yellow
            array[self.io[0] as usize] = (sum + chroma) / F::THREE;
            array[self.io[1] as usize] = array[self.io[0] as usize];
            array[self.io[2] as usize] = (sum - F::TWO * chroma) / F::THREE;
        } else {
            let oc = self.second * chroma;
            array[self.io[0] as usize] = (sum + F::TWO * chroma - oc) / F::THREE;
            array[self.io[1] as usize] = (sum + F::TWO * oc - chroma) / F::THREE;
            array[self.io[2] as usize] = (sum - oc - chroma) / F::THREE;
        }
        if array[self.io[0] as usize] > F::ONE || array[self.io[2] as usize] < F::ZERO {
            return None;
        } else {
            // NB: because floats only approximate real numbers trying to
            // set chroma too small (but non zero) results in a drift
            // in the hue angle of the resulting RGB. When this
            // happens we go straight to a zero chroma RGB
            if chroma < F::from(0.00001).unwrap() && chroma > F::ZERO {
                let rgb: RGB<F> = array.into();
                let xy: (F, F) = rgb.xy();
                let rgb_second = calc_other_from_xy_alt(xy);
                // deviation "second" indicates a drift in the hue
                if (rgb_second - self.second).abs() / rgb_second > F::from(0.0000000001).unwrap() {
                    let value = sum / F::THREE;
                    array = [value, value, value];
                }
            };
        }
        Some(array.into())
    }
}

#[cfg(test)]
mod test {
    use crate::chroma::{
        calc_chroma_correction, calc_other_from_xy, max_chroma_for_sum, sum_range_for_chroma,
        HueData,
    };
    use crate::rgb::*;
    use crate::ColourComponent;
    use float_plus::*;
    use normalised_angles::Degrees;

    const NON_ZERO_VALUES: [f64; 7] = [0.000000001, 0.025, 0.5, 0.75, 0.9, 0.99999, 1.0];
    const NON_ZERO_SUMS: [f64; 21] = [
        0.000000001,
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
        2.99999,
        3.0,
    ];
    const OTHER_VALUES: [f64; 13] = [
        0.0, 0.0000001, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 0.99999, 1.0,
    ];

    #[test]
    fn calc_other_from_angle_from_angle() {
        for (angle, expected) in &[
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
            let other = super::calc_other_from_angle(hue_angle.abs());
            assert!(other.is_proportion(), "other = {}", other);
            assert_approx_eq!(other, *expected);
        }
    }

    #[test]
    fn hue_data_from_angle() {
        for (angle, expected, io) in &[
            (-180.0, 1.0, [2, 1, 0]),
            (-165.0, 2.0 / (f64::SQRT_3 + 1.0), [2, 1, 0]),
            (-150.0, 0.5, [2, 1, 0]),
            (-135.0, (f64::SQRT_3 - 1.0) / (f64::SQRT_3 + 1.0), [2, 1, 0]),
            (-120.0, 0.0, [2, 0, 1]),
            (-105.0, (f64::SQRT_3 - 1.0) / (f64::SQRT_3 + 1.0), [2, 0, 1]),
            (-90.0, 0.5, [2, 0, 1]),
            (-75.0, 2.0 / (f64::SQRT_3 + 1.0), [2, 0, 1]),
            (-60.0, 1.0, [0, 2, 1]),
            (-45.0, 2.0 / (f64::SQRT_3 + 1.0), [0, 2, 1]),
            (-30.0, 0.5, [0, 2, 1]),
            (-15.0, (f64::SQRT_3 - 1.0) / (f64::SQRT_3 + 1.0), [0, 2, 1]),
            (0.0, 0.0, [0, 1, 2]),
            (15.0, (f64::SQRT_3 - 1.0) / (f64::SQRT_3 + 1.0), [0, 1, 2]),
            (30.0, 0.5, [0, 1, 2]),
            (45.0, 2.0 / (f64::SQRT_3 + 1.0), [0, 1, 2]),
            (60.0, 1.0, [1, 0, 2]),
            (75.0, 2.0 / (f64::SQRT_3 + 1.0), [1, 0, 2]),
            (90.0, 0.5, [1, 0, 2]),
            (105.0, (f64::SQRT_3 - 1.0) / (f64::SQRT_3 + 1.0), [1, 0, 2]),
            (120.0, 0.0, [1, 2, 0]),
            (135.0, (f64::SQRT_3 - 1.0) / (f64::SQRT_3 + 1.0), [1, 2, 0]),
            (150.0, 0.5, [1, 2, 0]),
            (165.0, 2.0 / (f64::SQRT_3 + 1.0), [1, 2, 0]),
            (180.0, 1.0, [2, 1, 0]),
        ] {
            let hue_angle = Degrees::<f64>::from(*angle);
            let hue_data: HueData<f64> = hue_angle.into();
            assert!(
                hue_data.second.is_proportion(),
                "angle = {:?} hue_date = {:?}",
                hue_angle,
                hue_data
            );
            assert_approx_eq!(hue_data.second, *expected);
            assert_eq!(hue_data.io, *io, "angle = {:?}", hue_angle);
            assert_approx_eq!(hue_data.hue_angle(), hue_angle, 0.000000000000001);
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
                    assert_approx_eq!(other, *expected);
                }
            }
        }
    }

    #[test]
    fn calc_other_from_xy_from_rgb_alt() {
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
                    let other = super::calc_other_from_xy_alt(rgb.xy());
                    assert!(other.is_proportion(), "other = {}", other);
                    assert_approx_eq!(other, *expected);
                }
            }
        }
    }

    #[test]
    fn calc_other_from_xy_from_angle() {
        for (angle, expected) in &[
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
            let other = super::calc_other_from_xy(hue_angle.xy());
            assert!(other.is_proportion(), "other = {}", other);
            assert_approx_eq!(other, *expected, 0.000000000000001);
        }
    }

    #[test]
    fn calc_other_from_xy_from_comparison() {
        for (angle, _expected) in &[
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
            let other_xy = super::calc_other_from_xy(hue_angle.xy());
            let other_hue = super::calc_other_from_angle(hue_angle.abs());
            assert_approx_eq!(other_xy, other_hue, 0.000000000000001);
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
                assert_approx_eq!(max_chroma, *chroma);
                let max_chroma = super::max_chroma_for_sum(*other, tint);
                assert_approx_eq!(max_chroma, *chroma, 0.000000000000001);
            }
        }
    }

    #[test]
    fn primary_max_chroma_rgbs() {
        for io in [[0_u8, 1, 2], [0_u8, 2, 1]].iter() {
            let hue_data = HueData::<f64> {
                second: 0.0,
                io: *io,
            };
            assert_eq!(hue_data.max_chroma_rgb_for_sum(1.0), RGB::RED);
            assert_eq!(hue_data.max_chroma_rgb_for_sum(0.0), RGB::BLACK);
            assert_eq!(hue_data.max_chroma_rgb_for_sum(3.0), RGB::WHITE);
            for sum in [0.0001, 0.25, 0.5, 0.75, 0.9999].iter() {
                let expected: RGB<f64> = [*sum, 0.0, 0.0].into();
                assert_eq!(hue_data.max_chroma_rgb_for_sum(*sum), expected);
            }
            for sum in [1.0001, 1.5, 2.0, 2.5, 2.9999].iter() {
                let expected: RGB<f64> = [1.0, (sum - 1.0) / 2.0, (sum - 1.0) / 2.0].into();
                assert_eq!(hue_data.max_chroma_rgb_for_sum(*sum), expected);
            }
        }
        for io in [[1_u8, 0, 2], [1_u8, 2, 0]].iter() {
            let hue_data = HueData::<f64> {
                second: 0.0,
                io: *io,
            };
            assert_eq!(hue_data.max_chroma_rgb_for_sum(1.0), RGB::GREEN);
            assert_eq!(hue_data.max_chroma_rgb_for_sum(0.0), RGB::BLACK);
            assert_eq!(hue_data.max_chroma_rgb_for_sum(3.0), RGB::WHITE);
            for sum in [0.0001, 0.25, 0.5, 0.75, 0.9999].iter() {
                let expected: RGB<f64> = [0.0, *sum, 0.0].into();
                assert_eq!(hue_data.max_chroma_rgb_for_sum(*sum), expected);
            }
            for sum in [1.0001, 1.5, 2.0, 2.5, 2.9999].iter() {
                let expected: RGB<f64> = [(sum - 1.0) / 2.0, 1.0, (sum - 1.0) / 2.0].into();
                assert_eq!(hue_data.max_chroma_rgb_for_sum(*sum), expected);
            }
        }
        for io in [[2_u8, 0, 1], [2_u8, 1, 0]].iter() {
            let hue_data = HueData::<f64> {
                second: 0.0,
                io: *io,
            };
            assert_eq!(hue_data.max_chroma_rgb_for_sum(1.0), RGB::BLUE);
            assert_eq!(hue_data.max_chroma_rgb_for_sum(0.0), RGB::BLACK);
            assert_eq!(hue_data.max_chroma_rgb_for_sum(3.0), RGB::WHITE);
            for sum in [0.0001, 0.25, 0.5, 0.75, 0.9999].iter() {
                let expected: RGB<f64> = [0.0, 0.0, *sum].into();
                assert_eq!(hue_data.max_chroma_rgb_for_sum(*sum), expected);
            }
            for sum in [1.0001, 1.5, 2.0, 2.5, 2.9999].iter() {
                let expected: RGB<f64> = [(sum - 1.0) / 2.0, (sum - 1.0) / 2.0, 1.0].into();
                assert_eq!(hue_data.max_chroma_rgb_for_sum(*sum), expected);
            }
        }
    }

    #[test]
    fn secondary_max_chroma_rgbs() {
        for io in [[2_u8, 1, 0], [1, 2, 0]].iter() {
            let hue_data = HueData::<f64> {
                second: 1.0,
                io: *io,
            };
            assert_eq!(hue_data.max_chroma_rgb_for_sum(2.0), RGB::CYAN);
            assert_eq!(hue_data.max_chroma_rgb_for_sum(0.0), RGB::BLACK);
            assert_eq!(hue_data.max_chroma_rgb_for_sum(3.0), RGB::WHITE);
            for sum in [0.0001, 0.25, 0.5, 0.75, 1.0, 1.5, 1.9999].iter() {
                let expected: RGB<f64> = [0.0, sum / 2.0, sum / 2.0].into();
                assert_eq!(hue_data.max_chroma_rgb_for_sum(*sum), expected);
            }
            for sum in [2.0001, 2.25, 2.5, 2.75, 2.9999].iter() {
                let expected: RGB<f64> = [sum - 2.0, 1.0, 1.0].into();
                assert_eq!(hue_data.max_chroma_rgb_for_sum(*sum), expected);
            }
        }
        for io in [[2_u8, 0, 1], [0_u8, 2, 1]].iter() {
            let hue_data = HueData::<f64> {
                second: 1.0,
                io: *io,
            };
            assert_eq!(hue_data.max_chroma_rgb_for_sum(2.0), RGB::MAGENTA);
            assert_eq!(hue_data.max_chroma_rgb_for_sum(0.0), RGB::BLACK);
            assert_eq!(hue_data.max_chroma_rgb_for_sum(3.0), RGB::WHITE);
            for sum in [0.0001, 0.25, 0.5, 0.75, 1.0, 1.5, 1.9999].iter() {
                let expected: RGB<f64> = [sum / 2.0, 0.0, sum / 2.0].into();
                assert_eq!(hue_data.max_chroma_rgb_for_sum(*sum), expected);
            }
            for sum in [2.0001, 2.25, 2.5, 2.75, 2.9999].iter() {
                let expected: RGB<f64> = [1.0, sum - 2.0, 1.0].into();
                assert_eq!(hue_data.max_chroma_rgb_for_sum(*sum), expected);
            }
        }
        for io in [[1_u8, 0, 2], [0_u8, 1, 2]].iter() {
            let hue_data = HueData::<f64> {
                second: 1.0,
                io: *io,
            };
            assert_eq!(hue_data.max_chroma_rgb_for_sum(2.0), RGB::YELLOW);
            assert_eq!(hue_data.max_chroma_rgb_for_sum(0.0), RGB::BLACK);
            assert_eq!(hue_data.max_chroma_rgb_for_sum(3.0), RGB::WHITE);
            for sum in [0.0001, 0.25, 0.5, 0.75, 1.0, 1.5, 1.9999].iter() {
                let expected: RGB<f64> = [sum / 2.0, sum / 2.0, 0.0].into();
                assert_eq!(hue_data.max_chroma_rgb_for_sum(*sum), expected);
            }
            for sum in [2.0001, 2.25, 2.5, 2.75, 2.9999].iter() {
                let expected: RGB<f64> = [1.0, 1.0, sum - 2.0].into();
                assert_eq!(hue_data.max_chroma_rgb_for_sum(*sum), expected);
            }
        }
    }

    #[test]
    fn general_max_chroma_rgbs() {
        let ios: [[u8; 3]; 6] = [
            [0, 1, 2],
            [0, 2, 1],
            [1, 0, 2],
            [1, 2, 0],
            [2, 0, 1],
            [2, 1, 0],
        ];
        for io in ios.iter() {
            for second in OTHER_VALUES.iter() {
                let hue_data = HueData::<f64> {
                    second: *second,
                    io: *io,
                };
                assert_eq!(hue_data.max_chroma_rgb_for_sum(0.0), RGB::BLACK);
                assert_eq!(hue_data.max_chroma_rgb_for_sum(3.0), RGB::WHITE);
                for sum in NON_ZERO_SUMS.iter() {
                    let rgb = hue_data.max_chroma_rgb_for_sum(*sum);
                    assert_approx_eq!(rgb.sum(), *sum);
                    let xy = rgb.xy();
                    if xy.0 == 0.0 && xy.1 == 0.0 {
                        assert_approx_eq!(rgb[0], rgb[1]);
                        assert_approx_eq!(rgb[0], rgb[2]);
                    } else {
                        let rgb_other = calc_other_from_xy(xy);
                        assert_approx_eq!(rgb_other, *second, 0.0000000001);
                        let rgb_io = rgb.indices_value_order_u8();
                        assert!(
                            *io == rgb_io
                                || rgb[io[1] as usize] == rgb[io[2] as usize]
                                || rgb[io[0] as usize] == rgb[io[1] as usize],
                            "{:?} == {:?} :: sum: {} other: {} {:?}",
                            *io,
                            rgb_io,
                            *sum,
                            *second,
                            rgb
                        );
                        let chroma_correction = calc_chroma_correction(rgb_other);
                        let rgb_chroma = xy.0.hypot(xy.1) * chroma_correction;
                        let max_chroma = max_chroma_for_sum(*second, *sum);
                        assert_approx_eq!(rgb_chroma, max_chroma, 0.000000000000001);
                    }
                }
            }
        }
    }

    #[test]
    fn general_rgb_for_sum_and_chroma() {
        let ios: [[u8; 3]; 6] = [
            [0, 1, 2],
            [0, 2, 1],
            [1, 0, 2],
            [1, 2, 0],
            [2, 0, 1],
            [2, 1, 0],
        ];
        for io in ios.iter() {
            for other in OTHER_VALUES.iter() {
                let hue_data = HueData::<f64> {
                    second: *other,
                    io: *io,
                };
                assert_eq!(hue_data.rgb_for_sum_and_chroma(0.0, 0.0), Some(RGB::BLACK));
                assert_eq!(hue_data.rgb_for_sum_and_chroma(3.0, 0.0), Some(RGB::WHITE));
                assert!(hue_data.rgb_for_sum_and_chroma(0.0, 1.0).is_none());
                assert!(hue_data.rgb_for_sum_and_chroma(3.0, 1.0).is_none());
                for chroma in NON_ZERO_VALUES.iter() {
                    for sum in NON_ZERO_SUMS.iter() {
                        if let Some(rgb) = hue_data.rgb_for_sum_and_chroma(*sum, *chroma) {
                            assert_approx_eq!(rgb.sum(), *sum, 0.000000000000001);
                            let xy = rgb.xy();
                            if xy.0 == 0.0 && xy.1 == 0.0 {
                                assert_approx_eq!(rgb[0], rgb[1]);
                                assert_approx_eq!(rgb[0], rgb[2]);
                            } else {
                                let rgb_other = calc_other_from_xy(xy);
                                assert_approx_eq!(rgb_other, *other, 0.0000000001);
                                let rgb_io = rgb.indices_value_order_u8();
                                assert!(
                                    *io == rgb_io
                                        || rgb[io[1] as usize] == rgb[io[2] as usize]
                                        || rgb[io[0] as usize] == rgb[io[1] as usize],
                                    "{:?} == {:?} :: sum: {} chroma: {} other: {} {:?}",
                                    *io,
                                    rgb_io,
                                    *sum,
                                    *chroma,
                                    *other,
                                    rgb
                                );
                                let chroma_correction = calc_chroma_correction(rgb_other);
                                let rgb_chroma = xy.0.hypot(xy.1) * chroma_correction;
                                assert_approx_eq!(rgb_chroma, *chroma, 0.000000000000001);
                            }
                        } else {
                            let (shade_sum, tint_sum) = sum_range_for_chroma(*other, *chroma);
                            assert!(
                                *sum < shade_sum || *sum > tint_sum,
                                "{} < {} < {} :: chroma: {} other: {} io: {:?}",
                                shade_sum,
                                *sum,
                                tint_sum,
                                chroma,
                                *other,
                                *io
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn min_max_sum_rgb_for_chroma() {
        assert_eq!(
            HueData::<f64> {
                second: 0.0,
                io: [0, 1, 2]
            }
            .min_sum_rgb_for_chroma(1.0),
            RGB::RED
        );
        assert_eq!(
            HueData::<f64> {
                second: 0.0,
                io: [0, 1, 2],
            }
            .max_sum_rgb_for_chroma(1.0),
            RGB::RED
        );
        assert_eq!(
            HueData::<f64> {
                second: 1.0,
                io: [0, 1, 2]
            }
            .min_sum_rgb_for_chroma(1.0),
            RGB::YELLOW
        );
        assert_eq!(
            HueData::<f64> {
                second: 1.0,
                io: [0, 1, 2],
            }
            .max_sum_rgb_for_chroma(1.0),
            RGB::YELLOW
        );
        let io: [u8; 3] = [0, 1, 2];
        for second in OTHER_VALUES.iter() {
            let hue_data = HueData::<f64> {
                second: *second,
                io: io,
            };
            assert_eq!(hue_data.min_sum_rgb_for_chroma(0.0), RGB::BLACK);
            assert_eq!(hue_data.max_sum_rgb_for_chroma(0.0), RGB::WHITE);
            for chroma in NON_ZERO_VALUES.iter() {
                let shade = hue_data.min_sum_rgb_for_chroma(*chroma);
                let tint = hue_data.max_sum_rgb_for_chroma(*chroma);
                assert!(shade.sum() <= tint.sum());
                assert_approx_eq!(shade.chroma(), *chroma, 0.00000000001);
                assert_approx_eq!(tint.chroma(), *chroma, 0.00000000001);
                assert_approx_eq!(shade.max_chroma_rgb(), tint.max_chroma_rgb(), 0.0000001);
            }
        }
    }
}
