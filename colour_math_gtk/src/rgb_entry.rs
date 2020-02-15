// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    cell::{Cell, RefCell},
    cmp,
    fmt::UpperHex,
    rc::Rc,
};

use gtk::prelude::*;

use pw_gix::wrapper::*;

use crate::{
    colour::{HueConstants, RGB, URGB},
    coloured::Colourable,
};

type BoxedChangeCallback<U> = Box<dyn Fn(U)>;

#[derive(PWO)]
pub struct HexEntry<U> {
    entry: gtk::Entry,
    value: Cell<U>,
    current_step: Cell<U>,
    max_step: U,
    callbacks: RefCell<Vec<BoxedChangeCallback<U>>>,
}

impl<U> HexEntry<U>
where
    U: Copy
        + UpperHex
        + Ord
        + num_traits::Unsigned
        + num_traits::Bounded
        + num_traits_plus::NumberConstants
        + 'static,
{
    pub fn value(&self) -> U {
        self.value.get()
    }

    pub fn set_value(&self, value: U) {
        self.value.set(value);
        self.reset_entry_text();
    }

    pub fn connect_value_changed<F: 'static + Fn(U)>(&self, callback: F) {
        self.callbacks.borrow_mut().push(Box::new(callback))
    }

    fn incr_value(&self) {
        let value = self.value.get();
        let adj_incr = cmp::min(U::max_value() - value, self.current_step.get());
        if adj_incr > U::zero() {
            self.set_value_and_notify(value + adj_incr);
        }
        if self.value.get() < U::max_value() {
            self.bump_current_step()
        }
    }

    fn decr_value(&self) {
        let value = self.value.get();
        let adj_decr = cmp::min(value, self.current_step.get());
        if adj_decr > U::zero() {
            self.set_value_and_notify(value - adj_decr);
        }
        if self.value.get() > U::min_value() {
            self.bump_current_step()
        }
    }

    fn reset_entry_text(&self) {
        self.entry.set_text(&format!(
            "{:#0width$X}",
            self.value.get(),
            width = U::BYTES * 2 + 2
        ));
    }

    fn set_value_from_text(&self, text: &str) {
        let value = if let Some(index) = text.find("x") {
            U::from_str_radix(&text[index + 1..], 16)
        } else {
            U::from_str_radix(text, 16)
        };
        if let Ok(value) = value {
            self.set_value_and_notify(value);
        } else {
            self.reset_entry_text();
        }
    }

    fn set_value_and_notify(&self, value: U) {
        self.set_value(value);
        self.inform_value_changed();
    }

    fn inform_value_changed(&self) {
        let value = self.value.get();
        for callback in self.callbacks.borrow().iter() {
            callback(value);
        }
    }

    fn bump_current_step(&self) {
        let new_step = cmp::min(self.current_step.get() + U::one(), self.max_step);
        self.current_step.set(new_step);
    }

    fn reset_current_step(&self) {
        self.current_step.set(U::one());
    }
}

#[derive(Default)]
pub struct HexEntryBuilder<U>
where
    U: Default,
{
    initial_value: U,
    editable: bool,
}

impl<U> HexEntryBuilder<U>
where
    U: Default
        + num_traits_plus::NumberConstants
        + UpperHex
        + num_traits::Bounded
        + num_traits::Unsigned
        + num_traits::FromPrimitive
        + Ord
        + Copy
        + 'static,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn editable(&mut self, editable: bool) -> &mut Self {
        self.editable = editable;
        self
    }

    pub fn initial_value(&mut self, initial_value: U) -> &mut Self {
        self.initial_value = initial_value;
        self
    }

    #[allow(non_upper_case_globals)]
    pub fn build(&self) -> Rc<HexEntry<U>> {
        let entry = gtk::EntryBuilder::new()
            .width_chars(U::BYTES as i32 * 2 + 2)
            .editable(self.editable)
            .build();
        let value = Cell::new(self.initial_value);
        let max_step = cmp::max(U::max_value() / U::from_u8(32).unwrap(), U::one());
        let current_step = Cell::new(U::one());
        let callbacks: RefCell<Vec<Box<dyn Fn(U)>>> = RefCell::new(Vec::new());
        let hex_entry = Rc::new(HexEntry {
            entry,
            value,
            max_step,
            current_step,
            callbacks,
        });
        hex_entry.reset_entry_text();

        let hex_entry_c = Rc::clone(&hex_entry);
        hex_entry
            .entry
            .connect_key_press_event(move |entry, event| {
                use gdk::enums::key::*;
                let key = event.get_keyval();
                match key {
                    Return | Tab | ISO_Left_Tab => {
                        if let Some(text) = entry.get_text() {
                            hex_entry_c.set_value_from_text(&text);
                        } else {
                            hex_entry_c.reset_entry_text();
                        }
                        // NB: this will nobble the "activate" signal
                        // but let the Tab key move the focus
                        gtk::Inhibit(key == Return)
                    }
                    Up => {
                        hex_entry_c.incr_value();
                        gtk::Inhibit(true)
                    }
                    Down => {
                        hex_entry_c.decr_value();
                        gtk::Inhibit(true)
                    }
                    _0 | _1 | _2 | _3 | _4 | _5 | _6 | _7 | _8 | _9 | A | B | C | D | E | F
                    | BackSpace | Delete | Copy | Paste | x | a | b | c | d | e | f | Left
                    | Right => gtk::Inhibit(false),
                    _ => gtk::Inhibit(true),
                }
            });

        let hex_entry_c = Rc::clone(&hex_entry);
        hex_entry.entry.connect_key_release_event(move |_, event| {
            use gdk::enums::key::*;
            match event.get_keyval() {
                Up | Down => {
                    hex_entry_c.reset_current_step();
                    gtk::Inhibit(true)
                }
                _ => gtk::Inhibit(false),
            }
        });

        hex_entry
    }
}

#[derive(PWO)]
pub struct RGBHexEntry<U> {
    hbox: gtk::Box,
    entries: [Rc<HexEntry<U>>; 3],
    change_callbacks: RefCell<Vec<BoxedChangeCallback<URGB<U>>>>,
}

impl<U> RGBHexEntry<U>
where
    U: Default
        + UpperHex
        + num_traits::Bounded
        + num_traits::Unsigned
        + num_traits::FromPrimitive
        + num_traits_plus::NumberConstants
        + Ord
        + Copy
        + 'static,
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
pub struct RGBHexEntryBuilder<U>
where
    U: Default
        + num_traits_plus::NumberConstants
        + UpperHex
        + num_traits::Bounded
        + num_traits::Unsigned
        + num_traits::FromPrimitive
        + Ord
        + Copy
        + 'static,
{
    initial_rgb: URGB<U>,
    editable: bool,
}

impl<U> RGBHexEntryBuilder<U>
where
    U: Default
        + num_traits_plus::NumberConstants
        + UpperHex
        + num_traits::Bounded
        + num_traits::Unsigned
        + num_traits::FromPrimitive
        + Ord
        + Copy
        + 'static,
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
            label.set_widget_colour_rgb(&(*rgb).into());
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
