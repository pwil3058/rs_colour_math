// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
#[macro_use]
extern crate serde_derive;

use std::fmt::Debug;

use num_traits_plus::float_plus::*;

pub mod hcv;
pub mod hue;
pub mod manipulator;
pub mod proportion;
pub mod rgb;

pub use crate::{
    hcv::HCV,
    hue::{angle::Angle, Hue, HueIfce},
    proportion::{Chroma, Prop, Sum},
    rgb::RGB,
};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CCI {
    Red,
    Green,
    Blue,
}

pub trait Float: FloatPlus + std::iter::Sum + FloatApproxEq<Self> {}

impl Float for f32 {}
impl Float for f64 {}

pub trait LightLevel: Clone + Copy + From<Prop> + Into<Prop> + PartialEq + Debug {
    const ZERO: Self;
    const ONE: Self;
}

impl LightLevel for f32 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
}
impl LightLevel for f64 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
}

impl LightLevel for u8 {
    const ZERO: Self = 0;
    const ONE: Self = u8::MAX;
}

impl LightLevel for u16 {
    const ZERO: Self = 0;
    const ONE: Self = u16::MAX;
}

impl LightLevel for u32 {
    const ZERO: Self = 0;
    const ONE: Self = u32::MAX;
}

impl LightLevel for u64 {
    const ZERO: Self = 0;
    const ONE: Self = u64::MAX;
}

pub trait HueConstants: Sized + Copy {
    const RED: Self;
    const GREEN: Self;
    const BLUE: Self;

    const CYAN: Self;
    const MAGENTA: Self;
    const YELLOW: Self;

    const PRIMARIES: [Self; 3] = [Self::RED, Self::GREEN, Self::BLUE];
    const SECONDARIES: [Self; 3] = [Self::CYAN, Self::MAGENTA, Self::YELLOW];
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

    fn is_grey(&self) -> bool {
        self.chroma() == Chroma::ZERO
    }

    fn chroma(&self) -> Chroma;
    fn value(&self) -> Prop;

    fn warmth(&self) -> Prop {
        if let Some(hue) = self.hue() {
            hue.warmth_for_chroma(self.chroma())
        } else {
            (Prop::ONE - self.value()) / 2
        }
    }

    fn hcv(&self) -> HCV;
    fn rgb<L: LightLevel>(&self) -> RGB<L>;
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
            ScalarAttribute::Value => self.value(),
            ScalarAttribute::Warmth => self.warmth(),
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

pub trait ChromaOneRGB {
    /// RGB wih chroma of 1.0 chroma and with its hue (value may change op or down)
    fn chroma_one_rgb<T: LightLevel>(&self) -> RGB<T>;
}
