// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use gtk::prelude::*;

use pw_gix::wrapper::*;

use colour_math_gtk::{
    attributes::{AttributeSelectorBuilder, ColourAttributeDisplayStackBuilder},
    colour::{ScalarAttribute, RGB},
    colour_edit::ColourEditorBuilder,
    hue_wheel::GtkHueWheelBuilder,
    manipulator::RGBManipulatorGUIBuilder,
    rgb_entry::RGBHexEntryBuilder,
};

fn main() {
    gtk::init().expect("nowhere to go if Gtk++ initialization fails");
    let win = gtk::Window::new(gtk::WindowType::Toplevel);
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let attributes = vec![ScalarAttribute::Value, ScalarAttribute::Chroma];

    let selector = AttributeSelectorBuilder::new()
        .attributes(&attributes)
        .build();
    selector.connect_changed(|sel| println!("Selected: {:?}", sel));
    vbox.pack_start(&selector.pwo(), false, false, 0);

    let cads = ColourAttributeDisplayStackBuilder::new()
        .attributes(&attributes)
        .build();
    cads.set_colour(Some(&RGB::from([0.1, 0.4, 0.7])));
    cads.set_target_colour(Some(&RGB::from([0.7, 0.4, 0.7])));
    vbox.pack_start(&cads.pwo(), true, true, 0);

    let rgb_entry = RGBHexEntryBuilder::<u8>::new().build();
    vbox.pack_start(&rgb_entry.pwo(), false, false, 0);

    let rgb_entry = RGBHexEntryBuilder::<u16>::new().build();
    vbox.pack_start(&rgb_entry.pwo(), false, false, 0);

    let colour_manipulator = RGBManipulatorGUIBuilder::new().build();
    vbox.pack_start(&colour_manipulator.pwo(), true, true, 0);

    let colour_editor = ColourEditorBuilder::new().build();
    vbox.pack_start(&colour_editor.pwo(), true, true, 0);

    let gtk_hue_wheel = GtkHueWheelBuilder::new().attributes(&attributes).build();
    vbox.pack_start(&gtk_hue_wheel.pwo(), true, true, 0);

    vbox.show_all();
    win.add(&vbox);
    win.connect_destroy(|_| gtk::main_quit());
    win.show();
    gtk::main()
}
