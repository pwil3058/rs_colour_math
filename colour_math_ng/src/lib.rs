// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
#[macro_use]
extern crate serde_derive;

use std::fmt::Debug;

use normalised_angles::{Degrees, DegreesConst, RadiansConst};
use num_traits_plus::float_plus::*;

pub mod hcv;
pub mod hue;
pub mod manipulator;
pub mod proportion;
pub mod rgb;

use crate::hue::angle::Angle;
pub use crate::{
    hcv::HCV,
    hue::Hue,
    proportion::{Chroma, Prop, Sum},
    rgb::RGB,
};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CCI {
    Red,
    Green,
    Blue,
}

pub trait Float:
    FloatPlus + DegreesConst + RadiansConst + std::iter::Sum + FloatApproxEq<Self>
{
}

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

impl<F: Float> HueConstants for Degrees<F> {
    const RED: Self = Self::DEG_0;
    const GREEN: Self = Self::DEG_120;
    const BLUE: Self = Self::NEG_DEG_120;

    const CYAN: Self = Self::DEG_180;
    const MAGENTA: Self = Self::NEG_DEG_60;
    const YELLOW: Self = Self::DEG_60;
}

pub trait HueAngle {
    fn hue_angle<T: Float + From<Prop>>(&self) -> Degrees<T>;
    fn angle(&self) -> Angle;
}

pub trait ChromaOneRGB {
    /// RGB wih chroma of 1.0 chroma and with its hue (value may change op or down)
    fn chroma_one_rgb<T: LightLevel>(&self) -> RGB<T>;
}
