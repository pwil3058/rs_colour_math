// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::{
    fdrn::{Prop, UFDRNumber},
    ColourBasics, LightLevel, HCV, RGB,
};

#[derive(Default, Debug)]
pub struct SubtractiveMixer {
    red: UFDRNumber,
    green: UFDRNumber,
    blue: UFDRNumber,
    total_parts: u64,
}

impl SubtractiveMixer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, colour: &impl ColourBasics, parts: u64) {
        let [red, green, blue] = <[Prop; 3]>::from(colour.hcv());
        self.red = self.red + UFDRNumber(red.0 as u128 * parts as u128);
        self.green = self.green + UFDRNumber(green.0 as u128 * parts as u128);
        self.blue = self.blue + UFDRNumber(blue.0 as u128 * parts as u128);
        self.total_parts += parts;
    }

    pub fn mixed_colour(&self) -> Option<HCV> {
        if self.total_parts > 0 {
            let red = Prop((self.red.0 / self.total_parts as u128) as u64);
            let green = Prop((self.green.0 / self.total_parts as u128) as u64);
            let blue = Prop((self.blue.0 / self.total_parts as u128) as u64);
            Some(HCV::from(&[red, green, blue]))
        } else {
            None
        }
    }

    pub fn mixed_rgb<L: LightLevel>(&self) -> Option<RGB<L>> {
        Some(self.mixed_colour()?.into())
    }

    pub fn reset(&mut self) {
        self.red = UFDRNumber::ZERO;
        self.green = UFDRNumber::ZERO;
        self.blue = UFDRNumber::ZERO;
        self.total_parts = 0;
    }
}

#[cfg(test)]
mod mixing_tests {
    use super::*;
    use crate::HueConstants;

    #[test]
    fn subtractive_mixing() {
        let mut subtractve_mixer = SubtractiveMixer::new();
        assert_eq!(subtractve_mixer.mixed_colour(), None);
        assert_eq!(subtractve_mixer.mixed_rgb::<u16>(), None);
        subtractve_mixer.add(&RGB::<u8>::RED, 5);
        assert_eq!(subtractve_mixer.mixed_colour(), Some(HCV::RED));
        assert_eq!(subtractve_mixer.mixed_rgb::<u16>(), Some(RGB::<u16>::RED));
        subtractve_mixer.add(&RGB::<u8>::GREEN, 10);
        let expected = RGB::<u16>::from([Prop::ONE / 3, (Prop::ONE * 2 / 3).into(), Prop::ZERO]);
        assert_eq!(subtractve_mixer.mixed_colour(), Some(expected.into()));
        assert_eq!(subtractve_mixer.mixed_rgb::<u16>(), Some(expected));
    }
}
