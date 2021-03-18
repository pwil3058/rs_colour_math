// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
#[macro_use]
extern crate serde_derive;

use std::fmt::{Debug, LowerExp, LowerHex, UpperExp, UpperHex};

use num_traits::{Signed, Unsigned};
use num_traits_plus::float_plus::*;

pub mod attributes;
pub mod beigui;
pub mod debug;
pub mod fdrn;
pub mod hcv;
pub mod hue;
pub mod manipulator;
pub mod mixing;
pub mod rgb;

pub use crate::{
    attributes::{Chroma, Greyness, Value, Warmth},
    fdrn::{Prop, UFDRNumber},
    hcv::HCV,
    hue::{angle::Angle, Hue},
    rgb::RGB,
};
use hue::HueIfce;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CCI {
    Red,
    Green,
    Blue,
}

impl From<usize> for CCI {
    fn from(index: usize) -> Self {
        match index {
            0 => CCI::Red,
            1 => CCI::Green,
            2 => CCI::Blue,
            _ => panic!("Illegal usize -> CCI"),
        }
    }
}

pub trait Float: FloatPlus + std::iter::Sum + FloatApproxEq<Self> {}

impl Float for f32 {}
impl Float for f64 {}

pub trait LightLevel:
    Clone + Copy + From<Prop> + Into<Prop> + PartialEq + Debug + Default + PartialOrd
{
    const ZERO: Self;
    const ONE: Self;
    const HALF: Self;
}

impl LightLevel for f32 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const HALF: Self = 0.5;
}
impl LightLevel for f64 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const HALF: Self = 0.5;
}

impl LightLevel for u8 {
    const ZERO: Self = 0;
    const ONE: Self = u8::MAX;
    const HALF: Self = u8::MAX / 2;
}

impl LightLevel for u16 {
    const ZERO: Self = 0;
    const ONE: Self = u16::MAX;
    const HALF: Self = u16::MAX / 2;
}

impl LightLevel for u32 {
    const ZERO: Self = 0;
    const ONE: Self = u32::MAX;
    const HALF: Self = u32::MAX / 2;
}

impl LightLevel for u64 {
    const ZERO: Self = 0;
    const ONE: Self = u64::MAX;
    const HALF: Self = u64::MAX / 2;
}

pub trait FloatLightLevel: LightLevel + Signed + Float + LowerExp + UpperExp {}

impl FloatLightLevel for f32 {}
impl FloatLightLevel for f64 {}

pub trait UnsignedLightLevel: LightLevel + Unsigned + Ord + Eq + UpperHex + LowerHex {}

impl UnsignedLightLevel for u8 {}
impl UnsignedLightLevel for u16 {}
impl UnsignedLightLevel for u32 {}
impl UnsignedLightLevel for u64 {}

pub trait HueConstants: Sized + Copy {
    const RED: Self;
    const GREEN: Self;
    const BLUE: Self;

    const CYAN: Self;
    const MAGENTA: Self;
    const YELLOW: Self;

    const BLUE_CYAN: Self;
    const BLUE_MAGENTA: Self;
    const RED_MAGENTA: Self;
    const RED_YELLOW: Self;
    const GREEN_YELLOW: Self;
    const GREEN_CYAN: Self;

    const PRIMARIES: [Self; 3] = [Self::BLUE, Self::RED, Self::GREEN];
    const SECONDARIES: [Self; 3] = [Self::CYAN, Self::MAGENTA, Self::YELLOW];
    const IN_BETWEENS: [Self; 6] = [
        Self::BLUE_CYAN,
        Self::BLUE_MAGENTA,
        Self::RED_MAGENTA,
        Self::RED_YELLOW,
        Self::GREEN_YELLOW,
        Self::GREEN_CYAN,
    ];
}

pub trait RGBConstants: HueConstants + Copy {
    const WHITE: Self;
    const BLACK: Self;

    const GREYS: [Self; 2] = [Self::BLACK, Self::WHITE];
}

pub trait ColourBasics {
    fn hue(&self) -> Option<Hue>;

    fn hue_angle(&self) -> Option<Angle> {
        Some(self.hue()?.angle())
    }

    fn hue_rgb<L: LightLevel>(&self) -> Option<RGB<L>> {
        Some(self.hue()?.max_chroma_rgb())
    }
    fn hue_hcv(&self) -> Option<HCV> {
        Some(self.hue()?.max_chroma_hcv())
    }

    fn is_grey(&self) -> bool {
        self.chroma() == Chroma::ZERO
    }

    fn chroma(&self) -> Chroma;
    fn value(&self) -> Value;

    fn greyness(&self) -> Greyness {
        self.chroma().into()
    }

    fn warmth(&self) -> Warmth {
        if let Some(hue) = self.hue() {
            hue.warmth_for_chroma(self.chroma())
        } else {
            Warmth::calculate_monochrome(self.value())
        }
    }

    fn hcv(&self) -> HCV;
    fn rgb<L: LightLevel>(&self) -> RGB<L>;

    fn monochrome_hcv(&self) -> HCV {
        HCV::new_grey(self.value())
    }

    fn monochrome_rgb<L: LightLevel>(&self) -> RGB<L> {
        RGB::<L>::new_grey(self.value())
    }

    fn best_foreground(&self) -> HCV {
        match self.chroma() {
            Chroma::Shade(_) => HCV::WHITE,
            Chroma::Tint(_) => HCV::BLACK,
            _ => {
                if self.value() < Value::ONE / 2 {
                    HCV::WHITE
                } else {
                    HCV::BLACK
                }
            }
        }
    }

    fn pango_string(&self) -> String {
        let rgb = self.rgb::<u8>();
        format!("#{:02X}{:02X}{:02X}", rgb.0[0], rgb.0[1], rgb.0[2])
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum ScalarAttribute {
    Chroma,
    Greyness,
    Value,
    Warmth,
}

impl std::fmt::Display for ScalarAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ScalarAttribute::Chroma => write!(f, "Chroma"),
            ScalarAttribute::Greyness => write!(f, "Greyness"),
            ScalarAttribute::Value => write!(f, "Value"),
            ScalarAttribute::Warmth => write!(f, "Warmth"),
        }
    }
}

pub trait ColourAttributes: ColourBasics {
    fn scalar_attribute(&self, attr: ScalarAttribute) -> Prop {
        match attr {
            ScalarAttribute::Chroma => self.chroma().prop(),
            ScalarAttribute::Greyness => Prop::ONE - self.chroma().prop(),
            ScalarAttribute::Value => self.value().into(),
            ScalarAttribute::Warmth => self.warmth().into(),
        }
    }

    fn scalar_attribute_rgb<T: LightLevel>(&self, attr: ScalarAttribute) -> RGB<T> {
        match attr {
            ScalarAttribute::Chroma => self.rgb(),
            ScalarAttribute::Greyness => self.rgb(),
            ScalarAttribute::Value => RGB::<T>::new_grey(self.value()),
            ScalarAttribute::Warmth => RGB::<T>::new_warmth_rgb(self.warmth()),
        }
    }
}

impl ColourAttributes for HCV {}
impl<L: LightLevel> ColourAttributes for RGB<L> {}

pub trait ColourIfce: ColourBasics + ColourAttributes {}

impl ColourIfce for HCV {}
impl<L: LightLevel> ColourIfce for RGB<L> {}

pub trait ManipulatedColour: ColourBasics {
    // TODO: modify Manipulated colour to make it more widely applicable
    fn lightened(&self, prop: Prop) -> Self;
    fn darkened(&self, prop: Prop) -> Self;
    fn saturated(&self, prop: Prop) -> Self;
    fn greyed(&self, prop: Prop) -> Self;
    fn rotated(&self, angle: Angle) -> Self;
}
