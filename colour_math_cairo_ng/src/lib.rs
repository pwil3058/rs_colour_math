// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::cell::Cell;

use pw_gix::cairo;

use colour_math_ng::beigui::DrawShapes;
use colour_math_ng::{
    beigui::{self, Draw, DrawIsosceles, Point},
    ColourBasics, Prop, RGBConstants, UFDRNumber, CCI, HCV, RGB,
};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

impl Size {
    pub fn centre(&self) -> [f64; 2] {
        [self.width / 2.0, self.height / 2.0]
    }
}

impl From<Size> for beigui::Size {
    fn from(size: Size) -> Self {
        Self {
            width: size.width.into(),
            height: size.height.into(),
        }
    }
}

impl From<beigui::Size> for Size {
    fn from(size: beigui::Size) -> Self {
        Self {
            width: size.width.into(),
            height: size.height.into(),
        }
    }
}

pub enum TextPosn {
    TopLeftCorner(f64, f64),
    TopRightCorner(f64, f64),
    BottomLeftCorner(f64, f64),
    BottomRightCorner(f64, f64),
    Centre(f64, f64),
}

impl From<beigui::TextPosn> for TextPosn {
    fn from(text_posn: beigui::TextPosn) -> Self {
        use beigui::TextPosn::*;
        match text_posn {
            TopLeftCorner(point) => TextPosn::TopLeftCorner(point.x.into(), point.y.into()),
            TopRightCorner(point) => TextPosn::TopRightCorner(point.x.into(), point.y.into()),
            BottomLeftCorner(point) => TextPosn::BottomLeftCorner(point.x.into(), point.y.into()),
            BottomRightCorner(point) => TextPosn::BottomRightCorner(point.x.into(), point.y.into()),
            Centre(point) => TextPosn::Centre(point.x.into(), point.y.into()),
        }
    }
}

pub trait CairoSetColour {
    fn set_source_colour_rgb(&self, rgb: &RGB<f64>);
}

impl CairoSetColour for pw_gix::cairo::Context {
    fn set_source_colour_rgb(&self, rgb: &RGB<f64>) {
        self.set_source_rgb(rgb[CCI::Red], rgb[CCI::Green], rgb[CCI::Blue]);
    }
}

pub struct Drawer<'a> {
    pub cairo_context: &'a cairo::Context,
    size: Size,
    fill_colour: Cell<RGB<f64>>,
    line_colour: Cell<RGB<f64>>,
    text_colour: Cell<RGB<f64>>,
}

impl<'a> Drawer<'a> {
    pub fn new(cairo_context: &'a cairo::Context, size: Size) -> Self {
        Self {
            cairo_context,
            size,
            fill_colour: Cell::new(RGB::<f64>::BLACK),
            line_colour: Cell::new(RGB::<f64>::BLACK),
            text_colour: Cell::new(RGB::<f64>::BLACK),
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

impl<'a> Draw for Drawer<'a> {
    fn size(&self) -> beigui::Size {
        self.size.into()
    }

    fn draw_polygon(&self, polygon: &[Point], fill: bool) {
        if let Some(start) = polygon.first() {
            self.cairo_context.move_to(start.x.into(), start.y.into());
            for point in polygon[1..].iter() {
                self.cairo_context.line_to(point.x.into(), point.y.into());
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

    fn set_fill_colour(&self, colour: &impl ColourBasics) {
        self.fill_colour.set(colour.rgb());
    }

    fn set_line_colour(&self, colour: &impl ColourBasics) {
        self.line_colour.set(colour.rgb());
    }

    fn set_text_colour(&self, colour: &impl ColourBasics) {
        self.text_colour.set(colour.rgb());
    }

    fn set_line_width(&self, width: UFDRNumber) {
        self.cairo_context.set_line_width(width.into());
    }

    fn draw_line(&self, line: &[Point]) {
        if let Some(start) = line.first() {
            self.cairo_context.move_to(start.x.into(), start.y.into());
            for point in line[1..].iter() {
                self.cairo_context.line_to(point.x.into(), point.y.into());
            }
            if line.len() > 1 {
                self.stroke();
            }
        }
    }

    fn draw_text(&self, text: &str, posn: beigui::TextPosn, font_size: UFDRNumber) {
        if text.is_empty() {
            return;
        }
        self.cairo_context.set_font_size(font_size.into());
        let te = self.cairo_context.text_extents(&text);
        match TextPosn::from(posn) {
            TextPosn::Centre(x, y) => {
                self.cairo_context
                    .move_to(x - te.width / 2.0, y + te.height / 2.0);
            }
            TextPosn::TopLeftCorner(x, y) => {
                self.cairo_context.move_to(x, y + te.height);
            }
            TextPosn::TopRightCorner(x, y) => {
                self.cairo_context.move_to(x - te.width, y + te.height);
            }
            TextPosn::BottomLeftCorner(x, y) => {
                self.cairo_context.move_to(x, y);
            }
            TextPosn::BottomRightCorner(x, y) => {
                self.cairo_context.move_to(x - te.width, y);
            }
        }
        self.cairo_context
            .set_source_colour_rgb(&self.text_colour.get());
        self.cairo_context.show_text(&text);
    }

    fn paint_linear_gradient(&self, posn: Point, size: beigui::Size, colour_stops: &[(HCV, Prop)]) {
        let linear_gradient = cairo::LinearGradient::new(
            0.0,
            0.5 * f64::from(size.height),
            size.width.into(),
            0.5 * f64::from(size.height),
        );
        for colour_stop in colour_stops.iter() {
            let rgb = colour_stop.0.rgb::<f64>();
            linear_gradient.add_color_stop_rgb(
                colour_stop.1.into(),
                rgb[CCI::Red],
                rgb[CCI::Green],
                rgb[CCI::Blue],
            );
        }
        self.cairo_context.rectangle(
            posn.x.into(),
            posn.y.into(),
            size.width.into(),
            size.height.into(),
        );
        self.cairo_context.set_source(&linear_gradient);
        self.cairo_context.fill();
    }
}

impl<'a> DrawIsosceles for Drawer<'a> {}

impl<'a> DrawShapes for Drawer<'a> {
    fn set_background_colour(&self, colour: &impl ColourBasics) {
        self.cairo_context
            .set_source_colour_rgb(&colour.rgb::<f64>());
        self.cairo_context.paint();
    }

    fn draw_circle(&self, centre: Point, radius: UFDRNumber, fill: bool) {
        const TWO_PI: f64 = 2.0 * std::f64::consts::PI;
        self.cairo_context
            .arc(centre.x.into(), centre.y.into(), radius.into(), 0.0, TWO_PI);
        if fill {
            self.fill();
        } else {
            self.stroke();
        }
    }
}

impl<'a> Drawer<'a> {
    pub fn cartesian_transform_matrix(width: f64, height: f64) -> cairo::Matrix {
        let scale = if width > height {
            height / 2.15
        } else {
            width / 2.15
        };
        cairo::Matrix::new(scale, 0.0, 0.0, -scale, width / 2.0, height / 2.0)
    }
}
