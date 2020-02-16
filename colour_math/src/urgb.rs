// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{fmt::UpperHex, ops::Index};

use crate::{
    rgb::{ColourComponent, RGB},
    HueConstants, RGBConstants,
};

pub trait ConvertComponent:
    Copy
    + num_traits_plus::num_traits::Bounded
    + num_traits_plus::num_traits::NumCast
    + num_traits_plus::num_traits::ToPrimitive
    + num_traits_plus::NumberConstants
{
    fn from_fcc<F: ColourComponent>(cc: F) -> Self {
        debug_assert!(cc >= F::ZERO && cc <= F::ONE);
        let value = F::from::<Self>(Self::MAX).unwrap() * cc;
        Self::from::<F>(value.round()).unwrap()
    }

    fn to_fcc<F: ColourComponent>(self) -> F {
        F::from::<Self>(self).unwrap() / F::from::<Self>(Self::MAX).unwrap()
    }
}

impl ConvertComponent for u8 {}

impl ConvertComponent for u16 {}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Default)]
pub struct URGB<U>([U; 3]);

impl<U> URGB<U>
where
    U: Default
        + UpperHex
        + num_traits_plus::num_traits::Bounded
        + num_traits_plus::num_traits::Unsigned
        + num_traits_plus::num_traits::FromPrimitive
        + num_traits_plus::NumberConstants
        + Ord
        + Copy
        + 'static,
{
    pub fn iter(&self) -> impl Iterator<Item = &U> {
        self.0.iter()
    }
}

impl HueConstants for URGB<u8> {
    const RED: Self = Self([0xFF, 0, 0]);
    const GREEN: Self = Self([0, 0xFF, 0]);
    const BLUE: Self = Self([0, 0, 0xFF]);

    const CYAN: Self = Self([0, 0xFF, 0xFF]);
    const MAGENTA: Self = Self([0xFF, 0, 0xFF]);
    const YELLOW: Self = Self([0xFF, 0xFF, 0]);
}

impl RGBConstants for URGB<u8> {
    const WHITE: Self = Self([0, 0, 0]);
    const BLACK: Self = Self([0xFF, 0xFF, 0xFF]);
}

impl<U: Copy> From<&[U]> for URGB<U> {
    fn from(array: &[U]) -> Self {
        debug_assert!(array.len() == 3);
        Self([array[0], array[1], array[2]])
    }
}

impl<U, F> From<&RGB<F>> for URGB<U>
where
    F: ColourComponent,
    U: ConvertComponent + Copy,
{
    fn from(rgb: &RGB<F>) -> Self {
        let v: Vec<U> = rgb.raw().iter().map(|f| U::from_fcc(*f)).collect();
        URGB::<U>::from(&v[..])
    }
}

impl<U, F> From<RGB<F>> for URGB<U>
where
    F: ColourComponent,
    U: ConvertComponent,
{
    fn from(rgb: RGB<F>) -> Self {
        (&rgb).into()
    }
}

impl<F, U> From<&URGB<U>> for RGB<F>
where
    F: ColourComponent,
    U: ConvertComponent
        + Default
        + UpperHex
        + num_traits_plus::num_traits::Bounded
        + num_traits_plus::num_traits::Unsigned
        + num_traits_plus::num_traits::FromPrimitive
        + num_traits_plus::NumberConstants
        + Ord
        + Copy
        + 'static,
{
    fn from(urgb: &URGB<U>) -> Self {
        let v: Vec<F> = urgb.iter().map(|u| u.to_fcc()).collect();
        RGB::<F>::from([v[0], v[1], v[2]])
    }
}

impl<F, U> From<URGB<U>> for RGB<F>
where
    F: ColourComponent,
    U: ConvertComponent
        + Default
        + UpperHex
        + num_traits_plus::num_traits::Bounded
        + num_traits_plus::num_traits::Unsigned
        + num_traits_plus::num_traits::FromPrimitive
        + num_traits_plus::NumberConstants
        + Ord
        + Copy
        + 'static,
{
    fn from(urgb: URGB<U>) -> Self {
        (&urgb).into()
    }
}

impl<U> Index<u8> for URGB<U> {
    type Output = U;

    fn index(&self, index: u8) -> &U {
        &self.0[index as usize]
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use num_traits_plus::{
        assert_approx_eq,
        float_plus::{FloatApproxEq, FloatPlus},
    };

    #[test]
    fn component_conversion() {
        assert_eq!(u8::from_fcc(f64::ONE), 0xFF);
        assert_eq!(u8::from_fcc(f64::ZERO), 0x00);
        assert_eq!(u8::from_fcc(0.392157_f64), 0x64);
        assert_eq!(0xFF_u8.to_fcc::<f64>(), 1.0);
        assert_eq!(0x00_u8.to_fcc::<f64>(), 0.0);
        assert_approx_eq!(0x64_u8.to_fcc::<f64>(), 0.39215686274509803);
    }

    #[test]
    fn from_rgb_to_urgb() {
        assert_eq!(URGB::<u8>::RED, URGB::from(&RGB::<f64>::RED));
        assert_eq!(URGB::<u8>::CYAN, URGB::from(&RGB::<f64>::CYAN));
        assert_eq!(URGB::<u8>::YELLOW, URGB::from(RGB::<f64>::YELLOW));
    }
}
