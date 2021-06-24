// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::{cell::RefCell, rc::Rc};

use pw_gix::{
    gtk::{self, prelude::*},
    wrapper::*,
};

use crate::{
    attributes::{ColourAttributeDisplayStack, ColourAttributeDisplayStackBuilder},
    colour::{GdkColour, LightLevel, ManipGdkColour, ScalarAttribute, Value, HCV, RGB},
    manipulator::{ChromaLabel, ColourManipulatorGUI, ColourManipulatorGUIBuilder},
    rgb_entry::{Hexable, RGBHexEntry, RGBHexEntryBuilder},
};

type ChangeCallback = Box<dyn Fn(&HCV)>;

#[derive(PWO, Wrapper)]
pub struct ColourEditor<U: Hexable> {
    vbox: gtk::Box,
    colour_manipulator: Rc<ColourManipulatorGUI>,
    cads: Rc<ColourAttributeDisplayStack>,
    rgb_entry: Rc<RGBHexEntry<U>>,
    change_callbacks: RefCell<Vec<ChangeCallback>>,
    default_colour: HCV,
}

impl<U: Hexable> ColourEditor<U> {
    pub fn rgb<L: LightLevel>(&self) -> RGB<L> {
        self.colour_manipulator.rgb()
    }

    pub fn hcv(&self) -> HCV {
        self.colour_manipulator.hcv()
    }

    pub fn set_colour(&self, colour: &impl ManipGdkColour) {
        self.rgb_entry.set_colour(colour);
        self.colour_manipulator.set_colour(colour);
        self.cads.set_colour(Some(colour));
    }

    pub fn reset(&self) {
        self.colour_manipulator.delete_samples();
        self.set_colour(&self.default_colour);
    }

    fn inform_change(&self, colour: &impl GdkColour) {
        for callback in self.change_callbacks.borrow().iter() {
            callback(&colour.hcv())
        }
    }

    pub fn connect_changed<F: Fn(&HCV) + 'static>(&self, callback: F) {
        self.change_callbacks.borrow_mut().push(Box::new(callback))
    }
}

#[derive(Default)]
pub struct ColourEditorBuilder {
    attributes: Vec<ScalarAttribute>,
    extra_buttons: Vec<gtk::Button>,
    default_colour: Option<HCV>,
}

impl ColourEditorBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn attributes(&mut self, attributes: &[ScalarAttribute]) -> &mut Self {
        self.attributes = attributes.to_vec();
        self
    }

    pub fn extra_buttons(&mut self, extra_buttons: &[gtk::Button]) -> &mut Self {
        self.extra_buttons = extra_buttons.to_vec();
        self
    }

    pub fn default_colour(&mut self, default_colour: &impl GdkColour) -> &mut Self {
        self.default_colour = Some(default_colour.hcv());
        self
    }

    pub fn build<U: Hexable>(&self) -> Rc<ColourEditor<U>> {
        let cads = ColourAttributeDisplayStackBuilder::new()
            .attributes(&self.attributes)
            .build();
        let rgb_entry = RGBHexEntryBuilder::<U>::new().editable(true).build();
        let colour_manipulator = ColourManipulatorGUIBuilder::new()
            .clamped(false)
            .extra_buttons(&self.extra_buttons)
            .chroma_label(if self.attributes.contains(&ScalarAttribute::Greyness) {
                if self.attributes.contains(&ScalarAttribute::Chroma) {
                    ChromaLabel::Both
                } else {
                    ChromaLabel::Greyness
                }
            } else {
                ChromaLabel::Chroma
            })
            .build();

        let colour_editor = Rc::new(ColourEditor::<U> {
            vbox: gtk::Box::new(gtk::Orientation::Vertical, 0),
            colour_manipulator,
            cads,
            rgb_entry,
            change_callbacks: RefCell::new(Vec::new()),
            default_colour: if let Some(rgb) = self.default_colour {
                rgb
            } else {
                HCV::new_grey(Value::ONE / 2)
            },
        });

        colour_editor
            .vbox
            .pack_start(colour_editor.cads.pwo(), false, false, 0);
        colour_editor
            .vbox
            .pack_start(colour_editor.rgb_entry.pwo(), false, false, 0);
        colour_editor
            .vbox
            .pack_start(colour_editor.colour_manipulator.pwo(), true, true, 0);

        colour_editor.vbox.show_all();

        let colour_editor_c = Rc::clone(&colour_editor);
        colour_editor.rgb_entry.connect_colour_changed(move |hcv| {
            colour_editor_c.cads.set_colour(Some(&hcv));
            colour_editor_c.colour_manipulator.set_colour(&hcv);
            colour_editor_c.inform_change(&hcv);
        });

        let colour_editor_c = Rc::clone(&colour_editor);
        colour_editor
            .colour_manipulator
            .connect_changed(move |hcv| {
                colour_editor_c.cads.set_colour(Some(&hcv));
                colour_editor_c.rgb_entry.set_colour(&hcv);
                colour_editor_c.inform_change(&hcv);
            });

        colour_editor
    }
}
