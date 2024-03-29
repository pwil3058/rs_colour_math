// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::{
    attributes::{Chroma, Greyness, Value, Warmth},
    beigui::{Dirn, Draw, DrawIsosceles, Point, TextPosn},
    fdrn::{FDRNumber, IntoProp, Prop, UFDRNumber},
    hcv::HCV,
    hue::{Hue, HueIfce},
    ColourBasics, HueConstants, RGBConstants,
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
                [indicator_x.into(), (height / 2).into()].into(),
                Dirn::Up,
                base,
                height,
                true,
            );
            drawer.draw_isosceles(
                [indicator_x.into(), (size.height - height / 2).into()].into(),
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
        (HCV::CYAN, Prop::ONE),
        (HCV::BLUE_CYAN, Prop(u64::MAX / 12 * 11)),
        (HCV::BLUE, Prop(u64::MAX / 12 * 10)),
        (HCV::BLUE_MAGENTA, Prop(u64::MAX / 12 * 9)),
        (HCV::MAGENTA, Prop(u64::MAX / 12 * 8)),
        (HCV::RED_MAGENTA, Prop(u64::MAX / 12 * 7)),
        (HCV::RED, Prop(u64::MAX / 12 * 6)),
        (HCV::RED_YELLOW, Prop(u64::MAX / 12 * 5)),
        (HCV::YELLOW, Prop(u64::MAX / 12 * 4)),
        (HCV::GREEN_YELLOW, Prop(u64::MAX / 12 * 3)),
        (HCV::GREEN, Prop(u64::MAX / 12 * 2)),
        (HCV::GREEN_CYAN, Prop(u64::MAX / 12)),
        (HCV::CYAN, Prop::ZERO),
    ];

    fn set_colour_stops_for_hue(&mut self, hue: Hue) {
        let hue_angle = hue.angle();
        let stops: Vec<(HCV, Prop)> = Self::DEFAULT_COLOUR_STOPS
            .iter()
            .map(|(hcv, prop)| (*hcv + hue_angle, *prop))
            .collect();
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

// Chroma
pub struct ChromaCAD {
    chroma: Option<Chroma>,
    target_chroma: Option<Chroma>,
    chroma_fg_colour: HCV,
    target_chroma_fg_colour: HCV,
    colour_stops: Vec<(HCV, Prop)>,
}

impl ChromaCAD {
    fn set_colour_stops(&mut self, colour: Option<&impl ColourBasics>) {
        self.colour_stops = if let Some(colour) = colour {
            if let Some(end_hcv) = colour.hue_hcv() {
                let start_hcv = colour.monochrome_hcv();
                vec![(start_hcv, Prop::ZERO), (end_hcv, Prop::ONE)]
            } else {
                let grey = colour.hcv();
                vec![(grey, Prop::ZERO), (grey, Prop::ONE)]
            }
        } else {
            Self::default_colour_stops()
        }
    }

    fn default_colour_stops() -> Vec<(HCV, Prop)> {
        let grey = HCV::new_grey(Value::ONE / 2);
        vec![(grey, Prop::ZERO), (grey, Prop::ONE)]
    }
}

impl ColourAttributeDisplayIfce for ChromaCAD {
    const LABEL: &'static str = "Chroma";

    fn new() -> Self {
        let grey = HCV::new_grey(Value::ONE / 2);
        Self {
            chroma: None,
            target_chroma: None,
            chroma_fg_colour: HCV::BLACK,
            target_chroma_fg_colour: HCV::BLACK,
            colour_stops: vec![(grey, Prop::ZERO), (grey, Prop::ONE)],
        }
    }

    fn set_colour(&mut self, colour: Option<&impl ColourBasics>) {
        if let Some(colour) = colour {
            self.chroma = Some(colour.chroma());
            self.chroma_fg_colour = colour.best_foreground();
            if let Some(target_chroma) = self.target_chroma {
                if target_chroma == Chroma::ZERO {
                    self.set_colour_stops(Some(colour));
                }
            } else {
                self.set_colour_stops(Some(colour));
            }
        } else {
            self.chroma = None;
            self.chroma_fg_colour = HCV::BLACK;
            if self.target_chroma.is_none() {
                self.colour_stops = Self::default_colour_stops()
            }
        }
    }

    fn attr_value(&self) -> Option<Prop> {
        self.chroma.map(|chroma| chroma.into_prop())
    }

    fn attr_value_fg_colour(&self) -> HCV {
        self.chroma_fg_colour
    }

    fn set_target_colour(&mut self, colour: Option<&impl ColourBasics>) {
        if let Some(colour) = colour {
            self.target_chroma = Some(colour.chroma());
            self.target_chroma_fg_colour = colour.monochrome_hcv().best_foreground();
            if colour.is_grey() {
                if let Some(chroma) = self.chroma {
                    if chroma == Chroma::ZERO {
                        self.set_colour_stops(Some(colour));
                    }
                } else {
                    self.set_colour_stops(Some(colour));
                }
            } else {
                self.set_colour_stops(Some(colour));
            }
        } else {
            self.target_chroma = None;
            self.target_chroma_fg_colour = HCV::BLACK;
        }
    }

    fn attr_target_value(&self) -> Option<Prop> {
        self.target_chroma.map(|chroma| chroma.into_prop())
    }

    fn attr_target_value_fg_colour(&self) -> HCV {
        self.target_chroma_fg_colour
    }

    fn label_colour(&self) -> HCV {
        HCV::WHITE
    }

    fn colour_stops(&self) -> Vec<(HCV, Prop)> {
        self.colour_stops.clone()
    }
}

// VALUE
pub struct ValueCAD {
    value: Option<Value>,
    target_value: Option<Value>,
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
            self.value_fg_colour = colour.monochrome_hcv().best_foreground();
        } else {
            self.value = None;
            self.value_fg_colour = HCV::BLACK;
        }
    }

    fn attr_value(&self) -> Option<Prop> {
        Some(self.value?.into())
    }

    fn attr_value_fg_colour(&self) -> HCV {
        self.value_fg_colour
    }

    fn set_target_colour(&mut self, colour: Option<&impl ColourBasics>) {
        if let Some(colour) = colour {
            self.target_value = Some(colour.value());
            self.target_value_fg_colour = colour.monochrome_hcv().best_foreground();
        } else {
            self.target_value = None;
            self.target_value_fg_colour = HCV::BLACK;
        }
    }

    fn attr_target_value(&self) -> Option<Prop> {
        Some(self.target_value?.into())
    }

    fn attr_target_value_fg_colour(&self) -> HCV {
        self.target_value_fg_colour
    }

    fn label_colour(&self) -> HCV {
        HCV::WHITE
    }
}

// Greyness
pub struct GreynessCAD {
    greyness: Option<Greyness>,
    target_greyness: Option<Greyness>,
    greyness_fg_colour: HCV,
    target_greyness_fg_colour: HCV,
    colour_stops: Vec<(HCV, Prop)>,
}

impl GreynessCAD {
    fn set_colour_stops(&mut self, colour: Option<&impl ColourBasics>) {
        self.colour_stops = if let Some(colour) = colour {
            if let Some(start_colour) = colour.hue_hcv() {
                let end_colour = colour.monochrome_hcv();
                vec![(start_colour, Prop::ZERO), (end_colour, Prop::ONE)]
            } else {
                let grey = colour.hcv();
                vec![(grey, Prop::ZERO), (grey, Prop::ONE)]
            }
        } else {
            Self::default_colour_stops()
        }
    }

    fn default_colour_stops() -> Vec<(HCV, Prop)> {
        let grey = HCV::new_grey(Value::ONE / 2);
        vec![(grey, Prop::ZERO), (grey, Prop::ONE)]
    }
}

impl ColourAttributeDisplayIfce for GreynessCAD {
    const LABEL: &'static str = "Greyness";

    fn new() -> Self {
        let grey = HCV::new_grey(Value::ONE / 2);
        Self {
            greyness: None,
            target_greyness: None,
            greyness_fg_colour: HCV::BLACK,
            target_greyness_fg_colour: HCV::BLACK,
            colour_stops: vec![(grey, Prop::ZERO), (grey, Prop::ONE)],
        }
    }

    fn set_colour(&mut self, colour: Option<&impl ColourBasics>) {
        if let Some(colour) = colour {
            self.greyness = Some(colour.greyness());
            self.greyness_fg_colour = colour.best_foreground();
            if let Some(target_greyness) = self.target_greyness {
                if target_greyness == Greyness::ZERO {
                    self.set_colour_stops(Some(colour));
                }
            } else {
                self.set_colour_stops(Some(colour));
            }
        } else {
            self.greyness = None;
            self.greyness_fg_colour = HCV::BLACK;
            if self.target_greyness.is_none() {
                self.colour_stops = Self::default_colour_stops()
            }
        }
    }

    fn attr_value(&self) -> Option<Prop> {
        self.greyness.map(|greyness| greyness.into_prop())
    }

    fn attr_value_fg_colour(&self) -> HCV {
        self.greyness_fg_colour
    }

    fn set_target_colour(&mut self, colour: Option<&impl ColourBasics>) {
        if let Some(colour) = colour {
            self.target_greyness = Some(colour.greyness());
            self.target_greyness_fg_colour = colour.monochrome_hcv().best_foreground();
            if colour.is_grey() {
                if let Some(greyness) = self.greyness {
                    if greyness == Greyness::ZERO {
                        self.set_colour_stops(Some(colour));
                    }
                } else {
                    self.set_colour_stops(Some(colour));
                }
            } else {
                self.set_colour_stops(Some(colour));
            }
        } else {
            self.target_greyness = None;
            self.target_greyness_fg_colour = HCV::BLACK;
        }
    }

    fn attr_target_value(&self) -> Option<Prop> {
        self.target_greyness.map(|greyness| greyness.into_prop())
    }

    fn attr_target_value_fg_colour(&self) -> HCV {
        self.target_greyness_fg_colour
    }

    fn label_colour(&self) -> HCV {
        HCV::WHITE
    }

    fn colour_stops(&self) -> Vec<(HCV, Prop)> {
        self.colour_stops.clone()
    }
}

// Warmth
pub struct WarmthCAD {
    warmth: Option<Warmth>,
    target_warmth: Option<Warmth>,
    warmth_fg_colour: HCV,
    target_warmth_fg_colour: HCV,
}

impl ColourAttributeDisplayIfce for WarmthCAD {
    const LABEL: &'static str = "Warmth";

    fn new() -> Self {
        Self {
            warmth: None,
            target_warmth: None,
            warmth_fg_colour: HCV::BLACK,
            target_warmth_fg_colour: HCV::BLACK,
        }
    }

    fn set_colour(&mut self, colour: Option<&impl ColourBasics>) {
        if let Some(colour) = colour {
            self.warmth = Some(colour.warmth());
            self.warmth_fg_colour = colour.monochrome_hcv().best_foreground();
        } else {
            self.warmth = None;
            self.warmth_fg_colour = HCV::BLACK;
        }
    }

    fn attr_value(&self) -> Option<Prop> {
        self.warmth.map(|warmth| warmth.into())
    }

    fn attr_value_fg_colour(&self) -> HCV {
        self.warmth_fg_colour
    }

    fn set_target_colour(&mut self, colour: Option<&impl ColourBasics>) {
        if let Some(colour) = colour {
            self.target_warmth = Some(colour.warmth());
            self.target_warmth_fg_colour = colour.monochrome_hcv().best_foreground();
        } else {
            self.target_warmth = None;
            self.target_warmth_fg_colour = HCV::BLACK;
        }
    }

    fn attr_target_value(&self) -> Option<Prop> {
        self.target_warmth.map(|target_warmth| target_warmth.into())
    }

    fn attr_target_value_fg_colour(&self) -> HCV {
        self.target_warmth_fg_colour
    }

    fn label_colour(&self) -> HCV {
        HCV::WHITE
    }

    fn colour_stops(&self) -> Vec<(HCV, Prop)> {
        vec![
            (HCV::WHITE, Prop::ZERO),
            (HCV::CYAN, Prop::ONE / 4),
            (HCV::new_grey(Value::ONE / 2), Prop::HALF),
            (HCV::RED, Prop::ONE),
        ]
    }
}
