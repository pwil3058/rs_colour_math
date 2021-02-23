// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    env,
    path::{Path, PathBuf},
};

use pw_pathux::expand_home_dir_or_mine;

use pw_gix::{
    gtk::{self, prelude::*},
    gtkx::window::RememberGeometry,
    recollections,
    wrapper::*,
};

const DEFAULT_CONFIG_DIR_PATH: &str = "~/.config/test_gui_gtk_ng";

const DCDP_OVERRIDE_ENVAR: &str = "COLOUR_MATH_NG_TEST_GUI_GTK_CONFIG_DIR";

fn abs_default_config_dir_path() -> PathBuf {
    expand_home_dir_or_mine(&Path::new(DEFAULT_CONFIG_DIR_PATH))
}

pub fn config_dir_path() -> PathBuf {
    match env::var(DCDP_OVERRIDE_ENVAR) {
        Ok(dir_path) => {
            if dir_path.is_empty() {
                abs_default_config_dir_path()
            } else if dir_path.starts_with('~') {
                expand_home_dir_or_mine(&Path::new(&dir_path))
            } else {
                dir_path.into()
            }
        }
        Err(_) => abs_default_config_dir_path(),
    }
}

pub fn gui_config_dir_path() -> PathBuf {
    config_dir_path().join("gui")
}

pub fn recollection_file_path() -> PathBuf {
    gui_config_dir_path().join("recollections")
}

use colour_math_gtk_ng::{
    attributes::ColourAttributeDisplayStackBuilder,
    colour::{ScalarAttribute, RGB},
};

fn main() {
    gtk::init().expect("nowhere to go if Gtk++ initialization fails");
    recollections::init(&recollection_file_path());
    let win = gtk::Window::new(gtk::WindowType::Toplevel);
    win.set_geometry_from_recollections("main_window", (600, 400));
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let attributes = vec![
        ScalarAttribute::Value,
        ScalarAttribute::Chroma,
        ScalarAttribute::Greyness,
    ];

    let cads = ColourAttributeDisplayStackBuilder::new()
        .attributes(&attributes)
        .build();
    cads.set_colour(Some(&RGB::from([0.1, 0.4, 0.7])));
    cads.set_target_colour(Some(&RGB::from([0.7, 0.4, 0.7])));
    vbox.pack_start(&cads.pwo(), true, true, 0);

    vbox.show_all();
    win.add(&vbox);
    win.connect_destroy(|_| gtk::main_quit());
    win.show();
    gtk::main()
}
