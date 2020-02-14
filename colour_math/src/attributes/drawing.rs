// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::{ColourComponent, RGB};
use num_traits_plus::float_plus::FloatPlus;
use std::default::Default;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point<F: FloatPlus + Default> {
    pub x: F,
    pub y: F,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Size<F: FloatPlus + Default> {
    pub width: F,
    pub height: F,
}

impl<F: FloatPlus + Default> Size<F> {
    pub fn centre(&self) -> Point<F> {
        (self.width / F::TWO, self.height / F::TWO).into()
    }
}

impl<F: FloatPlus + Default> From<(F, F)> for Point<F> {
    fn from(tuple: (F, F)) -> Self {
        Self {
            x: tuple.0,
            y: tuple.1,
        }
    }
}

/// Direction in which to draw isosceles triangle
pub enum Dirn {
    Down,
    Up,
    Right,
    Left,
}

pub enum TextPosn<F: FloatPlus + Default> {
    TopLeftCorner(Point<F>),
    TopRightCorner(Point<F>),
    BottomLeftCorner(Point<F>),
    BottomRightCorner(Point<F>),
    Centre(Point<F>),
}

pub trait Draw<F: ColourComponent + Default> {
    fn size(&self) -> Size<F>;
    fn set_fill_colour(&self, rgb: RGB<F>);
    fn set_line_colour(&self, rgb: RGB<F>);
    fn set_line_width(&self, width: F);
    fn set_text_colour(&self, rgb: RGB<F>);
    fn draw_line(&self, line: &[Point<F>]);
    fn paint_linear_gradient(&self, posn: Point<F>, size: Size<F>, colour_stops: &[(RGB<F>, F)]);
    fn draw_polygon(&self, polygon: &[Point<F>], fill: bool);
    fn draw_text(&self, text: &str, posn: TextPosn<F>, font_size: F);

    fn draw_isosceles(&self, centre: Point<F>, dirn: Dirn, base: F, height: F, fill: bool) {
        let half_base = base * F::HALF;
        let half_height = height * F::HALF;
        let points = match dirn {
            Dirn::Up => vec![
                Point {
                    x: centre.x - half_base,
                    y: centre.y - half_height,
                },
                Point {
                    x: centre.x,
                    y: centre.y + half_height,
                },
                Point {
                    x: centre.x + half_base,
                    y: centre.y - half_height,
                },
            ],
            Dirn::Down => vec![
                Point {
                    x: centre.x - half_base,
                    y: centre.y + half_height,
                },
                Point {
                    x: centre.x,
                    y: centre.y - half_height,
                },
                Point {
                    x: centre.x + half_base,
                    y: centre.y + half_height,
                },
            ],
            Dirn::Right => vec![
                Point {
                    x: centre.x - half_height,
                    y: centre.y - half_base,
                },
                Point {
                    x: centre.x - half_height,
                    y: centre.y + half_base,
                },
                Point {
                    x: centre.x + half_height,
                    y: centre.y,
                },
            ],
            Dirn::Left => vec![
                Point {
                    x: centre.x + half_height,
                    y: centre.y - half_base,
                },
                Point {
                    x: centre.x + half_height,
                    y: centre.y + half_base,
                },
                Point {
                    x: centre.x - half_height,
                    y: centre.y,
                },
            ],
        };
        self.draw_polygon(&points, fill);
    }
}
