// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::cell::Cell;

use colour_math::attributes::drawing;

pub type Point = drawing::Point<f64>;
pub type Size = drawing::Size<f64>;
pub type TextPosn = drawing::TextPosn<f64>;

pub type RGB = colour_math::RGB<f64>;

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

    fn set_colour(&self, rgb: RGB) {
        self.cairo_context.set_source_rgb(rgb[0], rgb[1], rgb[2]);
    }

    fn fill(&self) {
        self.set_colour(self.fill_colour.get());
        self.cairo_context.fill();
    }

    fn stroke(&self) {
        self.set_colour(self.line_colour.get());
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
                colour_stop.0[0],
                colour_stop.0[1],
                colour_stop.0[2],
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
        self.set_colour(self.text_colour.get());
        self.cairo_context.show_text(&text);
    }
}
