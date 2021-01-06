// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
//use std::slice::Iter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct XY {
    pub x: usize,
    pub y: usize,
}

impl From<(usize, usize)> for XY {
    fn from(tuple: (usize, usize)) -> Self {
        Self {
            x: tuple.0,
            y: tuple.1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

impl From<(usize, usize)> for Size {
    fn from(tuple: (usize, usize)) -> Self {
        Self {
            width: tuple.0,
            height: tuple.1,
        }
    }
}

impl Size {
    pub fn area(&self) -> usize {
        self.width * self.height
    }
}

pub trait Transformer<P: Copy + 'static> {
    fn transform(&self, pixel: &P) -> P;
}

pub trait ImageIfce<'a, P: Copy + Default + 'static>:
    std::ops::Index<usize, Output = [P]>
    + std::ops::IndexMut<usize>
    + std::convert::From<(Vec<P>, usize)>
    + Sized
{
    fn new(width: usize, height: usize) -> Self;
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn pixels(&self) -> &[P];

    fn sub_image(&self, start: XY, size: Size) -> Option<Self> {
        if size.area() > 0 && start.x < self.width() && start.y < self.height() {
            let width = size.width.min(self.width() - start.x);
            let height = size.height.min(self.height() - start.y);
            let end_col = start.x + width;
            let end_row = start.y + height;
            let mut pixels: Vec<P> = Vec::<P>::with_capacity(width * height);
            for i in start.y..end_row {
                let row = &self[i];
                pixels.extend(&row[start.x..end_col]);
            }
            Some((pixels, width).into())
        } else {
            None
        }
    }

    fn transformed<T: Transformer<P>>(&self, transformer: T) -> Self {
        let pixels: Vec<P> = self
            .pixels()
            .iter()
            .map(|p| transformer.transform(p))
            .collect();
        debug_assert_eq!(pixels.len(), self.size().area());
        (pixels, self.width()).into()
    }

    fn size(&self) -> Size {
        Size {
            width: self.width(),
            height: self.height(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericImage<P> {
    width: usize,
    height: usize,
    pixels: Vec<P>,
}

impl<P> std::ops::Index<usize> for GenericImage<P> {
    type Output = [P];

    fn index(&self, row: usize) -> &Self::Output {
        let start = self.width * row;
        debug_assert!(start < self.pixels.len());
        &self.pixels[start..start + self.width]
    }
}

impl<P> std::ops::IndexMut<usize> for GenericImage<P> {
    fn index_mut(&mut self, row: usize) -> &mut Self::Output {
        let start = self.width * row;
        debug_assert!(start < self.pixels.len());
        &mut self.pixels[start..start + self.width]
    }
}

impl<'a, P: Copy + Default + 'static> ImageIfce<'a, P> for GenericImage<P> {
    fn new(width: usize, height: usize) -> Self {
        debug_assert!(width > 0 && height > 0);
        let pixels = vec![P::default(); width * height];
        Self {
            width,
            height,
            pixels,
        }
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn pixels(&self) -> &[P] {
        &self.pixels[..]
    }

    fn size(&self) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
    }
}

impl<P: Copy> From<(Vec<P>, usize)> for GenericImage<P> {
    fn from(data: (Vec<P>, usize)) -> Self {
        debug_assert_eq!(data.0.len() % data.1, 0);
        Self {
            width: data.1,
            height: data.0.len() / data.1,
            pixels: data.0,
        }
    }
}

impl<P: Copy> From<(Size, Vec<P>)> for GenericImage<P> {
    fn from(data: (Size, Vec<P>)) -> Self {
        debug_assert_eq!(data.0.area(), data.1.len());
        Self {
            width: data.0.width,
            height: data.0.height,
            pixels: data.1,
        }
    }
}

impl<P: Copy> From<(Size, &[P])> for GenericImage<P> {
    fn from(data: (Size, &[P])) -> Self {
        debug_assert_eq!(data.0.area(), data.1.len());
        Self {
            width: data.0.width,
            height: data.0.height,
            pixels: data.1.to_vec(),
        }
    }
}

use crate::rgb::*;
use crate::{ColourComponent, ColourInterface};

pub struct OpaqueImage<F: ColourComponent> {
    pixels: Vec<RGB<F>>,
    width: usize,
}

impl<'a, F: ColourComponent + 'static> ImageIfce<'a, RGB<F>> for OpaqueImage<F> {
    fn new(width: usize, height: usize) -> Self {
        Self {
            pixels: vec![RGB::default(); width * height],
            width,
        }
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.pixels.len() / self.width()
    }

    fn pixels(&self) -> &[RGB<F>] {
        &self.pixels[..]
    }
}

impl<F: ColourComponent> OpaqueImage<F> {
    pub fn average_value(&self) -> F {
        let sum: F = self.pixels.iter().map(|p| p.value()).sum();
        sum / F::from_usize(self.pixels.len()).unwrap()
    }

    pub fn average_chroma(&self) -> F {
        let sum: F = self.pixels.iter().map(|p| p.chroma()).sum();
        sum / F::from_usize(self.pixels.len()).unwrap()
    }

    pub fn average_warmth(&self) -> F {
        let sum: F = self.pixels.iter().map(|p| p.warmth()).sum();
        sum / F::from_usize(self.pixels.len()).unwrap()
    }
}

impl<F: ColourComponent> std::ops::Index<usize> for OpaqueImage<F> {
    type Output = [RGB<F>];

    fn index(&self, row: usize) -> &[RGB<F>] {
        let start = self.width * row;
        debug_assert!(start < self.pixels.len());
        &self.pixels[start..start + self.width]
    }
}

impl<F: ColourComponent> std::ops::IndexMut<usize> for OpaqueImage<F> {
    fn index_mut(&mut self, row: usize) -> &mut [RGB<F>] {
        let start = self.width * row;
        debug_assert!(start < self.pixels.len());
        &mut self.pixels[start..start + self.width]
    }
}

impl<F: ColourComponent> From<(Vec<RGB<F>>, usize)> for OpaqueImage<F> {
    fn from(data: (Vec<RGB<F>>, usize)) -> Self {
        debug_assert_eq!(data.0.len() % data.1, 0);
        Self {
            width: data.1,
            pixels: data.0,
        }
    }
}

impl<F: ColourComponent> From<(&[u8], usize)> for OpaqueImage<F> {
    fn from(tuple: (&[u8], usize)) -> Self {
        let (data, width) = tuple;
        debug_assert_eq!(data.len() % (width * 3), 0);
        let mut pixels: Vec<RGB<F>> = Vec::with_capacity(data.len() / 3);
        for chunk in data.chunks(3) {
            pixels.push(chunk.into());
        }
        Self { pixels, width }
    }
}

impl<F: ColourComponent> From<(&[u8], usize, usize)> for OpaqueImage<F> {
    fn from(tuple: (&[u8], usize, usize)) -> Self {
        let (data, width, stride) = tuple;
        debug_assert!((stride >= width));
        let row_len = stride * 3;
        debug_assert_eq!(data.len() % row_len, 0);
        let height = data.len() / row_len;
        let mut pixels: Vec<RGB<F>> = Vec::with_capacity(width * height);
        for h in 0..height {
            let row_start = h * stride;
            let row_end = row_start + width;
            for chunk in data[row_start..row_end].chunks(3) {
                pixels.push(chunk.into());
            }
        }
        Self { pixels, width }
    }
}

#[cfg(test)]
mod image_tests {
    use super::*;
    use crate::{rgb::RGB, ColourInterface};
    use serde::export::PhantomData;

    #[derive(Default)]
    struct ToMonochrome<P: Copy + ColourInterface<f64>> {
        phantom_data: PhantomData<P>,
    }

    impl Transformer<RGB<f64>> for ToMonochrome<RGB<f64>> {
        fn transform(&self, pixel: &RGB<f64>) -> RGB<f64> {
            pixel.monochrome_rgb()
        }
    }

    #[test]
    fn new_image() {
        let image = GenericImage::<RGB<f64>>::new(6, 4);
        assert_eq!(
            image.size(),
            Size {
                width: 6,
                height: 4
            }
        );
        assert_eq!(image.size().area(), image.pixels.len());
        for i in 0..image.height() {
            for j in 0..image.width() {
                assert_eq!(image[i][j], RGB::<f64>::BLACK);
            }
        }
    }

    #[test]
    fn new_image_from_data() {
        let mut v = Vec::<RGB<f64>>::with_capacity(6);
        v.extend(&RGB::<f64>::PRIMARIES);
        v.extend(&RGB::<f64>::SECONDARIES);
        let image = GenericImage::<RGB<f64>>::from((Size::from((3, 2)), v));
        assert_eq!(image[0][0], RGB::<f64>::RED);
        assert_eq!(image[0][2], RGB::<f64>::BLUE);
        assert_eq!(image[1][2], RGB::<f64>::YELLOW);
    }

    #[test]
    fn sub_image() {
        let mut image = GenericImage::<RGB<f64>>::new(6, 4);
        assert_eq!(image.sub_image(XY::default(), Size::default()), None);
        assert_eq!(image.sub_image(XY::from((6, 0)), image.size()), None);
        assert_eq!(image.sub_image(XY::from((0, 4)), image.size()), None);
        image[1][1] = RGB::<f64>::YELLOW;
        let sub_image = image.sub_image(XY::default(), image.size()).unwrap();
        assert_eq!(image, sub_image);
        image[2][2] = RGB::<f64>::RED;
        assert_ne!(image, sub_image);
        let sub_image = image.sub_image(XY::from((1, 1)), image.size()).unwrap();
        assert_eq!(
            sub_image.size(),
            Size::from((image.width() - 1, image.height() - 1))
        );
        assert_eq!(sub_image[0][0], RGB::<f64>::YELLOW);
        assert_eq!(sub_image[1][1], RGB::<f64>::RED);
    }

    #[test]
    fn transformed_image() {
        let mut v = Vec::<RGB<f64>>::with_capacity(6);
        v.extend(&RGB::<f64>::PRIMARIES);
        v.extend(&RGB::<f64>::SECONDARIES);
        let image = GenericImage::<RGB<f64>>::from((Size::from((3, 2)), v));
        for pixel in image.pixels() {
            assert!(!pixel.is_grey());
        }
        let transformed = image.transformed(ToMonochrome::default());
        for pixel in transformed.pixels() {
            assert!(pixel.is_grey());
        }
        for i in 0..2 {
            for j in 0..3 {
                assert_eq!(image[i][j].value(), transformed[i][j].value())
            }
        }
    }
}
