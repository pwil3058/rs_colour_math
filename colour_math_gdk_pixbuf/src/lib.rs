// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use colour_math::{RGB, RGB8};

#[derive(PartialEq, Eq, Debug)]
pub enum U8Pixel {
    RGB { rgb: RGB<f64> },
    RGBA { rgb: RGB<f64>, alpha: u8 },
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
