// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    env,
    path::{Path, PathBuf},
    rc::Rc,
};

use pw_pathux::expand_home_dir_or_mine;

use pw_gix::{
    gtk::{self, prelude::*, MessageDialogBuilder},
    gtkx::window::RememberGeometry,
    recollections, sample,
    wrapper::*,
};

const DEFAULT_CONFIG_DIR_PATH: &str = "~/.config/test_gui_gtk";

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

use colour_math_gtk::{
    attributes::ColourAttributeDisplayStackBuilder,
    colour::{
        beigui::hue_wheel::Shape, ColouredShape, HueConstants, Prop, ScalarAttribute, HCV, RGB,
    },
    colour_edit::ColourEditorBuilder,
    hue_wheel::GtkHueWheelBuilder,
};

fn main() {
    gtk::init().expect("nowhere to go if Gtk++ initialization fails");
    recollections::init(&recollection_file_path());
    let win = gtk::Window::new(gtk::WindowType::Toplevel);
    win.set_geometry_from_recollections("main_window", (600, 400));
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);

    if sample::screen_sampling_available() {
        let btn = gtk::Button::with_label("Take Sample");
        btn.set_tooltip_text(Some("Take a sample of a portion of the screen"));
        let win_c = win.clone();
        btn.connect_clicked(move |_| {
            if let Err(err) = sample::take_screen_sample() {
                let msg = format!("Failure: {:?}", err);
                let dialog = MessageDialogBuilder::new()
                    .parent(&win_c)
                    .text(&msg)
                    .build();
                dialog.show()
            }
        });
        vbox.pack_start(&btn, false, false, 0);
    }

    let attributes = vec![
        ScalarAttribute::Value,
        ScalarAttribute::Chroma,
        ScalarAttribute::Greyness,
        ScalarAttribute::Warmth,
    ];

    let cads = ColourAttributeDisplayStackBuilder::new()
        .attributes(&attributes)
        .build();
    cads.set_colour(Some(&RGB::from([0.1, 0.4, 0.7])));
    cads.set_target_colour(Some(&RGB::from([0.7, 0.4, 0.7])));
    vbox.pack_start(&cads.pwo(), true, true, 0);
    //
    // let rgb_hex_entry = RGBHexEntryBuilder::<u16>::new()
    //     .initial_colour(&RGB::from([0.1, 0.4, 0.7]))
    //     .editable(true)
    //     .build();
    // let cads_c = Rc::clone(&cads);
    // rgb_hex_entry.connect_colour_changed(move |c| cads_c.set_colour(Some(&c)));
    // vbox.pack_start(&rgb_hex_entry.pwo(), false, false, 0);
    //
    // let colour_manipulator = ColourManipulatorGUIBuilder::new().build();
    // let hex_entry_c = Rc::clone(&rgb_hex_entry);
    // colour_manipulator.connect_changed(move |c| hex_entry_c.set_colour(&c));
    // let cads_c = Rc::clone(&cads);
    // colour_manipulator.connect_changed(move |c| cads_c.set_colour(Some(&c)));
    // vbox.pack_start(&colour_manipulator.pwo(), true, true, 0);

    let gtk_hue_wheel = GtkHueWheelBuilder::new()
        .attributes(&attributes)
        .menu_item_specs(&[("add", ("Add", None, Some("Add something")).into(), 0)])
        .build();
    vbox.pack_start(&gtk_hue_wheel.pwo(), true, true, 0);
    gtk_hue_wheel.add_item(ColouredShape::new(
        &HCV::RED,
        "Red",
        "Pure Red",
        Shape::Square,
    ));
    gtk_hue_wheel.add_item(ColouredShape::new(
        &HCV::YELLOW,
        "Yellow",
        "Pure Yellow",
        Shape::Diamond,
    ));
    gtk_hue_wheel.add_item(ColouredShape::new(
        &HCV::new_grey(Prop::ONE / 2),
        "Grey",
        "Midle Grey",
        Shape::Circle,
    ));

    let colour_editor = ColourEditorBuilder::new()
        .attributes(&attributes)
        .build::<u16>();
    let cads_c = Rc::clone(&cads);
    colour_editor.connect_changed(move |c| cads_c.set_colour(Some(c)));
    vbox.pack_start(&colour_editor.pwo(), true, true, 0);

    vbox.show_all();
    win.add(&vbox);
    win.connect_destroy(|_| gtk::main_quit());
    win.show();
    gtk::main()
}

#[cfg(test)]
mod tests {
    use super::*;

    use colour_math::*;
    use colour_math_derive::*;

    #[derive(Colour)]
    pub struct ColourWrapper {
        colour: HCV,
        _dummy: u64,
    }

    #[derive(Colour)]
    pub struct RGBWrapper {
        _dummy: u64,
        #[colour]
        rgb: RGB<u64>,
    }

    #[test]
    fn colour_wrapper() {
        let colour_wrapper = ColourWrapper {
            colour: HCV::YELLOW,
            _dummy: 0,
        };
        assert_eq!(colour_wrapper.rgb::<u64>(), RGB::<u64>::YELLOW);
    }

    #[test]
    fn rgb_wrapper() {
        let rgb_wrapper = RGBWrapper {
            rgb: RGB::CYAN,
            _dummy: 0,
        };
        assert_eq!(rgb_wrapper.hcv(), HCV::CYAN);
    }
}
