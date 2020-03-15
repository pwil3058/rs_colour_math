// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{cell::RefCell, rc::Rc};

use gtk::prelude::*;

use pw_gix::{
    gtkx::entry::{HexEntry, HexEntryBuilder},
    wrapper::*,
};

use colour_math::urgb::UnsignedComponent;

use crate::{
    colour::{HueConstants, RGB, URGB},
    coloured::Colourable,
};

type BoxedChangeCallback<U> = Box<dyn Fn(U)>;

#[derive(PWO)]
pub struct RGBHexEntry<U>
where
    U: UnsignedComponent + std::ops::Shr<u8, Output = U> + 'static,
{
    hbox: gtk::Box,
    entries: [Rc<HexEntry<U>>; 3],
    change_callbacks: RefCell<Vec<BoxedChangeCallback<URGB<U>>>>,
}

impl<U> RGBHexEntry<U>
where
    U: UnsignedComponent + std::ops::Shr<u8, Output = U> + 'static,
{
    pub fn rgb(&self) -> URGB<U> {
        let v: Vec<U> = self.entries.iter().map(|e| e.value()).collect();
        URGB::<U>::from(&v[..])
    }

    pub fn set_rgb(&self, rgb: &URGB<U>) {
        for (entry, value) in self.entries.iter().zip(rgb.iter()) {
            entry.set_value(*value);
        }
    }

    pub fn connect_value_changed<F: 'static + Fn(URGB<U>)>(&self, callback: F) {
        self.change_callbacks.borrow_mut().push(Box::new(callback))
    }

    fn inform_value_changed(&self) {
        let urgb = self.rgb();
        for callback in self.change_callbacks.borrow().iter() {
            callback(urgb)
        }
    }
}

#[derive(Default)]
pub struct RGBHexEntryBuilder<U: UnsignedComponent>
where
    U: UnsignedComponent + std::ops::Shr<u8, Output = U> + 'static,
{
    initial_rgb: URGB<U>,
    editable: bool,
}

impl<U: UnsignedComponent> RGBHexEntryBuilder<U>
where
    U: UnsignedComponent + std::ops::Shr<u8, Output = U> + 'static,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn editable(&mut self, editable: bool) -> &mut Self {
        self.editable = editable;
        self
    }

    pub fn initial_rgb(&mut self, initial_rgb: &URGB<U>) -> &mut Self {
        self.initial_rgb = *initial_rgb;
        self
    }

    pub fn build(&self) -> Rc<RGBHexEntry<U>> {
        let hbox = gtk::BoxBuilder::new().build();

        let mut v: Vec<Rc<HexEntry<U>>> = vec![];
        for (index, (label, rgb)) in [
            ("Red:", RGB::RED),
            ("Green:", RGB::GREEN),
            ("Blue:", RGB::BLUE),
        ]
        .iter()
        .enumerate()
        {
            let entry = HexEntryBuilder::new()
                .editable(self.editable)
                .initial_value(self.initial_rgb[index as u8])
                .build();
            let label = gtk::Label::new(Some(label));
            label.set_widget_colour_rgb(rgb);
            hbox.pack_start(&label, true, true, 0);
            hbox.pack_start(&entry.pwo(), false, false, 0);
            v.push(entry);
        }
        let entries = [Rc::clone(&v[0]), Rc::clone(&v[1]), Rc::clone(&v[2])];

        let rgb_hex_entry = Rc::new(RGBHexEntry {
            hbox,
            entries,
            change_callbacks: RefCell::new(vec![]),
        });

        for entry in rgb_hex_entry.entries.iter() {
            let rgb_hex_entry_c = Rc::clone(&rgb_hex_entry);
            entry.connect_value_changed(move |_| rgb_hex_entry_c.inform_value_changed());
        }

        rgb_hex_entry
    }
}
