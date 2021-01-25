// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use pw_gix::{
    gtk::{self, prelude::*},
    wrapper::*,
};

use colour_math_gtk::{
    attributes::{AttributeSelectorBuilder, ColourAttributeDisplayStackBuilder},
    colour::{ColouredShape, HueConstants, ScalarAttribute, Shape, RGB},
    colour_edit::ColourEditorBuilder,
    hue_wheel::GtkHueWheelBuilder,
    //manipulator::ColourManipulatorGUIBuilder,
    //rgb_entry::RGBHexEntryBuilder,
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

    // let rgb_entry = RGBHexEntryBuilder::<u8>::new().build();
    // vbox.pack_start(&rgb_entry.pwo(), false, false, 0);
    //
    // let rgb_entry = RGBHexEntryBuilder::<u16>::new().build();
    // vbox.pack_start(&rgb_entry.pwo(), false, false, 0);
    //
    // let colour_manipulator = ColourManipulatorGUIBuilder::new().build();
    // vbox.pack_start(&colour_manipulator.pwo(), true, true, 0);

    let button1 = gtk::ButtonBuilder::new().label("Reset").build();
    let button2 = gtk::ButtonBuilder::new().label("button2").build();
    let colour_editor = ColourEditorBuilder::new()
        .attributes(&attributes)
        .extra_buttons(&[button1.clone(), button2.clone()])
        .build();
    colour_editor.connect_changed(move |rgb| cads.set_colour(Some(rgb)));
    let colour_editor_c = colour_editor.clone();
    button1.connect_clicked(move |_| colour_editor_c.reset());
    vbox.pack_start(&colour_editor.pwo(), true, true, 0);

    let gtk_hue_wheel = GtkHueWheelBuilder::new()
        .attributes(&attributes)
        .menu_item_specs(&[("add", ("Add", None, Some("Add something")).into(), 0)])
        .build();
    vbox.pack_start(&gtk_hue_wheel.pwo(), true, true, 0);
    gtk_hue_wheel.add_item(ColouredShape::new(
        RGB::RED,
        "Red",
        "Pure Red",
        Shape::Square,
    ));
    gtk_hue_wheel.add_item(ColouredShape::new(
        RGB::YELLOW,
        "Yellow",
        "Pure Yellow",
        Shape::Diamond,
    ));
    gtk_hue_wheel.add_item(ColouredShape::new(
        RGB::from([0.5, 0.5, 0.5]),
        "Grey",
        "Midle Grey",
        Shape::Circle,
    ));

    vbox.show_all();
    win.add(&vbox);
    win.connect_destroy(|_| gtk::main_quit());
    win.show();
    gtk::main()
}
