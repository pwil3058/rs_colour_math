// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::{
    cmp::Ordering,
    convert::TryFrom,
    ops::{Add, Sub},
};

use crate::hue::{CMYHue, RGBHue, Sextant};
use crate::{
    fdrn::UFDRNumber, hue::HueIfce, proportion::Warmth, rgb::RGB, Angle, Chroma, ColourBasics, Hue,
    HueConstants, LightLevel, ManipulatedColour, Prop, RGBConstants,
};

#[derive(Debug, Clone, Copy, Eq, Ord, Default, Serialize, Deserialize)]
pub struct HCV {
    pub(crate) hue: Option<Hue>,
    pub(crate) chroma: Chroma,
    pub(crate) sum: UFDRNumber,
}

impl PartialEq for HCV {
    fn eq(&self, rhs: &Self) -> bool {
        if self.sum != rhs.sum {
            false
        } else if self.chroma != rhs.chroma {
            false
        } else {
            match self.chroma {
                Chroma::ZERO => true,
                _ => self.hue == rhs.hue,
            }
        }
    }
}

impl PartialOrd for HCV {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        match self.chroma {
            Chroma::ZERO => {
                if rhs.chroma != Chroma::ZERO {
                    Some(Ordering::Less)
                } else {
                    match self.chroma.cmp(&rhs.chroma) {
                        Ordering::Equal => self.sum.partial_cmp(&rhs.sum),
                        ordering => Some(ordering),
                    }
                }
            }
            _ => match self.hue.cmp(&rhs.hue) {
                Ordering::Equal => match self.chroma.cmp(&rhs.chroma) {
                    Ordering::Equal => self.sum.partial_cmp(&rhs.sum),
                    ordering => Some(ordering),
                },
                ordering => Some(ordering),
            },
        }
    }
}

impl HCV {
    pub(crate) fn new(hue_data: Option<(Hue, Chroma)>, sum: UFDRNumber) -> Self {
        if let Some((hue, chroma)) = hue_data {
            debug_assert!(hue.sum_and_chroma_are_compatible(sum, chroma));
            Self {
                hue: Some(hue),
                chroma,
                sum,
            }
        } else {
            debug_assert!(sum.is_valid_sum());
            Self {
                hue: None,
                chroma: Chroma::ZERO,
                sum,
            }
        }
    }

    pub(crate) fn new_grey_sum(sum: UFDRNumber) -> Self {
        Self {
            hue: None,
            chroma: Chroma::ZERO,
            sum,
        }
    }

    pub fn new_grey(value: Prop) -> Self {
        Self::new_grey_sum(value * 3)
    }

    pub fn is_grey(&self) -> bool {
        self.chroma == Chroma::ZERO
    }

    pub(crate) fn is_valid(&self) -> bool {
        if let Some(hue) = self.hue {
            if hue.sum_and_chroma_are_compatible(self.sum, self.chroma) {
                true
            } else {
                false
            }
        } else {
            if self.chroma == Chroma::ZERO && self.sum <= UFDRNumber::THREE && self.sum.0 % 3 == 0 {
                true
            } else {
                false
            }
        }
    }

    pub fn sum_range_for_current_chroma(&self) -> (UFDRNumber, UFDRNumber) {
        if let Some(hue) = self.hue {
            if let Some(range) = hue.sum_range_for_chroma(self.chroma) {
                range
            } else {
                (UFDRNumber::ZERO, UFDRNumber::THREE)
            }
        } else {
            (UFDRNumber::ZERO, UFDRNumber::THREE)
        }
    }

    pub fn sum_range_for_current_chroma_prop(&self) -> (UFDRNumber, UFDRNumber) {
        if let Some(hue) = self.hue {
            if let Some(range) = hue.sum_range_for_chroma_prop(self.chroma.prop()) {
                (range.min, range.max)
            } else {
                (UFDRNumber::ZERO, UFDRNumber::THREE)
            }
        } else {
            (UFDRNumber::ZERO, UFDRNumber::THREE)
        }
    }

    pub fn max_chroma_for_current_sum(&self) -> Chroma {
        if let Some(hue) = self.hue {
            if let Some(max_chroma) = hue.max_chroma_for_sum(self.sum) {
                max_chroma
            } else {
                Chroma::ZERO
            }
        } else {
            Chroma::ZERO
        }
    }
}

impl HueConstants for HCV {
    const RED: Self = Self {
        hue: Some(Hue::RED),
        chroma: Chroma::ONE,
        sum: UFDRNumber::ONE,
    };

    const GREEN: Self = Self {
        hue: Some(Hue::GREEN),
        chroma: Chroma::ONE,
        sum: UFDRNumber::ONE,
    };

    const BLUE: Self = Self {
        hue: Some(Hue::BLUE),
        chroma: Chroma::ONE,
        sum: UFDRNumber::ONE,
    };

    const CYAN: Self = Self {
        hue: Some(Hue::CYAN),
        chroma: Chroma::ONE,
        sum: UFDRNumber::TWO,
    };

    const MAGENTA: Self = Self {
        hue: Some(Hue::MAGENTA),
        chroma: Chroma::ONE,
        sum: UFDRNumber::TWO,
    };

    const YELLOW: Self = Self {
        hue: Some(Hue::YELLOW),
        chroma: Chroma::ONE,
        sum: UFDRNumber::TWO,
    };

    const BLUE_CYAN: Self = Self {
        hue: Some(Hue::BLUE_CYAN),
        chroma: Chroma::ONE,
        sum: UFDRNumber::ONE_PT_5,
    };

    const BLUE_MAGENTA: Self = Self {
        hue: Some(Hue::BLUE_MAGENTA),
        chroma: Chroma::ONE,
        sum: UFDRNumber::ONE_PT_5,
    };

    const RED_MAGENTA: Self = Self {
        hue: Some(Hue::RED_MAGENTA),
        chroma: Chroma::ONE,
        sum: UFDRNumber::ONE_PT_5,
    };

    const RED_YELLOW: Self = Self {
        hue: Some(Hue::RED_YELLOW),
        chroma: Chroma::ONE,
        sum: UFDRNumber::ONE_PT_5,
    };

    const GREEN_YELLOW: Self = Self {
        hue: Some(Hue::GREEN_YELLOW),
        chroma: Chroma::ONE,
        sum: UFDRNumber::ONE_PT_5,
    };

    const GREEN_CYAN: Self = Self {
        hue: Some(Hue::GREEN_CYAN),
        chroma: Chroma::ONE,
        sum: UFDRNumber::ONE_PT_5,
    };
}

impl RGBConstants for HCV {
    const WHITE: Self = Self {
        hue: None,
        chroma: Chroma::ZERO,
        sum: UFDRNumber::THREE,
    };

    const BLACK: Self = Self {
        hue: None,
        chroma: Chroma::ZERO,
        sum: UFDRNumber::ZERO,
    };
}

fn sum_and_chroma((hue, [red, green, blue]): (Hue, [Prop; 3])) -> (UFDRNumber, Chroma) {
    let sum = red + green + blue;
    let prop = match hue {
        Hue::Primary(RGBHue::Red) => red - blue,
        Hue::Primary(RGBHue::Green) => green - red,
        Hue::Primary(RGBHue::Blue) => blue - green,
        Hue::Secondary(CMYHue::Cyan) => blue - red,
        Hue::Secondary(CMYHue::Magenta) => red - green,
        Hue::Secondary(CMYHue::Yellow) => green - blue,
        Hue::Sextant(sextant_hue) => match sextant_hue.sextant() {
            Sextant::RedYellow => red - blue,
            Sextant::RedMagenta => red - green,
            Sextant::GreenYellow => green - blue,
            Sextant::GreenCyan => green - red,
            Sextant::BlueCyan => blue - red,
            Sextant::BlueMagenta => blue - green,
        },
    };
    let chroma = match prop {
        Prop::ZERO => Chroma::ZERO,
        Prop::ONE => Chroma::ONE,
        prop => match sum.cmp(&hue.sum_for_max_chroma()) {
            Ordering::Greater => Chroma::Tint(prop),
            Ordering::Less => Chroma::Shade(prop),
            Ordering::Equal => Chroma::Neither(prop),
        },
    };
    (sum, chroma)
}

impl From<[Prop; 3]> for HCV {
    fn from(array: [Prop; 3]) -> Self {
        if let Ok(hue) = Hue::try_from(array) {
            let (sum, chroma) = sum_and_chroma((hue, array));
            Self {
                hue: Some(hue),
                chroma,
                sum,
            }
        } else {
            Self {
                hue: None,
                chroma: Chroma::ZERO,
                sum: array[0] + array[1] + array[2],
            }
        }
    }
}

impl From<&[Prop; 3]> for HCV {
    fn from(array: &[Prop; 3]) -> Self {
        HCV::from(*array)
    }
}

impl From<HCV> for [Prop; 3] {
    fn from(hcv: HCV) -> Self {
        debug_assert!(hcv.is_valid());
        if let Some(hue) = hcv.hue {
            hue.array_for_sum_and_chroma(hcv.sum, hcv.chroma)
                .expect("Invalid Hue")
        } else {
            let value: Prop = (hcv.sum / 3).into();
            [value, value, value]
        }
    }
}

impl<L: LightLevel> From<&RGB<L>> for HCV {
    fn from(rgb: &RGB<L>) -> Self {
        Self::from(<[Prop; 3]>::from(*rgb))
    }
}

impl<L: LightLevel> From<RGB<L>> for HCV {
    fn from(rgb: RGB<L>) -> Self {
        HCV::from(&rgb)
    }
}

impl<L: LightLevel> From<HCV> for RGB<L> {
    fn from(hcv: HCV) -> Self {
        RGB::<L>::from(<[Prop; 3]>::from(hcv))
    }
}

impl<L: LightLevel> From<&HCV> for RGB<L> {
    fn from(hcv: &HCV) -> Self {
        RGB::<L>::from(*hcv)
    }
}

impl ColourBasics for HCV {
    fn hue(&self) -> Option<Hue> {
        self.hue
    }

    fn hue_angle(&self) -> Option<Angle> {
        if let Some(hue) = self.hue {
            Some(hue.angle())
        } else {
            None
        }
    }

    fn is_grey(&self) -> bool {
        self.chroma == Chroma::ZERO
    }

    fn chroma(&self) -> Chroma {
        self.chroma
    }

    fn value(&self) -> Prop {
        (self.sum / 3).into()
    }

    fn warmth(&self) -> Warmth {
        if let Some(hue) = self.hue {
            hue.warmth_for_chroma(self.chroma)
        } else {
            Warmth::calculate_monochrome_fm_sum(self.sum)
        }
    }

    fn rgb<L: LightLevel>(&self) -> RGB<L> {
        self.into()
    }

    fn hcv(&self) -> HCV {
        *self
    }
}

impl ManipulatedColour for HCV {
    fn lightened(&self, prop: Prop) -> Self {
        let rgb = RGB::<u64>::from(self).lightened(prop);
        HCV::from(rgb)
    }

    fn darkened(&self, prop: Prop) -> Self {
        let rgb = RGB::<u64>::from(self).darkened(prop);
        HCV::from(rgb)
    }
    fn saturated(&self, prop: Prop) -> Self {
        if let Some(hue) = self.hue {
            let new_chroma = match self.chroma {
                Chroma::Shade(c_prop) => Chroma::Shade((c_prop - c_prop * prop + prop).into()),
                Chroma::Tint(c_prop) => Chroma::Tint((c_prop - c_prop * prop + prop).into()),
                Chroma::Neither(c_prop) => match prop {
                    Prop::ONE => Chroma::ONE,
                    prop => Chroma::Neither((c_prop - c_prop * prop + prop).into()),
                },
            };
            let new_sum = if let Some((min_sum, max_sum)) = hue.sum_range_for_chroma(new_chroma) {
                if self.sum < min_sum {
                    min_sum
                } else if self.sum > max_sum {
                    max_sum
                } else {
                    self.sum
                }
            } else {
                self.sum
            };
            Self {
                hue: Some(hue),
                chroma: new_chroma,
                sum: new_sum,
            }
        } else {
            *self
        }
    }

    fn greyed(&self, prop: Prop) -> Self {
        if let Some(hue) = self.hue {
            let new_chroma = match self.chroma {
                Chroma::Shade(c_prop) => Chroma::Shade((c_prop - c_prop * prop).into()),
                Chroma::Tint(c_prop) => Chroma::Tint((c_prop - c_prop * prop).into()),
                Chroma::Neither(c_prop) => match prop {
                    Prop::ONE => Chroma::from((Prop::ONE - prop, hue, self.sum)),
                    prop => Chroma::Neither((c_prop - c_prop * prop).into()),
                },
            };
            let new_sum = if let Some((min_sum, max_sum)) = hue.sum_range_for_chroma(new_chroma) {
                if self.sum < min_sum {
                    min_sum
                } else if self.sum > max_sum {
                    max_sum
                } else {
                    self.sum
                }
            } else {
                self.sum
            };
            Self {
                hue: self.hue,
                chroma: new_chroma,
                sum: new_sum,
            }
        } else {
            *self
        }
    }

    fn rotated(&self, angle: Angle) -> Self {
        *self + angle
    }
}

impl Add<Angle> for HCV {
    type Output = Self;

    fn add(self, angle: Angle) -> Self {
        if let Some(hue) = self.hue {
            let new_hue = hue + angle;
            if let Some((sum, chroma)) = new_hue
                .adjusted_sum_and_chroma_for_chroma_compatibility(self.chroma.prop(), self.sum)
            {
                HCV::new(Some((new_hue, chroma)), sum)
            } else {
                HCV::new(None, self.sum)
            }
        } else {
            self
        }
    }
}

impl Sub<Angle> for HCV {
    type Output = Self;

    fn sub(self, angle: Angle) -> Self {
        if let Some(hue) = self.hue {
            let new_hue = hue - angle;
            if let Some((sum, chroma)) = new_hue
                .adjusted_sum_and_chroma_for_chroma_compatibility(self.chroma.prop(), self.sum)
            {
                HCV::new(Some((new_hue, chroma)), sum)
            } else {
                HCV::new(None, self.sum)
            }
        } else {
            self
        }
    }
}

#[cfg(test)]
mod hcv_tests;
