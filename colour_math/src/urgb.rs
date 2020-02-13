// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::fmt::UpperHex;

use crate::{
    rgb::{ColourComponent, RGB},
    RGBConstants,
};

pub trait FromColourComponent: Copy + num_traits::Bounded {
    fn from_cc<F: ColourComponent>(cc: F) -> Self;
    fn to_cc<F: ColourComponent>(self) -> F;

    fn unsigned_one<F: ColourComponent>() -> F {
        Self::max_value().to_cc()
    }
}

impl FromColourComponent for u8 {
    fn from_cc<F: ColourComponent>(cc: F) -> Self {
        debug_assert!(cc >= F::ZERO && cc <= F::ONE);
        let value = F::from_u8(u8::max_value()).unwrap() * cc;
        value.to_u8().unwrap()
    }

    fn to_cc<F: ColourComponent>(self) -> F {
        F::from_u8(self).unwrap() / F::from_u8(u8::max_value()).unwrap()
    }
}

impl FromColourComponent for u16 {
    fn from_cc<F: ColourComponent>(cc: F) -> Self {
        debug_assert!(cc >= F::ZERO && cc <= F::ONE);
        let value = F::from_u16(u16::max_value()).unwrap() * cc;
        value.to_u16().unwrap()
    }

    fn to_cc<F: ColourComponent>(self) -> F {
        F::from_u16(self).unwrap() / F::from_u16(u16::max_value()).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Default)]
pub struct URGB<U>([U; 3]);

impl<U> URGB<U>
where
    U: Default
        + UpperHex
        + num_traits::Bounded
        + num_traits::Unsigned
        + num_traits::FromPrimitive
        + num_traits_plus::NumberConstants
        + Ord
        + Copy
        + 'static,
{
    pub fn iter(&self) -> impl Iterator<Item = &U> {
        self.0.iter()
    }
}

impl RGBConstants for URGB<u8> {
    const RED: Self = Self([0xFF, 0, 0]);
    const GREEN: Self = Self([0, 0xFF, 0]);
    const BLUE: Self = Self([0, 0, 0xFF]);

    const CYAN: Self = Self([0, 0xFF, 0xFF]);
    const MAGENTA: Self = Self([0xFF, 0, 0xFF]);
    const YELLOW: Self = Self([0xFF, 0xFF, 0]);

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
    U: FromColourComponent + Copy,
{
    fn from(rgb: &RGB<F>) -> Self {
        let one: F = U::unsigned_one();
        let v: Vec<U> = rgb
            .raw()
            .iter()
            .map(|f| {
                let value = (one * *f).round();
                U::from_cc(value)
            })
            .collect();
        URGB::<U>::from(&v[..])
    }
}

impl<U, F> From<RGB<F>> for URGB<U>
where
    F: ColourComponent,
    U: FromColourComponent,
{
    fn from(rgb: RGB<F>) -> Self {
        (&rgb).into()
    }
}

impl<F, U> From<&URGB<U>> for RGB<F>
where
    F: ColourComponent,
    U: FromColourComponent
        + Default
        + UpperHex
        + num_traits::Bounded
        + num_traits::Unsigned
        + num_traits::FromPrimitive
        + num_traits_plus::NumberConstants
        + Ord
        + Copy
        + 'static,
{
    fn from(urgb: &URGB<U>) -> Self {
        let one: F = U::unsigned_one();
        let v: Vec<F> = urgb
            .iter()
            .map(|u| {
                let enumerator: F = u.to_cc();
                enumerator / one
            })
            .collect();
        RGB::<F>::from([v[0], v[1], v[2]])
    }
}

impl<F, U> From<URGB<U>> for RGB<F>
where
    F: ColourComponent,
    U: FromColourComponent
        + Default
        + UpperHex
        + num_traits::Bounded
        + num_traits::Unsigned
        + num_traits::FromPrimitive
        + num_traits_plus::NumberConstants
        + Ord
        + Copy
        + 'static,
{
    fn from(urgb: URGB<U>) -> Self {
        (&urgb).into()
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
        assert_eq!(u8::from_cc(f64::ONE), 0xFF);
        assert_eq!(u8::from_cc(f64::ZERO), 0x00);
        assert_eq!(u8::from_cc(0.392157_f64), 0x64);
        assert_eq!(0xFF_u8.to_cc::<f64>(), 1.0);
        assert_eq!(0x00_u8.to_cc::<f64>(), 0.0);
        assert_approx_eq!(0x64_u8.to_cc::<f64>(), 0.39215686274509803);
    }

    #[test]
    fn from_rgb_to_urgb() {
        assert_eq!(URGB::<u8>::RED, URGB::from(&RGB::<f64>::RED));
        assert_eq!(URGB::<u8>::CYAN, URGB::from(&RGB::<f64>::CYAN));
        assert_eq!(URGB::<u8>::YELLOW, URGB::from(RGB::<f64>::YELLOW));
    }
}
