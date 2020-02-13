// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::{cell::RefCell, rc::Rc};

use gtk::prelude::*;

use pw_gix::wrapper::*;

use crate::{
    attributes::{ColourAttributeDisplayStack, ColourAttributeDisplayStackBuilder},
    colour::{ScalarAttribute, RGB},
    manipulator::{ChromaLabel, RGBManipulatorGUI, RGBManipulatorGUIBuilder},
    rgb_entry::{RGBHexEntry, RGBHexEntryBuilder},
};

type ChangeCallback = Box<dyn Fn(RGB)>;

#[derive(PWO, Wrapper)]
pub struct ColourEditor {
    vbox: gtk::Box,
    rgb_manipulator: Rc<RGBManipulatorGUI>,
    cads: ColourAttributeDisplayStack,
    rgb_entry: Rc<RGBHexEntry<u8>>,
    change_callbacks: RefCell<Vec<ChangeCallback>>,
}

impl ColourEditor {
    fn inform_change(&self) {
        let rgb = self.rgb_manipulator.rgb();
        for callback in self.change_callbacks.borrow().iter() {
            callback(rgb)
        }
    }

    pub fn connect_changed<F: Fn(RGB) + 'static>(&self, callback: F) {
        self.change_callbacks.borrow_mut().push(Box::new(callback))
    }
}

#[derive(Default)]
pub struct ColourEditorBuilder {
    attributes: Vec<ScalarAttribute>,
    extra_buttons: Vec<gtk::Button>,
}

impl ColourEditorBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(&self) -> Rc<ColourEditor> {
        let cads = ColourAttributeDisplayStackBuilder::new()
            .attributes(&self.attributes)
            .build();
        let rgb_entry = RGBHexEntryBuilder::<u8>::new().build();
        let rgb_manipulator = RGBManipulatorGUIBuilder::new()
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
            rgb_manipulator,
            cads,
            rgb_entry,
            change_callbacks: RefCell::new(Vec::new()),
        });

        colour_editor
            .vbox
            .pack_start(&colour_editor.cads.pwo(), false, false, 0);
        colour_editor
            .vbox
            .pack_start(&colour_editor.rgb_entry.pwo(), false, false, 0);
        colour_editor
            .vbox
            .pack_start(&colour_editor.rgb_manipulator.pwo(), true, true, 0);

        colour_editor.vbox.show_all();

        let colour_editor_c = Rc::clone(&colour_editor);
        colour_editor.rgb_entry.connect_value_changed(move |rgb| {
            let rgb: RGB = rgb.into();
            colour_editor_c.cads.set_colour(Some(&rgb));
            colour_editor_c.rgb_manipulator.set_rgb(&rgb);
            colour_editor_c.inform_change();
        });

        let colour_editor_c = Rc::clone(&colour_editor);
        colour_editor.rgb_manipulator.connect_changed(move |rgb| {
            colour_editor_c.cads.set_colour(Some(&rgb));
            colour_editor_c.rgb_entry.set_rgb(&rgb.into());
            colour_editor_c.inform_change();
        });

        colour_editor
    }
}
