// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::{
    cmp::Ordering,
    fmt::{self, Debug, Formatter},
    ops::{Add, Neg, Sub},
};

use crate::{fdrn::FDRNumber, HueConstants};

#[derive(Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Angle(i64);

impl Angle {
    pub const MSEC: Self = Self(i64::MAX / (180 * 60 * 60 * 1000));
    pub const SECOND: Self = Self(Self::MSEC.0 * 100);
    pub const MINUTE: Self = Self(Self::SECOND.0 * 60);
    pub const DEGREE: Self = Self(Self::MINUTE.0 * 60);

    pub(crate) const MIN: Self = Self(Self::DEGREE.0 * -180);
    pub(crate) const MAX: Self = Self(Self::DEGREE.0 * 180);

    pub fn asin(arg: FDRNumber) -> Self {
        Self::from(f64::from(arg).asin().to_degrees())
    }

    pub fn cos(self) -> FDRNumber {
        FDRNumber::from(f64::from(self).to_radians().cos())
    }

    pub fn sin(self) -> FDRNumber {
        FDRNumber::from(f64::from(self).to_radians().sin())
    }

    pub fn is_valid(self) -> bool {
        self >= Self::MIN && self < Self::MAX
    }

    pub fn abs_diff(&self, other: &Self) -> Self {
        match self.0.cmp(&other.0) {
            Ordering::Equal => Self(0),
            Ordering::Greater => *self - *other,
            Ordering::Less => *other - *self,
        }
    }

    #[cfg(test)]
    pub fn approx_eq(&self, other: &Self, acceptable_rounding_error: Option<u64>) -> bool {
        if let Some(acceptable_rounding_error) = acceptable_rounding_error {
            self.abs_diff(other).0 < acceptable_rounding_error as i64
        } else {
            self.abs_diff(other).0 < 3
        }
    }
}

impl Debug for Angle {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        //formatter.write_fmt(format_args!("Prop(0.{:08X})", self.0))
        formatter.write_fmt(format_args!("Prop({:?})", f64::from(*self)))
    }
}

impl HueConstants for Angle {
    const RED: Self = Self(Self::DEGREE.0 * 0);
    const GREEN: Self = Self(Self::DEGREE.0 * 120);
    const BLUE: Self = Self(Self::DEGREE.0 * -120);

    const CYAN: Self = Self(Self::DEGREE.0 * -180);
    const MAGENTA: Self = Self(Self::DEGREE.0 * -60);
    const YELLOW: Self = Self(Self::DEGREE.0 * 60);

    const BLUE_CYAN: Self = Self(Self::DEGREE.0 * -150);
    const BLUE_MAGENTA: Self = Self(Self::DEGREE.0 * -90);
    const RED_MAGENTA: Self = Self(Self::DEGREE.0 * -30);
    const RED_YELLOW: Self = Self(Self::DEGREE.0 * 30);
    const GREEN_YELLOW: Self = Self(Self::DEGREE.0 * 90);
    const GREEN_CYAN: Self = Self(Self::DEGREE.0 * 150);
}

impl Neg for Angle {
    type Output = Self;

    fn neg(self) -> Self {
        // NB: - -180 is -180 (wraparound)
        if self == Self::MIN {
            self
        } else {
            Self(self.0.neg())
        }
    }
}

impl Add for Angle {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let ws: i128 = self.0 as i128 + other.0 as i128;
        if ws >= Self::MAX.0 as i128 {
            Self((ws - Self::MAX.0 as i128 * 2) as i64)
        } else if ws < Self::MIN.0 as i128 {
            Self((ws + Self::MAX.0 as i128 * 2) as i64)
        } else {
            Self(ws as i64)
        }
    }
}

impl Sub for Angle {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        let ws: i128 = self.0 as i128 - other.0 as i128;
        if ws >= Self::MAX.0 as i128 {
            Self((ws - Self::MAX.0 as i128 * 2) as i64)
        } else if ws < Self::MIN.0 as i128 {
            Self((ws + Self::MAX.0 as i128 * 2) as i64)
        } else {
            Self(ws as i64)
        }
    }
}

impl From<i16> for Angle {
    fn from(deg: i16) -> Self {
        let mut ws = deg;
        if deg > 0 {
            while ws >= 180 {
                ws -= 360
            }
        } else {
            while ws < -180 {
                ws += 360
            }
        };
        Self(ws as i64 * Self::DEGREE.0)
    }
}

impl From<(u8, u8)> for Angle {
    fn from((deg, min): (u8, u8)) -> Self {
        debug_assert!(deg < 179 && min < 60);
        Self(Self::DEGREE.0 * deg as i64 + Self::MINUTE.0 * min as i64)
    }
}

impl From<(u8, u8, u8)> for Angle {
    fn from((deg, min, sec): (u8, u8, u8)) -> Self {
        debug_assert!(deg < 179 && min < 60 && sec < 60);
        Self(
            Self::DEGREE.0 * deg as i64 + Self::MINUTE.0 * min as i64 + Self::SECOND.0 * sec as i64,
        )
    }
}

impl From<(u8, u8, u8, u16)> for Angle {
    fn from((deg, min, sec, msec): (u8, u8, u8, u16)) -> Self {
        debug_assert!(deg < 179 && min < 60 && sec < 60 && msec < 1000);
        Self(
            Self::DEGREE.0 * deg as i64
                + Self::MINUTE.0 * min as i64
                + Self::SECOND.0 * sec as i64
                + Self::MSEC.0 * msec as i64,
        )
    }
}

impl From<f64> for Angle {
    fn from(float: f64) -> Self {
        debug_assert!(float >= -180.0 && float <= 180.0);
        let ws: i128 = (float * Self::DEGREE.0 as f64) as i128;
        if ws >= Self::MAX.0 as i128 {
            Self((ws - Self::MAX.0 as i128 * 2) as i64)
        } else if ws < Self::MIN.0 as i128 {
            Self((ws + Self::MAX.0 as i128 * 2) as i64)
        } else {
            Self(ws as i64)
        }
    }
}

impl From<Angle> for f64 {
    fn from(angle: Angle) -> Self {
        debug_assert!(angle.is_valid());
        angle.0 as f64 / Angle::DEGREE.0 as f64
    }
}

impl From<Angle> for FDRNumber {
    fn from(angle: Angle) -> Self {
        debug_assert!(angle.is_valid());
        Self(angle.0 as i128 * u64::MAX as i128 / Angle::DEGREE.0 as i128)
    }
}

#[cfg(test)]
mod angle_tests {
    use super::*;
    use num_traits_plus::assert_approx_eq;

    #[test]
    fn convert() {
        assert_eq!(Angle::from(-240).0, Angle::DEGREE.0 * 120);
        assert_eq!(Angle::from(180).0, Angle::DEGREE.0 * -180);
        assert_approx_eq!(Angle::from((120, 45)), Angle::from(120.75), 0x100);
        assert_approx_eq!(Angle::from(180.0), Angle::from(-180.0), 16);
        assert_approx_eq!(Angle::from(120.0), Angle::from(120), 2000);

        assert_eq!(FDRNumber::from(Angle::DEGREE), FDRNumber::ONE);
        assert_eq!(FDRNumber::from(Angle::from(12)), FDRNumber::ONE * 12);
    }

    #[test]
    fn ops() {
        assert_eq!(Angle::from(120) + Angle::from(120), -Angle::from(120));
        assert_eq!(Angle::from(120) - Angle::from(150), -Angle::from(30));
        assert_eq!(
            Angle::from(120) + Angle::from((9, 11, 25, 900)),
            Angle::from((129, 11, 25, 900))
        );
    }

    #[test]
    fn trigonometry() {
        assert_approx_eq!(Angle::from(30).sin(), FDRNumber::from(0.5_f64), 10000);
        assert_approx_eq!(
            Angle::asin(FDRNumber::from(0.5_f64)),
            Angle::from(30),
            10000
        );
    }
}
