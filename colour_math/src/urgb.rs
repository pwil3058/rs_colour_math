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

#[derive(
    Serialize, Deserialize, Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord,
)]
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

impl<U: num_traits_plus::NumberConstants> HueConstants for URGB<U> {
    const RED: Self = Self([U::MAX, U::ZERO, U::ZERO]);
    const GREEN: Self = Self([U::ZERO, U::MAX, U::ZERO]);
    const BLUE: Self = Self([U::ZERO, U::ZERO, U::MAX]);

    const CYAN: Self = Self([U::ZERO, U::MAX, U::MAX]);
    const MAGENTA: Self = Self([U::MAX, U::ZERO, U::MAX]);
    const YELLOW: Self = Self([U::MAX, U::MAX, U::ZERO]);
}

impl<U: num_traits_plus::NumberConstants> RGBConstants for URGB<U> {
    const WHITE: Self = Self([U::ZERO, U::ZERO, U::ZERO]);
    const BLACK: Self = Self([U::MAX, U::MAX, U::MAX]);
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
        let v: Vec<U> = rgb.iter().map(|f| U::from_fcc(*f)).collect();
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

impl<U, V> From<&URGB<V>> for URGB<U>
where
    U: ConvertComponent
        + num_traits_plus::NumberConstants
        + num_traits_plus::num_traits::FromPrimitive
        + num_traits_plus::num_traits::NumCast
        + std::ops::Shl<usize, Output = U>
        + Copy,
    V: ConvertComponent
        + Default
        + UpperHex
        + Ord
        + num_traits_plus::num_traits::Unsigned
        + num_traits_plus::NumberConstants
        + num_traits_plus::num_traits::FromPrimitive
        + num_traits_plus::num_traits::ToPrimitive
        + Copy
        + 'static,
{
    fn from(urgb: &URGB<V>) -> Self {
        if U::BYTES == V::BYTES {
            Self([
                U::from::<V>(urgb.0[0]).unwrap(),
                U::from::<V>(urgb.0[1]).unwrap(),
                U::from::<V>(urgb.0[2]).unwrap(),
            ])
        } else {
            let rgb: RGB<f64> = urgb.into();
            rgb.into()
        }
    }
}

impl<U: Copy> From<&URGB<U>> for (U, U, U) {
    fn from(urgb: &URGB<U>) -> (U, U, U) {
        (urgb[0], urgb[1], urgb[2])
    }
}

impl<U: Copy> From<&URGB<U>> for [U; 3] {
    fn from(urgb: &URGB<U>) -> [U; 3] {
        urgb.0
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

    #[test]
    fn from_urgb_to_urgb() {
        assert_eq!(URGB::<u8>::RED, URGB::<u8>::from(&URGB::<u16>::RED));
        assert_eq!(URGB::<u8>::RED, URGB::<u8>::from(&URGB::<u8>::RED));
        assert_eq!(URGB::<u16>::RED, URGB::<u16>::from(&URGB::<u8>::RED));
    }
}
