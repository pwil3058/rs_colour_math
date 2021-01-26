// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::convert::{TryFrom, TryInto};

use normalised_angles::*;

use crate::{rgb::RGB, ColourComponent, HueConstants, HueIfce, IndicesValueOrder};

pub(crate) fn calc_other_from_angle<F: ColourComponent>(abs_angle: Degrees<F>) -> F {
    if Degrees::PRIMARIES.contains(&abs_angle) {
        F::ZERO
    } else if Degrees::SECONDARIES.contains(&abs_angle) {
        F::ONE
    } else {
        fn f<F: ColourComponent>(angle: Degrees<F>) -> F {
            // Careful of float not fully representing real numbers
            (angle.sin() / (Degrees::GREEN - angle).sin()).min(F::ONE)
        };
        if abs_angle <= Degrees::YELLOW {
            f(abs_angle)
        } else if abs_angle <= Degrees::GREEN {
            f(Degrees::GREEN - abs_angle)
        } else {
            f(abs_angle - Degrees::GREEN)
        }
    }
}

pub fn calc_other_from_xy<F: ColourComponent>(xy: (F, F)) -> F {
    if xy.0.abs() * F::SQRT_3 > xy.1.abs() {
        let divisor = xy.0.abs() * F::SQRT_3 + xy.1.abs();
        debug_assert!(divisor != F::ZERO);
        let x = xy.0 * F::SQRT_3 / divisor;
        if xy.0 >= F::ZERO {
            ((F::ONE - x) * F::TWO).min(F::ONE)
        } else {
            (-(F::TWO * x + F::ONE)).min(F::ONE)
        }
    } else {
        (F::HALF + xy.0 * F::SIN_120 / xy.1.abs()).min(F::ONE)
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
                ((F::ONE - x) * F::TWO).min(F::ONE)
            } else {
                (-(x * F::TWO + F::ONE)).min(F::ONE)
            }
        } else if x_sqrt_3 < xy.1.abs() {
            (F::HALF + xy.0 * F::SIN_120 / xy.1.abs()).min(F::ONE)
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HueData<F: ColourComponent> {
    // TODO: un pub HueData fields
    pub(crate) second: F,
    pub(crate) io: IndicesValueOrder,
}

impl<F: ColourComponent> HueConstants for HueData<F> {
    const RED: Self = Self {
        second: F::ZERO,
        io: IndicesValueOrder::RED,
    };

    const GREEN: Self = Self {
        second: F::ZERO,
        io: IndicesValueOrder::GREEN,
    };

    const BLUE: Self = Self {
        second: F::ZERO,
        io: IndicesValueOrder::BLUE,
    };

    const CYAN: Self = Self {
        second: F::ONE,
        io: IndicesValueOrder::CYAN,
    };

    const MAGENTA: Self = Self {
        second: F::ONE,
        io: IndicesValueOrder::MAGENTA,
    };

    const YELLOW: Self = Self {
        second: F::ONE,
        io: IndicesValueOrder::YELLOW,
    };
}

impl<F: ColourComponent> From<Degrees<F>> for HueData<F> {
    fn from(angle: Degrees<F>) -> Self {
        if angle == Degrees::RED {
            Self::RED
        } else if angle == Degrees::GREEN {
            Self::GREEN
        } else if angle == Degrees::BLUE {
            Self::BLUE
        } else if angle == Degrees::CYAN || angle == -Degrees::CYAN {
            Self::CYAN
        } else if angle == Degrees::MAGENTA {
            Self::MAGENTA
        } else if angle == Degrees::YELLOW {
            Self::YELLOW
        } else {
            fn f<F: ColourComponent>(angle: Degrees<F>) -> F {
                // Careful of float not fully representing real numbers
                (angle.sin() / (Degrees::GREEN - angle).sin()).min(F::ONE)
            };
            if angle >= Degrees::DEG_0 {
                if angle < Degrees::YELLOW {
                    Self {
                        second: f(angle),
                        io: IndicesValueOrder::RED,
                    }
                } else if angle < Degrees::GREEN {
                    Self {
                        second: f(Degrees::GREEN - angle),
                        io: IndicesValueOrder::YELLOW,
                    }
                } else {
                    Self {
                        second: f(angle - Degrees::GREEN),
                        io: IndicesValueOrder::GREEN,
                    }
                }
            } else if angle > Degrees::MAGENTA {
                Self {
                    second: f(-angle),
                    io: IndicesValueOrder::MAGENTA,
                }
            } else if angle > Degrees::BLUE {
                Self {
                    second: f(Degrees::GREEN + angle),
                    io: IndicesValueOrder::BLUE,
                }
            } else {
                Self {
                    second: f(-angle - Degrees::GREEN),
                    io: IndicesValueOrder::CYAN,
                }
            }
        }
    }
}

impl<F: ColourComponent> TryFrom<(F, F)> for HueData<F> {
    type Error = &'static str;

    fn try_from(xy: (F, F)) -> Result<Self, Self::Error> {
        if xy.1 == F::ZERO {
            if xy.0 > F::ZERO {
                Ok(Self::RED)
            } else if xy.0 < F::ZERO {
                Ok(Self::CYAN)
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
                        second: ((F::ONE - x) * F::TWO).min(F::ONE),
                        io: if xy.1 > F::ZERO {
                            IndicesValueOrder::RED //[0, 1, 2].into()
                        } else {
                            IndicesValueOrder::MAGENTA //[0, 2, 1].into()
                        },
                    })
                } else {
                    Ok(Self {
                        second: (-(x * F::TWO + F::ONE)).min(F::ONE),
                        io: if xy.1 > F::ZERO {
                            IndicesValueOrder::GREEN //[1, 2, 0].into()
                        } else {
                            IndicesValueOrder::CYAN //[2, 1, 0].into()
                        },
                    })
                }
            } else if x_sqrt_3 < xy.1.abs() {
                Ok(Self {
                    second: (F::HALF + xy.0 * F::SIN_120 / xy.1.abs()).min(F::ONE),
                    io: if xy.1 > F::ZERO {
                        IndicesValueOrder::YELLOW //[1, 0, 2].into()
                    } else {
                        IndicesValueOrder::BLUE //[2, 0, 1].into()
                    },
                })
            } else if xy.0 > F::ZERO {
                if xy.1 > F::ZERO {
                    Ok(Self::YELLOW)
                } else {
                    Ok(Self::MAGENTA)
                }
            } else {
                if xy.1 > F::ZERO {
                    Ok(Self::GREEN)
                } else {
                    Ok(Self::BLUE)
                }
            }
        }
    }
}

impl<F: ColourComponent> TryFrom<&RGB<F>> for HueData<F> {
    type Error = &'static str;

    fn try_from(rgb: &RGB<F>) -> Result<Self, Self::Error> {
        rgb.xy().try_into()
    }
}

impl<F: ColourComponent> TryFrom<RGB<F>> for HueData<F> {
    type Error = &'static str;

    fn try_from(rgb: RGB<F>) -> Result<Self, Self::Error> {
        rgb.xy().try_into()
    }
}

impl<F: ColourComponent> HueIfce<F> for HueData<F> {
    fn hue_angle(&self) -> Degrees<F> {
        if self.second == F::ZERO {
            match self.io[0] {
                0 => Degrees::RED,
                1 => Degrees::GREEN,
                2 => Degrees::BLUE,
                _ => panic!("illegal colour component index: {}", self.io[0]),
            }
        } else if self.second == F::ONE {
            match self.io[2] {
                0 => Degrees::CYAN,
                1 => Degrees::MAGENTA,
                2 => Degrees::YELLOW,
                _ => panic!("illegal colour component index: {}", self.io[0]),
            }
        } else {
            let sin = F::SQRT_3 * self.second
                / F::TWO
                / (F::ONE - self.second + self.second.powi(2)).sqrt();
            let angle = Degrees::asin(sin);
            match self.io {
                IndicesValueOrder::RED => angle,
                IndicesValueOrder::YELLOW => Degrees::GREEN - angle,
                IndicesValueOrder::GREEN => Degrees::GREEN + angle,
                IndicesValueOrder::MAGENTA => -angle,
                IndicesValueOrder::BLUE => Degrees::BLUE + angle,
                IndicesValueOrder::CYAN => Degrees::BLUE - angle,
                _ => panic!("illegal colour component indices: {:?}", self.io),
            }
        }
    }

    fn chroma_correction(&self) -> F {
        calc_chroma_correction(self.second)
    }

    fn sum_range_for_chroma(&self, chroma: F) -> (F, F) {
        debug_assert!(chroma.is_proportion(), "chroma: {:?}", chroma);
        if chroma == F::ONE {
            (
                (F::ONE + self.second).min(F::TWO),
                (F::ONE + self.second).min(F::TWO),
            )
        } else {
            let temp = self.second * chroma;
            (
                (chroma + temp).min(F::THREE),
                (F::THREE + temp - F::TWO * chroma).min(F::THREE),
            )
        }
    }

    fn max_chroma_for_sum(&self, sum: F) -> F {
        debug_assert!(sum >= F::ZERO && sum <= F::THREE, "sum: {:?}", sum);
        if sum == F::ZERO || sum == F::THREE {
            F::ZERO
        } else if sum < F::ONE + self.second {
            (sum / (F::ONE + self.second)).min(F::ONE)
        } else if sum > F::ONE + self.second {
            ((F::THREE - sum) / (F::TWO - self.second)).min(F::ONE)
        } else {
            F::ONE
        }
    }

    fn max_chroma_rgb(&self) -> RGB<F> {
        let mut array = [F::ZERO, F::ZERO, F::ZERO];
        array[self.io[0] as usize] = F::ONE;
        array[self.io[1] as usize] = self.second;
        array.into()
    }

    fn max_chroma_rgb_for_sum(&self, sum: F) -> RGB<F> {
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
                array[self.io[2] as usize] = (sum - F::TWO).max(F::ZERO).min(F::ONE);
            }
        } else if sum < F::ONE + self.second {
            let divisor = F::ONE + self.second;
            array[self.io[0] as usize] = (sum / divisor).min(F::ONE);
            array[self.io[1] as usize] = sum * self.second / divisor;
        } else if sum > F::ONE + self.second {
            let chroma = (F::THREE - sum) / (F::TWO - self.second);
            let oc = self.second * chroma;
            array[self.io[0] as usize] =
                ((sum + F::TWO * chroma - oc).max(F::ZERO) / F::THREE).min(F::ONE);
            array[self.io[1] as usize] =
                ((sum + F::TWO * oc - chroma).max(F::ZERO) / F::THREE).min(F::ONE);
            array[self.io[2] as usize] = ((sum - oc - chroma).max(F::ZERO) / F::THREE).min(F::ONE);
        } else {
            array[self.io[0] as usize] = F::ONE;
            array[self.io[1] as usize] = self.second;
        };
        array.into()
    }

    fn min_sum_rgb_for_chroma(&self, chroma: F) -> RGB<F> {
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
            array[self.io[1] as usize] = (chroma * self.second).min(F::ONE);
        };
        array.into()
    }

    fn max_sum_rgb_for_chroma(&self, chroma: F) -> RGB<F> {
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
            array[self.io[1] as usize] =
                (chroma * self.second + array[self.io[2] as usize]).min(F::ONE);
        };
        array.into()
    }

    fn rgb_for_sum_and_chroma(&self, sum: F, chroma: F) -> Option<RGB<F>> {
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
            debug_assert!(array[self.io[0] as usize] >= array[self.io[1] as usize]);
            debug_assert!(array[self.io[2] as usize] <= array[self.io[1] as usize]);
            // NB: because floats only approximate real numbers trying to
            // set chroma too small (but non zero) results in a drift
            // in the hue angle of the resulting RGB. When this
            // happens we go straight to a zero chroma RGB
            if chroma < F::from(0.00001).unwrap() && chroma > F::ZERO {
                let rgb: RGB<F> = array.into();
                let xy: (F, F) = rgb.xy();
                if !(xy.0 == F::ZERO && xy.1 == F::ZERO) {
                    let rgb_second = calc_other_from_xy_alt(xy);
                    // deviation "second" indicates a drift in the hue
                    if (rgb_second - self.second).abs() / rgb_second
                        > F::from(0.000_000_000_1).unwrap()
                    {
                        let value = (sum / F::THREE).min(F::ONE);
                        array = [value, value, value];
                    }
                }
            };
        }
        Some(array.into())
    }
}

#[cfg(test)]
mod test {
    use crate::chroma::{calc_chroma_correction, calc_other_from_xy, HueData};
    use crate::rgb::*;
    use crate::{ColourComponent, HueConstants, HueIfce, RGBConstants};
    use normalised_angles::Degrees;
    use num_traits_plus::{assert_approx_eq, float_plus::*};
    use std::convert::TryFrom;

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
            (-180.0, 1.0, [2, 1, 0].into()),
            (-165.0, 2.0 / (f64::SQRT_3 + 1.0), [2, 1, 0].into()),
            (-150.0, 0.5, [2, 1, 0].into()),
            (
                -135.0,
                (f64::SQRT_3 - 1.0) / (f64::SQRT_3 + 1.0),
                [2, 1, 0].into(),
            ),
            (-120.0, 0.0, [2, 0, 1].into()),
            (
                -105.0,
                (f64::SQRT_3 - 1.0) / (f64::SQRT_3 + 1.0),
                [2, 0, 1].into(),
            ),
            (-90.0, 0.5, [2, 0, 1].into()),
            (-75.0, 2.0 / (f64::SQRT_3 + 1.0), [2, 0, 1].into()),
            (-60.0, 1.0, [0, 2, 1].into()),
            (-45.0, 2.0 / (f64::SQRT_3 + 1.0), [0, 2, 1].into()),
            (-30.0, 0.5, [0, 2, 1].into()),
            (
                -15.0,
                (f64::SQRT_3 - 1.0) / (f64::SQRT_3 + 1.0),
                [0, 2, 1].into(),
            ),
            (0.0, 0.0, [0, 1, 2].into()),
            (
                15.0,
                (f64::SQRT_3 - 1.0) / (f64::SQRT_3 + 1.0),
                [0, 1, 2].into(),
            ),
            (30.0, 0.5, [0, 1, 2].into()),
            (45.0, 2.0 / (f64::SQRT_3 + 1.0), [0, 1, 2].into()),
            (60.0, 1.0, [1, 0, 2].into()),
            (75.0, 2.0 / (f64::SQRT_3 + 1.0), [1, 0, 2].into()),
            (90.0, 0.5, [1, 0, 2].into()),
            (
                105.0,
                (f64::SQRT_3 - 1.0) / (f64::SQRT_3 + 1.0),
                [1, 0, 2].into(),
            ),
            (120.0, 0.0, [1, 2, 0].into()),
            (
                135.0,
                (f64::SQRT_3 - 1.0) / (f64::SQRT_3 + 1.0),
                [1, 2, 0].into(),
            ),
            (150.0, 0.5, [1, 2, 0].into()),
            (165.0, 2.0 / (f64::SQRT_3 + 1.0), [1, 2, 0].into()),
            (180.0, 1.0, [2, 1, 0].into()),
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
    fn hue_data_from_xy() {
        assert_eq!(
            HueData::<f64>::try_from(RGB::<f64>::RED.xy()),
            Ok(HueData::<f64>::RED)
        );
        assert_eq!(
            HueData::<f64>::try_from(RGB::<f64>::GREEN.xy()),
            Ok(HueData::<f64>::GREEN)
        );
        assert_eq!(
            HueData::<f64>::try_from(RGB::<f64>::BLUE.xy()),
            Ok(HueData::<f64>::BLUE)
        );
        assert_eq!(
            HueData::<f64>::try_from(RGB::<f64>::CYAN.xy()),
            Ok(HueData::<f64>::CYAN)
        );
        assert_eq!(
            HueData::<f64>::try_from(RGB::<f64>::YELLOW.xy()),
            Ok(HueData::<f64>::YELLOW)
        );
        assert_eq!(
            HueData::<f64>::try_from(RGB::<f64>::MAGENTA.xy()),
            Ok(HueData::<f64>::MAGENTA)
        );
        assert!(HueData::<f64>::try_from(RGB::<f64>::BLACK.xy()).is_err());
        assert!(HueData::<f64>::try_from(RGB::<f64>::WHITE.xy()).is_err());
        for (array, expected) in &[
            ([0.9, 0.5, 0.1], IndicesValueOrder::RED),
            ([0.9, 0.5, 0.5], IndicesValueOrder::RED),
            ([0.5, 0.9, 0.1], IndicesValueOrder::YELLOW),
            ([0.5, 0.9, 0.50000001], IndicesValueOrder::GREEN), // inexactness of floating point
            ([0.1, 0.5, 0.9], IndicesValueOrder::CYAN),
            ([0.5, 0.5, 0.9], IndicesValueOrder::BLUE),
        ] {
            assert_eq!(
                HueData::<f64>::try_from(RGB::<f64>::from(array).xy())
                    .unwrap()
                    .io,
                *expected
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
            let hue_data = HueData::<f64> {
                second: *other,
                io: IndicesValueOrder::default(),
            };
            assert_eq!(
                hue_data.sum_range_for_chroma(0.0),
                (0.0, 3.0),
                "other: {}",
                other
            );
            assert_eq!(
                hue_data.sum_range_for_chroma(1.0),
                (1.0 + *other, 1.0 + *other),
                "other: {}",
                other
            );
            for chroma in NON_ZERO_VALUES.iter() {
                let (shade, tint) = hue_data.sum_range_for_chroma(*chroma);
                let max_chroma = hue_data.max_chroma_for_sum(shade);
                assert_approx_eq!(max_chroma, *chroma);
                let max_chroma = hue_data.max_chroma_for_sum(tint);
                assert_approx_eq!(max_chroma, *chroma, 0.000000000000001);
            }
        }
    }

    #[test]
    fn primary_max_chroma_rgbs() {
        for (hue_data, expected_rgb) in HueData::<f64>::PRIMARIES
            .iter()
            .zip(RGB::<f64>::PRIMARIES.iter())
        {
            assert_eq!(hue_data.max_chroma_rgb_for_sum(1.0), *expected_rgb);
            assert_eq!(hue_data.max_chroma_rgb_for_sum(0.0), RGB::BLACK);
            assert_eq!(hue_data.max_chroma_rgb_for_sum(3.0), RGB::WHITE);
            for sum in [0.0001, 0.25, 0.5, 0.75, 0.9999].iter() {
                let mut array = [0.0_f64, 0.0, 0.0];
                array[hue_data.io[0] as usize] = *sum;
                let expected: RGB<f64> = array.into();
                assert_eq!(hue_data.max_chroma_rgb_for_sum(*sum), expected);
            }
            for sum in [2.0001, 2.25, 2.5, 2.75, 2.9999].iter() {
                let mut array = [1.0_f64, 1.0, 1.0];
                array[hue_data.io[1] as usize] = (sum - 1.0) / 2.0;
                array[hue_data.io[2] as usize] = (sum - 1.0) / 2.0;
                let expected: RGB<f64> = array.into();
                assert_eq!(hue_data.max_chroma_rgb_for_sum(*sum), expected);
            }
        }
    }

    #[test]
    fn secondary_max_chroma_rgbs() {
        for (hue_data, expected_rgb) in HueData::<f64>::SECONDARIES
            .iter()
            .zip(RGB::<f64>::SECONDARIES.iter())
        {
            assert_eq!(hue_data.max_chroma_rgb_for_sum(2.0), *expected_rgb);
            assert_eq!(hue_data.max_chroma_rgb_for_sum(0.0), RGB::BLACK);
            assert_eq!(hue_data.max_chroma_rgb_for_sum(3.0), RGB::WHITE);
            for sum in [0.0001, 0.25, 0.5, 0.75, 1.0, 1.5, 1.9999].iter() {
                let mut array = [0.0_f64, 0.0, 0.0];
                array[hue_data.io[0] as usize] = sum / 2.0;
                array[hue_data.io[1] as usize] = sum / 2.0;
                let expected: RGB<f64> = array.into();
                assert_eq!(hue_data.max_chroma_rgb_for_sum(*sum), expected);
            }
            for sum in [2.0001, 2.25, 2.5, 2.75, 2.9999].iter() {
                let mut array = [1.0_f64, 1.0, 1.0];
                array[hue_data.io[2] as usize] = sum - 2.0;
                let expected: RGB<f64> = array.into();
                assert_eq!(hue_data.max_chroma_rgb_for_sum(*sum), expected);
            }
        }
    }

    #[test]
    fn general_max_chroma_rgbs() {
        for io in IndicesValueOrder::PRIMARIES
            .iter()
            .chain(IndicesValueOrder::SECONDARIES.iter())
        {
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
                        let rgb_io = rgb.indices_value_order();
                        assert!(
                            rgb_io == *io || rgb[io[1]] == rgb[io[2]] || rgb[io[0]] == rgb[io[1]],
                            "{:?} == {:?} :: sum: {} other: {} {:?}",
                            *io,
                            rgb_io,
                            *sum,
                            *second,
                            rgb
                        );
                        let chroma_correction = hue_data.chroma_correction();
                        let rgb_chroma = xy.0.hypot(xy.1) * chroma_correction;
                        let max_chroma = hue_data.max_chroma_for_sum(*sum);
                        assert_approx_eq!(rgb_chroma, max_chroma, 0.000000000000001);
                    }
                }
            }
        }
    }

    #[test]
    fn general_rgb_for_sum_and_chroma() {
        for io in IndicesValueOrder::PRIMARIES
            .iter()
            .chain(IndicesValueOrder::SECONDARIES.iter())
        {
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
                                let rgb_io = rgb.indices_value_order();
                                assert!(
                                    rgb_io == *io
                                        || rgb[io[1]] == rgb[io[2]]
                                        || rgb[io[0]] == rgb[io[1]],
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
                            let (shade_sum, tint_sum) = hue_data.sum_range_for_chroma(*chroma);
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
        for (hue_data, expected_rgb) in HueData::<f64>::PRIMARIES
            .iter()
            .zip(RGB::<f64>::PRIMARIES.iter())
        {
            assert_eq!(hue_data.min_sum_rgb_for_chroma(1.0), *expected_rgb);
            assert_eq!(hue_data.max_sum_rgb_for_chroma(1.0), *expected_rgb);
        }
        for (hue_data, expected_rgb) in HueData::<f64>::SECONDARIES
            .iter()
            .zip(RGB::<f64>::SECONDARIES.iter())
        {
            assert_eq!(hue_data.min_sum_rgb_for_chroma(1.0), *expected_rgb);
            assert_eq!(hue_data.max_sum_rgb_for_chroma(1.0), *expected_rgb);
        }
        for io in IndicesValueOrder::PRIMARIES
            .iter()
            .chain(IndicesValueOrder::SECONDARIES.iter())
        {
            for second in OTHER_VALUES.iter() {
                let hue_data = HueData::<f64> {
                    second: *second,
                    io: *io,
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
}
