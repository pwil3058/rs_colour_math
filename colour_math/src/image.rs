// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
//use std::slice::Iter;

#[derive(Debug, Clone, Copy)]
pub struct XY {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

pub trait Filter<P: Copy + 'static> {
    fn filter(&self, pixel: &P) -> P;
}

pub trait ImageIfce<'a, P: Copy + 'static> {
    //type PixelIterator: Iterator<Item = &'a P>;
    //type RowIterator: Iterator<Item = &'a [P]>;

    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn sub_image(&self, start: XY, size: Size) -> Self;
    fn pixels(&self) -> &[P];
    //fn rows(&self) -> Self::RowIterator;
    fn filtered<F: Filter<P>>(&self, filter: F) -> Self;

    fn size(&self) -> Size {
        Size {
            width: self.width(),
            height: self.height(),
        }
    }
}

pub struct GenericImage<P> {
    width: usize,
    height: usize,
    _pixels: Vec<P>,
}

impl<'a, P: Copy + 'static> ImageIfce<'a, P> for GenericImage<P> {
    //type PixelIterator = std::slice::Iter<'a, P>;
    //type RowIterator = std::slice::Iter<'a, &'a [P]>;

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn sub_image(&self, _start: XY, _size: Size) -> Self {
        unimplemented!("later")
    }

    fn pixels(&self) -> &[P] {
        &self._pixels[..]
    }

    //fn rows(&self) -> Self::RowIterator {
    //    unimplemented!("later")
    //}

    fn filtered<F: Filter<P>>(&self, _filter: F) -> Self {
        unimplemented!("later")
    }

    fn size(&self) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
    }
}

use crate::rgb::*;
use crate::ColourComponent;

pub struct OpaqueImage<F: ColourComponent> {
    pixels: Vec<RGB<F>>,
    width: usize,
}

impl<F: ColourComponent> OpaqueImage<F> {
    pub fn new(width: usize, height: usize, rgb: RGB<F>) -> Self {
        Self {
            pixels: vec![rgb; width * height],
            width,
        }
    }

    pub fn width(self) -> usize {
        self.width
    }

    pub fn height(self) -> usize {
        self.pixels.len() / self.width()
    }

    pub fn pixels(&self) -> impl Iterator<Item = &RGB<F>> {
        self.pixels.iter()
    }

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
