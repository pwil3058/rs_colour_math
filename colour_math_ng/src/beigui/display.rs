// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::beigui::{Dirn, Draw, DrawIsosceles, Point, TextPosn};
use crate::{
    fdrn::{FDRNumber, UFDRNumber},
    ColourBasics, Prop, RGBConstants, HCV,
};

pub trait ColourAttributeDisplayIfce {
    const LABEL: &'static str;

    fn new() -> Self;

    fn set_colour(&mut self, colour: Option<&impl ColourBasics>);
    fn attr_value(&self) -> Option<Prop>;
    fn attr_value_fg_colour(&self) -> HCV;

    fn set_target_colour(&mut self, colour: Option<&impl ColourBasics>);
    fn attr_target_value(&self) -> Option<Prop>;
    fn attr_target_value_fg_colour(&self) -> HCV;

    fn label_colour(&self) -> HCV {
        match self.attr_value() {
            Some(_) => self.attr_value_fg_colour(),
            None => match self.attr_target_value() {
                Some(_) => self.attr_target_value_fg_colour(),
                None => HCV::BLACK,
            },
        }
    }

    fn colour_stops(&self) -> Vec<(HCV, Prop)> {
        vec![(HCV::BLACK, Prop::ZERO), (HCV::WHITE, Prop::ONE)]
    }

    fn draw_attr_value_indicator(&self, drawer: &impl DrawIsosceles) {
        if let Some(attr_value) = self.attr_value() {
            let size = drawer.size();
            let indicator_x = size.width * attr_value;
            drawer.set_fill_colour(&self.attr_value_fg_colour());
            drawer.set_line_colour(&self.attr_value_fg_colour());
            let base: UFDRNumber = UFDRNumber::ONE * 8;
            let height: UFDRNumber = UFDRNumber::ONE * 6;
            drawer.draw_isosceles(
                [indicator_x.into(), (height * 2).into()].into(),
                Dirn::Up,
                base,
                height,
                true,
            );
            drawer.draw_isosceles(
                [indicator_x.into(), (size.height - height * 2).into()].into(),
                Dirn::Down,
                base,
                height,
                true,
            );
        }
    }

    fn draw_target_attr_value_indicator(&self, drawer: &impl Draw) {
        if let Some(attr_value) = self.attr_target_value() {
            let size = drawer.size();
            let indicator_x: FDRNumber = (size.width * attr_value).into();
            drawer.set_line_width(UFDRNumber::ONE * 2);
            drawer.set_line_colour(&self.attr_target_value_fg_colour());
            drawer.draw_line(&[
                [indicator_x, FDRNumber::ONE].into(),
                [indicator_x, FDRNumber::from(size.height) - FDRNumber::ONE].into(),
            ]);
        }
    }

    fn draw_label(&self, drawer: &impl Draw) {
        if !Self::LABEL.is_empty() {
            let posn = TextPosn::Centre(drawer.size().centre());
            let font_size = UFDRNumber::ONE * 15;
            drawer.set_text_colour(&self.label_colour());
            drawer.draw_text(Self::LABEL, posn, font_size);
        }
    }

    fn draw_background(&self, drawer: &impl Draw) {
        let posn = Point::default();
        let size = drawer.size();
        drawer.paint_linear_gradient(posn, size, &self.colour_stops());
    }

    fn draw_all(&self, drawer: &impl DrawIsosceles) {
        self.draw_background(drawer);
        self.draw_target_attr_value_indicator(drawer);
        self.draw_attr_value_indicator(drawer);
        self.draw_label(drawer);
    }
}

// VALUE
pub struct ValueCAD {
    value: Option<Prop>,
    target_value: Option<Prop>,
    value_fg_colour: HCV,
    target_value_fg_colour: HCV,
}

impl ColourAttributeDisplayIfce for ValueCAD {
    const LABEL: &'static str = "Value";

    fn new() -> Self {
        Self {
            value: None,
            target_value: None,
            value_fg_colour: HCV::BLACK,
            target_value_fg_colour: HCV::BLACK,
        }
    }

    fn set_colour(&mut self, colour: Option<&impl ColourBasics>) {
        if let Some(colour) = colour {
            self.value = Some(colour.value());
            self.value_fg_colour = HCV::new_grey(colour.value()).best_foreground();
        } else {
            self.value = None;
            self.value_fg_colour = HCV::BLACK;
        }
    }

    fn attr_value(&self) -> Option<Prop> {
        self.value
    }

    fn attr_value_fg_colour(&self) -> HCV {
        self.value_fg_colour
    }

    fn set_target_colour(&mut self, colour: Option<&impl ColourBasics>) {
        if let Some(colour) = colour {
            self.target_value = Some(colour.value());
            self.target_value_fg_colour = HCV::new_grey(colour.value()).best_foreground();
        } else {
            self.target_value = None;
            self.target_value_fg_colour = HCV::BLACK;
        }
    }

    fn attr_target_value(&self) -> Option<Prop> {
        self.target_value
    }

    fn attr_target_value_fg_colour(&self) -> HCV {
        self.target_value_fg_colour
    }

    fn label_colour(&self) -> HCV {
        HCV::WHITE
    }
}
