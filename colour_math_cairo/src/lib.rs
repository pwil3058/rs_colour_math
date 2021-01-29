// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::cell::Cell;

use pw_gix::cairo;

use colour_math::{
    attributes::drawing::{self, Cartesian},
    ColourInterface, RGBConstants, CCI,
};

pub type Point = drawing::Point<f64>;
pub type Size = drawing::Size<f64>;
pub type TextPosn = drawing::TextPosn<f64>;

pub type RGB = colour_math::RGB<f64>;

pub trait CairoSetColour {
    fn set_source_colour<C: ColourInterface<f64>>(self, colour: &C);
    fn set_source_colour_rgb(&self, rgb: &RGB);
}

impl CairoSetColour for pw_gix::cairo::Context {
    fn set_source_colour<C: ColourInterface<f64>>(self, colour: &C) {
        self.set_source_colour_rgb(&colour.rgb());
    }

    fn set_source_colour_rgb(&self, rgb: &RGB) {
        self.set_source_rgb(rgb[CCI::Red], rgb[CCI::Green], rgb[CCI::Blue]);
    }
}

pub struct Drawer<'a> {
    pub cairo_context: &'a cairo::Context,
    size: Size,
    fill_colour: Cell<RGB>,
    line_colour: Cell<RGB>,
    text_colour: Cell<RGB>,
}

impl<'a> Drawer<'a> {
    pub fn new(cairo_context: &'a cairo::Context, size: Size) -> Self {
        Self {
            cairo_context,
            size,
            fill_colour: Cell::new(RGB::BLACK),
            line_colour: Cell::new(RGB::BLACK),
            text_colour: Cell::new(RGB::BLACK),
        }
    }

    fn fill(&self) {
        self.cairo_context
            .set_source_colour_rgb(&self.fill_colour.get());
        self.cairo_context.fill();
    }

    fn stroke(&self) {
        self.cairo_context
            .set_source_colour_rgb(&self.line_colour.get());
        self.cairo_context.stroke();
    }
}

impl<'a> drawing::Draw<f64> for Drawer<'a> {
    fn size(&self) -> Size {
        self.size
    }

    fn set_fill_colour(&self, rgb: RGB) {
        self.fill_colour.set(rgb)
    }

    fn set_line_colour(&self, rgb: RGB) {
        self.line_colour.set(rgb)
    }

    fn set_line_width(&self, width: f64) {
        self.cairo_context.set_line_width(width)
    }

    fn set_text_colour(&self, rgb: RGB) {
        self.text_colour.set(rgb)
    }

    fn draw_line(&self, line: &[Point]) {
        if let Some(start) = line.first() {
            self.cairo_context.move_to(start.x, start.y);
            for point in line[1..].iter() {
                self.cairo_context.line_to(point.x, point.y);
            }
            if line.len() > 1 {
                self.stroke();
            }
        }
    }

    fn paint_linear_gradient(&self, posn: Point, size: Size, colour_stops: &[(RGB, f64)]) {
        let linear_gradient =
            cairo::LinearGradient::new(0.0, 0.5 * size.height, size.width, 0.5 * size.height);
        for colour_stop in colour_stops.iter() {
            linear_gradient.add_color_stop_rgb(
                colour_stop.1,
                colour_stop.0[CCI::Red],
                colour_stop.0[CCI::Green],
                colour_stop.0[CCI::Blue],
            );
        }
        self.cairo_context
            .rectangle(posn.x, posn.y, size.width, size.height);
        //cairo_context.set_source(&cairo::Pattern::LinearGradient(linear_gradient));
        self.cairo_context.set_source(&linear_gradient);
        self.cairo_context.fill()
    }

    fn draw_polygon(&self, polygon: &[Point], fill: bool) {
        if let Some(start) = polygon.first() {
            self.cairo_context.move_to(start.x, start.y);
            for point in polygon[1..].iter() {
                self.cairo_context.line_to(point.x, point.y);
            }
            if polygon.len() > 1 {
                self.cairo_context.close_path();
                if fill {
                    self.fill();
                } else {
                    self.stroke();
                }
            }
        }
    }

    fn draw_text(&self, text: &str, posn: TextPosn, font_size: f64) {
        if text.is_empty() {
            return;
        }
        self.cairo_context.set_font_size(font_size);
        let te = self.cairo_context.text_extents(&text);
        match posn {
            TextPosn::Centre(point) => {
                self.cairo_context
                    .move_to(point.x - te.width / 2.0, point.y + te.height / 2.0);
            }
            TextPosn::TopLeftCorner(point) => {
                self.cairo_context.move_to(point.x, point.y + te.height);
            }
            TextPosn::TopRightCorner(point) => {
                self.cairo_context
                    .move_to(point.x - te.width, point.y + te.height);
            }
            TextPosn::BottomLeftCorner(point) => {
                self.cairo_context.move_to(point.x, point.y);
            }
            TextPosn::BottomRightCorner(point) => {
                self.cairo_context.move_to(point.x - te.width, point.y);
            }
        }
        self.cairo_context
            .set_source_colour_rgb(&self.text_colour.get());
        self.cairo_context.show_text(&text);
    }
}

pub struct CairoCartesian<'a> {
    pub cairo_context: &'a cairo::Context,
    fill_colour: Cell<RGB>,
    line_colour: Cell<RGB>,
}

impl<'a> CairoCartesian<'a> {
    pub fn cartesian_transform_matrix(width: f64, height: f64) -> cairo::Matrix {
        let scale = if width > height {
            height / 2.15
        } else {
            width / 2.15
        };
        cairo::Matrix::new(scale, 0.0, 0.0, -scale, width / 2.0, height / 2.0)
    }

    pub fn new(cairo_context: &'a cairo::Context) -> Self {
        Self {
            cairo_context,
            fill_colour: Cell::new(RGB::BLACK),
            line_colour: Cell::new(RGB::BLACK),
        }
    }

    fn fill(&self) {
        self.cairo_context
            .set_source_colour_rgb(&self.fill_colour.get());
        self.cairo_context.fill();
    }

    fn stroke(&self) {
        self.cairo_context
            .set_source_colour_rgb(&self.line_colour.get());
        self.cairo_context.stroke();
    }
}

impl<'a> Cartesian<f64> for CairoCartesian<'a> {
    fn draw_circle(&self, centre: Point, radius: f64, fill: bool) {
        const TWO_PI: f64 = 2.0 * std::f64::consts::PI;
        self.cairo_context
            .arc(centre.x, centre.y, radius, 0.0, TWO_PI);
        if fill {
            self.fill();
        } else {
            self.stroke();
        }
    }

    fn draw_line(&self, line: &[Point]) {
        if let Some(start) = line.first() {
            self.cairo_context.move_to(start.x, start.y);
            for point in line[1..].iter() {
                self.cairo_context.line_to(point.x, point.y);
            }
            if line.len() > 1 {
                self.stroke();
            }
        }
    }

    fn draw_polygon(&self, polygon: &[Point], fill: bool) {
        if let Some(start) = polygon.first() {
            self.cairo_context.move_to(start.x, start.y);
            for point in polygon[1..].iter() {
                self.cairo_context.line_to(point.x, point.y);
            }
            if polygon.len() > 1 {
                self.cairo_context.close_path();
                if fill {
                    self.fill();
                } else {
                    self.stroke();
                }
            }
        }
    }

    fn set_line_width(&self, width: f64) {
        self.cairo_context.set_line_width(width);
    }

    fn set_background_colour(&self, rgb: RGB) {
        self.cairo_context.set_source_colour_rgb(&rgb);
        self.cairo_context.paint();
    }

    fn set_line_colour(&self, rgb: RGB) {
        self.line_colour.set(rgb);
    }

    fn set_fill_colour(&self, rgb: RGB) {
        self.fill_colour.set(rgb);
    }
}
