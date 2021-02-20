// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::{
    fdrn::{FDRNumber, UFDRNumber},
    ColourBasics, HCV,
};

#[cfg(test)]
mod test_beigui;

pub mod display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Point {
    pub x: FDRNumber,
    pub y: FDRNumber,
}

impl From<[FDRNumber; 2]> for Point {
    fn from(array: [FDRNumber; 2]) -> Self {
        Self {
            x: array[0],
            y: array[1],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Size {
    pub width: UFDRNumber,
    pub height: UFDRNumber,
}

impl Size {
    pub fn centre(&self) -> Point {
        [(self.width / 2).into(), (self.height / 2).into()].into()
    }
}

impl From<[UFDRNumber; 2]> for Size {
    fn from(array: [UFDRNumber; 2]) -> Self {
        Self {
            width: array[0],
            height: array[1],
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

pub enum TextPosn {
    TopLeftCorner(Point),
    TopRightCorner(Point),
    BottomLeftCorner(Point),
    BottomRightCorner(Point),
    Centre(Point),
}

pub trait Draw {
    fn size(&self) -> Size;
    fn draw_polygon(&self, polygon: &[Point], fill: bool);

    fn set_fill_colour(&self, colour: &impl ColourBasics);
    fn set_line_colour(&self, colour: &impl ColourBasics);
    fn set_text_colour(&self, colour: &impl ColourBasics);

    fn set_line_width(&self, width: UFDRNumber);

    fn draw_line(&self, line: &[Point]);
    fn draw_text(&self, text: &str, posn: TextPosn, font_size: UFDRNumber);

    fn paint_linear_gradient(&self, posn: Point, size: Size, colour_stops: &[(HCV, FDRNumber)]);
}

pub trait DrawIsosceles: Draw {
    fn draw_isosceles(
        &self,
        centre: Point,
        dirn: Dirn,
        base: UFDRNumber,
        height: UFDRNumber,
        fill: bool,
    ) {
        let half_base = base / 2;
        let half_height = height / 2;
        let points = match dirn {
            Dirn::Up => vec![
                Point {
                    x: centre.x - half_base.into(),
                    y: centre.y - half_height.into(),
                },
                Point {
                    x: centre.x,
                    y: centre.y + half_height.into(),
                },
                Point {
                    x: centre.x + half_base.into(),
                    y: centre.y - half_height.into(),
                },
            ],
            Dirn::Down => vec![
                Point {
                    x: centre.x - half_base.into(),
                    y: centre.y + half_height.into(),
                },
                Point {
                    x: centre.x,
                    y: centre.y - half_height.into(),
                },
                Point {
                    x: centre.x + half_base.into(),
                    y: centre.y + half_height.into(),
                },
            ],
            Dirn::Right => vec![
                Point {
                    x: centre.x - half_height.into(),
                    y: centre.y - half_base.into(),
                },
                Point {
                    x: centre.x - half_height.into(),
                    y: centre.y + half_base.into(),
                },
                Point {
                    x: centre.x + half_height.into(),
                    y: centre.y,
                },
            ],
            Dirn::Left => vec![
                Point {
                    x: centre.x + half_height.into(),
                    y: centre.y - half_base.into(),
                },
                Point {
                    x: centre.x + half_height.into(),
                    y: centre.y + half_base.into(),
                },
                Point {
                    x: centre.x - half_height.into(),
                    y: centre.y,
                },
            ],
        };
        self.draw_polygon(&points, fill);
    }
}

pub trait DrawShapes: DrawIsosceles {
    fn draw_diamond(&self, centre: Point, side_length: FDRNumber, fill: bool) {
        let dist = side_length / 2;
        let points = vec![
            Point {
                x: centre.x,
                y: centre.y + dist,
            },
            Point {
                x: centre.x + dist,
                y: centre.y,
            },
            Point {
                x: centre.x,
                y: centre.y - dist,
            },
            Point {
                x: centre.x - dist,
                y: centre.y,
            },
        ];
        self.draw_polygon(&points, fill);
    }
}
