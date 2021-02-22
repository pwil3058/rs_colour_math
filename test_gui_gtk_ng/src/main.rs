// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use pw_gix::{
    gtk::{self, prelude::*},
    wrapper::*,
};

use colour_math_gtk_ng::attributes::{HueCAD, ValueCAD};

fn main() {
    gtk::init().expect("nowhere to go if Gtk++ initialization fails");
    let win = gtk::Window::new(gtk::WindowType::Toplevel);
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);

    let hue_cad = HueCAD::new();
    vbox.pack_start(&hue_cad.pwo(), true, true, 0);

    let value_cad = ValueCAD::new();
    vbox.pack_start(&value_cad.pwo(), true, true, 0);

    vbox.show_all();
    win.add(&vbox);
    win.connect_destroy(|_| gtk::main_quit());
    win.show();
    gtk::main()
}
