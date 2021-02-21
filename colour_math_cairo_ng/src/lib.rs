// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::cell::Cell;

use pw_gix::cairo;

use colour_math_ng::{
    beigui::{Draw, DrawIsosceles, Point, Size, TextPosn},
    ColourBasics, Prop, UFDRNumber, CCI, HCV,
};

pub type RGB = colour_math_ng::RGB<f64>;

pub trait CairoSetColour {
    fn set_source_colour_rgb(&self, rgb: &RGB);
}

impl CairoSetColour for pw_gix::cairo::Context {
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
    fn size(&self) -> Size {
        self.size
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

    fn draw_text(&self, text: &str, posn: TextPosn, font_size: UFDRNumber) {
        if text.is_empty() {
            return;
        }
        self.cairo_context.set_font_size(font_size.into());
        let te = self.cairo_context.text_extents(&text);
        match posn {
            TextPosn::Centre(point) => {
                self.cairo_context.move_to(
                    f64::from(point.x) - te.width / 2.0,
                    f64::from(point.y) + te.height / 2.0,
                );
            }
            TextPosn::TopLeftCorner(point) => {
                self.cairo_context
                    .move_to(f64::from(point.x), f64::from(point.y) + te.height);
            }
            TextPosn::TopRightCorner(point) => {
                self.cairo_context.move_to(
                    f64::from(point.x) - te.width,
                    f64::from(point.y) + te.height,
                );
            }
            TextPosn::BottomLeftCorner(point) => {
                self.cairo_context
                    .move_to(f64::from(point.x), f64::from(point.y));
            }
            TextPosn::BottomRightCorner(point) => {
                self.cairo_context
                    .move_to(f64::from(point.x) - te.width, f64::from(point.y));
            }
        }
        self.cairo_context
            .set_source_colour_rgb(&self.text_colour.get());
        self.cairo_context.show_text(&text);
    }

    fn paint_linear_gradient(&self, posn: Point, size: Size, colour_stops: &[(HCV, Prop)]) {
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
