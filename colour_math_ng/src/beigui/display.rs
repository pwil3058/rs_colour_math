// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::beigui::{Dirn, Draw, DrawIsosceles, Point, TextPosn};
use crate::{
    fdrn::{FDRNumber, UFDRNumber},
    ColourBasics, Hue, HueConstants, HueIfce, Prop, RGBConstants, HCV,
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
                None => self.attr_value_fg_colour(),
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

// HUE
pub struct HueCAD {
    hue: Option<Hue>,
    target_hue: Option<Hue>,
    hue_value: Option<Prop>,
    hue_fg_colour: HCV,
    target_hue_fg_colour: HCV,
    colour_stops: Vec<(HCV, Prop)>,
}

impl HueCAD {
    const DEFAULT_COLOUR_STOPS: [(HCV, Prop); 13] = [
        (HCV::CYAN, Prop::ZERO),
        (HCV::BLUE_CYAN, Prop(u64::MAX / 12)),
        (HCV::BLUE, Prop(u64::MAX / 6)),
        (HCV::BLUE_MAGENTA, Prop(u64::MAX / 12 * 3)),
        (HCV::MAGENTA, Prop(u64::MAX / 6 * 2)),
        (HCV::RED_MAGENTA, Prop(u64::MAX / 12 * 5)),
        (HCV::RED, Prop(u64::MAX / 6 * 3)),
        (HCV::RED_YELLOW, Prop(u64::MAX / 12 * 7)),
        (HCV::YELLOW, Prop(u64::MAX / 6 * 4)),
        (HCV::GREEN_YELLOW, Prop(u64::MAX / 12 * 9)),
        (HCV::GREEN, Prop(u64::MAX / 6 * 5)),
        (HCV::GREEN_CYAN, Prop(u64::MAX / 12 * 11)),
        (HCV::CYAN, Prop::ONE),
    ];

    fn set_colour_stops_for_hue(&mut self, hue: Hue) {
        let hue_angle = hue.angle();
        let stops: Vec<(HCV, Prop)> = Self::DEFAULT_COLOUR_STOPS
            .iter()
            .map(|(hcv, prop)| (*hcv + hue_angle, *prop))
            .collect();
        // let mut stops = vec![];
        // let mut hue = hue + Angle::from(180);
        // let delta = Angle::from(30);
        // for i in 0_u8..13 {
        //     let offset: Prop = (Prop::ONE * i / 12).into();
        //     let hcv = hue.max_chroma_hcv();
        //     stops.push((hcv, offset));
        //     hue = hue - delta;
        // }
        self.colour_stops = stops
    }

    fn set_colour_stops(&mut self, colour: Option<&impl ColourBasics>) {
        if let Some(colour) = colour {
            if let Some(hue) = colour.hue() {
                self.set_colour_stops_for_hue(hue);
            } else {
                self.colour_stops = Self::DEFAULT_COLOUR_STOPS.to_vec();
            }
        } else {
            self.colour_stops = Self::DEFAULT_COLOUR_STOPS.to_vec();
        }
    }

    fn set_defaults_for_no_hue(&mut self) {
        self.hue = None;
        self.hue_value = None;
        if let Some(target_hue) = self.target_hue {
            self.set_colour_stops_for_hue(target_hue);
        } else {
            self.colour_stops = Self::DEFAULT_COLOUR_STOPS.to_vec();
        }
    }

    fn set_defaults_for_no_target(&mut self) {
        self.target_hue = None;
        if let Some(hue) = self.hue {
            self.set_colour_stops_for_hue(hue);
            self.hue_value = Some(Prop::ONE / 2);
        } else {
            self.colour_stops = Self::DEFAULT_COLOUR_STOPS.to_vec();
        }
    }

    fn calc_hue_value(hue: Hue, target_hue: Hue) -> Prop {
        ((FDRNumber::ONE + FDRNumber::from(target_hue - hue) / 180) / 2).into()
    }
}

impl ColourAttributeDisplayIfce for HueCAD {
    const LABEL: &'static str = "Hue";

    fn new() -> Self {
        Self {
            hue: None,
            target_hue: None,
            hue_value: None,
            hue_fg_colour: HCV::WHITE,
            target_hue_fg_colour: HCV::BLACK,
            colour_stops: Self::DEFAULT_COLOUR_STOPS.to_vec(),
        }
    }

    fn set_colour(&mut self, colour: Option<&impl ColourBasics>) {
        if let Some(colour) = colour {
            if let Some(hue) = colour.hue() {
                self.hue = Some(hue);
                self.hue_fg_colour = hue.max_chroma_hcv().best_foreground();
                if let Some(target_hue) = self.target_hue {
                    self.hue_value = Some(Self::calc_hue_value(hue, target_hue));
                } else {
                    self.set_colour_stops(Some(colour));
                    self.hue_value = Some(Prop::ONE / 2);
                }
            } else {
                self.set_defaults_for_no_hue()
            }
        } else {
            self.set_defaults_for_no_hue()
        }
    }

    fn attr_value(&self) -> Option<Prop> {
        self.hue_value
    }

    fn attr_value_fg_colour(&self) -> HCV {
        self.hue_fg_colour
    }

    fn set_target_colour(&mut self, colour: Option<&impl ColourBasics>) {
        if let Some(colour) = colour {
            if let Some(target_hue) = colour.hue() {
                self.target_hue = Some(target_hue);
                self.target_hue_fg_colour = target_hue.max_chroma_hcv().best_foreground();
                self.set_colour_stops_for_hue(target_hue);
                if let Some(hue) = self.hue {
                    self.hue_value = Some(Self::calc_hue_value(hue, target_hue));
                }
            } else {
                self.set_defaults_for_no_target();
            }
        } else {
            self.set_defaults_for_no_target();
        }
    }

    fn attr_target_value(&self) -> Option<Prop> {
        if self.target_hue.is_some() {
            Some(Prop::ONE / 2)
        } else {
            None
        }
    }

    fn attr_target_value_fg_colour(&self) -> HCV {
        self.target_hue_fg_colour
    }

    fn colour_stops(&self) -> Vec<(HCV, Prop)> {
        self.colour_stops.clone()
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
