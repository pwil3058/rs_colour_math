// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use colour_math::{ColourInterface, Hue, RGB};
use normalised_angles::Degrees;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum U8Pixel {
    RGB { rgb: RGB<f64> },
    RGBA { rgb: RGB<f64>, alpha: u8 },
}

impl ColourInterface<f64> for U8Pixel {
    fn rgb(&self) -> RGB<f64> {
        match self {
            U8Pixel::RGB { rgb } => rgb.rgb(),
            U8Pixel::RGBA { rgb, alpha: _ } => rgb.rgb(),
        }
    }

    fn rgba(&self) -> [f64; 4] {
        match self {
            U8Pixel::RGB { rgb } => rgb.rgba(),
            U8Pixel::RGBA { rgb, alpha } => [rgb[0], rgb[1], rgb[2], *alpha as f64 / 255.0],
        }
    }

    fn hue(&self) -> Option<Hue<f64>> {
        match self {
            U8Pixel::RGB { rgb } => rgb.hue(),
            U8Pixel::RGBA { rgb, alpha: _ } => rgb.hue(),
        }
    }

    fn hue_angle(&self) -> Option<Degrees<f64>> {
        match self {
            U8Pixel::RGB { rgb } => rgb.hue_angle(),
            U8Pixel::RGBA { rgb, alpha: _ } => rgb.hue_angle(),
        }
    }

    fn is_grey(&self) -> bool {
        match self {
            U8Pixel::RGB { rgb } => rgb.is_grey(),
            U8Pixel::RGBA { rgb, alpha: _ } => rgb.is_grey(),
        }
    }

    fn chroma(&self) -> f64 {
        match self {
            U8Pixel::RGB { rgb } => rgb.chroma(),
            U8Pixel::RGBA { rgb, alpha: _ } => rgb.chroma(),
        }
    }

    fn greyness(&self) -> f64 {
        match self {
            U8Pixel::RGB { rgb } => rgb.greyness(),
            U8Pixel::RGBA { rgb, alpha: _ } => rgb.greyness(),
        }
    }

    fn value(&self) -> f64 {
        match self {
            U8Pixel::RGB { rgb } => rgb.value(),
            U8Pixel::RGBA { rgb, alpha: _ } => rgb.value(),
        }
    }

    fn warmth(&self) -> f64 {
        match self {
            U8Pixel::RGB { rgb } => rgb.warmth(),
            U8Pixel::RGBA { rgb, alpha: _ } => rgb.warmth(),
        }
    }

    fn best_foreground_rgb(&self) -> RGB<f64> {
        match self {
            U8Pixel::RGB { rgb } => rgb.best_foreground_rgb(),
            U8Pixel::RGBA { rgb, alpha: _ } => rgb.best_foreground_rgb(),
        }
    }

    fn monochrome_rgb(&self) -> RGB<f64> {
        match self {
            U8Pixel::RGB { rgb } => rgb.monochrome_rgb(),
            U8Pixel::RGBA { rgb, alpha: _ } => rgb.monochrome_rgb(),
        }
    }

    fn max_chroma_rgb(&self) -> RGB<f64> {
        match self {
            U8Pixel::RGB { rgb } => rgb.max_chroma_rgb(),
            U8Pixel::RGBA { rgb, alpha: _ } => rgb.max_chroma_rgb(),
        }
    }

    fn warmth_rgb(&self) -> RGB<f64> {
        match self {
            U8Pixel::RGB { rgb } => rgb.warmth_rgb(),
            U8Pixel::RGBA { rgb, alpha: _ } => rgb.warmth_rgb(),
        }
    }
}

impl From<&[u8]> for U8Pixel {
    fn from(array: &[u8]) -> Self {
        let rgb: RGB<f64> = array[0..3].into();
        match array.len() {
            3 => U8Pixel::RGB { rgb },
            4 => U8Pixel::RGBA {
                rgb,
                alpha: array[3],
            },
            _ => panic!("Array must have 3 or 4 elements"),
        }
    }
}

impl From<&Vec<u8>> for U8Pixel {
    fn from(vec: &Vec<u8>) -> Self {
        let rgb: RGB<f64> = vec[0..3].into();
        match vec.len() {
            3 => U8Pixel::RGB { rgb },
            4 => U8Pixel::RGBA { rgb, alpha: vec[3] },
            _ => panic!("Array must have 3 or 4 elements"),
        }
    }
}

pub trait PixelTransform {
    fn transform(&self, pixel: &U8Pixel) -> U8Pixel;
}

#[derive(Default)]
pub struct ToMonochrome {}

impl PixelTransform for ToMonochrome {
    fn transform(&self, pixel: &U8Pixel) -> U8Pixel {
        match pixel {
            U8Pixel::RGB { rgb } => U8Pixel::RGB {
                rgb: rgb.monochrome_rgb(),
            },
            U8Pixel::RGBA { rgb, alpha } => U8Pixel::RGBA {
                rgb: rgb.monochrome_rgb(),
                alpha: *alpha,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use colour_math::{ColourInterface, HueConstants, RGB};

    #[test]
    fn create_u8_pixel() {
        let expected = U8Pixel::RGB {
            rgb: RGB::<f64>::from(&[1u8, 3, 255]),
        };
        let pixel = U8Pixel::from(&[1u8, 3, 255][0..3]);
        assert_eq!(pixel, expected);
        let pixel = U8Pixel::from(&vec![1u8, 3, 255]);
        assert_eq!(pixel, expected);
        let expected = U8Pixel::RGBA {
            rgb: (&[1u8, 3, 255]).into(),
            alpha: 64,
        };
        let pixel = U8Pixel::from(&[1u8, 3, 255, 64][0..4]);
        assert_eq!(pixel, expected);
    }

    #[test]
    fn monochrome_transform() {
        let pixel = U8Pixel::RGB {
            rgb: RGB::<f64>::RED,
        };
        let monochrome = ToMonochrome::default().transform(&pixel);
        assert_eq!(pixel.value(), monochrome.value());
        assert!(monochrome.is_grey());
    }
}
