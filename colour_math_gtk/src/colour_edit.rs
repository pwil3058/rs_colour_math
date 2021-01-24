// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::{cell::RefCell, rc::Rc};

use pw_gix::{
    gtk::{self, prelude::*},
    wrapper::*,
};

use crate::colour::RGBConstants;
use crate::{
    attributes::{ColourAttributeDisplayStack, ColourAttributeDisplayStackBuilder},
    colour::{ScalarAttribute, RGB},
    manipulator::{ChromaLabel, ColourManipulatorGUI, ColourManipulatorGUIBuilder},
    rgb_entry::{RGBHexEntry, RGBHexEntryBuilder},
};

type ChangeCallback = Box<dyn Fn(&RGB)>;

#[derive(PWO, Wrapper)]
pub struct ColourEditor {
    vbox: gtk::Box,
    colour_manipulator: Rc<ColourManipulatorGUI>,
    cads: ColourAttributeDisplayStack,
    rgb_entry: Rc<RGBHexEntry<u8>>,
    change_callbacks: RefCell<Vec<ChangeCallback>>,
    default_colour: RGB,
}

impl ColourEditor {
    pub fn rgb(&self) -> RGB {
        self.colour_manipulator.rgb()
    }

    pub fn set_rgb(&self, rgb: &RGB) {
        self.rgb_entry.set_rgb(&rgb.into());
        self.colour_manipulator.set_rgb(rgb);
        self.cads.set_colour(Some(rgb));
    }

    pub fn reset(&self) {
        self.colour_manipulator.delete_samples();
        self.set_rgb(&self.default_colour);
    }

    fn inform_change(&self, rgb: &RGB) {
        for callback in self.change_callbacks.borrow().iter() {
            callback(rgb)
        }
    }

    pub fn connect_changed<F: Fn(&RGB) + 'static>(&self, callback: F) {
        self.change_callbacks.borrow_mut().push(Box::new(callback))
    }
}

#[derive(Default)]
pub struct ColourEditorBuilder {
    attributes: Vec<ScalarAttribute>,
    extra_buttons: Vec<gtk::Button>,
    default_colour: Option<RGB>,
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

    pub fn default_colour(&mut self, default_colour: &RGB) -> &mut Self {
        self.default_colour = Some(*default_colour);
        self
    }

    pub fn build(&self) -> Rc<ColourEditor> {
        let cads = ColourAttributeDisplayStackBuilder::new()
            .attributes(&self.attributes)
            .build();
        let rgb_entry = RGBHexEntryBuilder::<u8>::new().editable(true).build();
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

        let colour_editor = Rc::new(ColourEditor {
            vbox: gtk::Box::new(gtk::Orientation::Vertical, 0),
            colour_manipulator,
            cads,
            rgb_entry,
            change_callbacks: RefCell::new(Vec::new()),
            default_colour: if let Some(rgb) = self.default_colour {
                rgb
            } else {
                RGB::WHITE * 0.5
            },
        });

        colour_editor
            .vbox
            .pack_start(&colour_editor.cads.pwo(), false, false, 0);
        colour_editor
            .vbox
            .pack_start(&colour_editor.rgb_entry.pwo(), false, false, 0);
        colour_editor
            .vbox
            .pack_start(&colour_editor.colour_manipulator.pwo(), true, true, 0);

        colour_editor.vbox.show_all();

        let colour_editor_c = Rc::clone(&colour_editor);
        colour_editor.rgb_entry.connect_value_changed(move |rgb| {
            let rgb: RGB = rgb.into();
            colour_editor_c.cads.set_colour(Some(&rgb));
            colour_editor_c.colour_manipulator.set_rgb(&rgb);
            colour_editor_c.inform_change(&rgb);
        });

        let colour_editor_c = Rc::clone(&colour_editor);
        colour_editor
            .colour_manipulator
            .connect_changed(move |rgb| {
                colour_editor_c.cads.set_colour(Some(&rgb));
                colour_editor_c.rgb_entry.set_rgb(&rgb.into());
                colour_editor_c.inform_change(&rgb);
            });

        colour_editor
    }
}
