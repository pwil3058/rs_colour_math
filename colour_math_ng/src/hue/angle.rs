// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::cmp::Ordering;
use std::ops::{Add, Neg, Sub};

use crate::{HueConstants, Prop};

#[derive(
    Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default, Debug,
)]
pub struct Angle(i64);

impl Angle {
    pub const DEGREE: Self = Self(i64::MAX / 180);
    pub const MINUTE: Self = Self(Self::DEGREE.0 / 60);
    pub const SECOND: Self = Self(Self::MINUTE.0 / 60);

    pub(crate) const MIN: Self = Self(Self::DEGREE.0 * -180);
    pub(crate) const MAX: Self = Self(Self::DEGREE.0 * 180);

    pub fn asin(arg: Prop) -> Self {
        Self::from(f64::from(arg).asin().to_degrees())
    }

    pub fn sin(self) -> Prop {
        Prop::from(f64::from(self).to_radians().sin())
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

impl HueConstants for Angle {
    const RED: Self = Self(Self::DEGREE.0);
    const GREEN: Self = Self(Self::DEGREE.0 * 120);
    const BLUE: Self = Self(Self::DEGREE.0 * -120);

    const CYAN: Self = Self::MIN;
    const MAGENTA: Self = Self(Self::DEGREE.0 * -60);
    const YELLOW: Self = Self(Self::DEGREE.0 * 60);
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

#[derive(
    Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default, Debug,
)]
pub struct DMS(pub (u16, u8, u8));

impl From<DMS> for Angle {
    fn from(dms: DMS) -> Self {
        debug_assert!(dms.0 .0 <= 360 && dms.0 .1 < 60 && dms.0 .2 < 60);
        let ws: i128 = Self::DEGREE.0 as i128 * dms.0 .0 as i128
            + Self::MINUTE.0 as i128 * dms.0 .1 as i128
            + Self::SECOND.0 as i128 * dms.0 .2 as i128;
        if ws >= Self::MAX.0 as i128 {
            Self((ws - Self::MAX.0 as i128 * 2) as i64)
        } else {
            Self(ws as i64)
        }
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

#[cfg(test)]
mod angle_tests {
    use super::*;
    use num_traits_plus::assert_approx_eq;

    #[test]
    fn convert() {
        assert_approx_eq!(Angle::from(180.0), Angle::from(-180.0), 16);
        assert_approx_eq!(Angle::from(120.0), Angle::from(DMS((120, 0, 0))), 2000);
    }

    #[test]
    fn ops() {
        assert_eq!(
            Angle::from(DMS((120, 0, 0))) + Angle::from(DMS((120, 0, 0))),
            -Angle::from(DMS((120, 0, 0)))
        );
        assert_eq!(
            Angle::from(DMS((120, 0, 0))) - Angle::from(DMS((150, 0, 0))),
            -Angle::from(DMS((30, 0, 0)))
        );
    }

    #[test]
    fn trigonometry() {
        assert_approx_eq!(
            Angle::from(DMS((30, 0, 0))).sin(),
            Prop::from(0.5_f64),
            10000
        );
        assert_approx_eq!(
            Angle::asin(Prop::from(0.5_f64)),
            Angle::from(DMS((30, 0, 0))),
            10000
        );
    }
}
