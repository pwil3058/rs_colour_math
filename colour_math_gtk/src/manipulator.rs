// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use pw_gix::{
    cairo, gdk, gdk_pixbuf,
    gtk::{self, prelude::*, DrawingAreaBuilder},
    gtkx::menu::WrappedMenu,
    wrapper::*,
};

use colour_math::{ColourInterface, RGBConstants, CCI};

use colour_math_cairo::Point;

use crate::{
    angles::Degrees,
    colour::{ColourManipulator, ColourManipulatorBuilder, RGB},
    coloured::Colourable,
};

macro_rules! connect_button {
    ( $ed:ident, $btn:ident, $delta:ident, $apply:ident ) => {
        let ced_c = Rc::clone(&$ed);
        $ed.$btn.connect_clicked(move |btn| {
            let delta = ced_c.delta_size.get().$delta();
            let changed = ced_c.colour_manipulator.borrow_mut().$apply(delta);
            if changed {
                let new_rgb = ced_c.colour_manipulator.borrow().rgb();
                ced_c.set_rgb_and_inform(&new_rgb);
            } else {
                btn.error_bell();
            }
        });
    };
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum DeltaSize {
    Small,
    Normal,
    Large,
}

impl DeltaSize {
    fn for_value(self) -> f64 {
        match self {
            DeltaSize::Small => 0.0025,
            DeltaSize::Normal => 0.005,
            DeltaSize::Large => 0.01,
        }
    }

    fn for_chroma(self) -> f64 {
        match self {
            DeltaSize::Small => 0.0025,
            DeltaSize::Normal => 0.005,
            DeltaSize::Large => 0.01,
        }
    }

    fn for_hue_anticlockwise(self) -> Degrees {
        match self {
            DeltaSize::Small => 0.5.into(),
            DeltaSize::Normal => 1.0.into(),
            DeltaSize::Large => 5.0.into(),
        }
    }

    fn for_hue_clockwise(self) -> Degrees {
        -self.for_hue_anticlockwise()
    }
}

struct Sample {
    pixbuf: gdk_pixbuf::Pixbuf,
    position: Point,
}

type ChangeCallback = Box<dyn Fn(RGB)>;

#[derive(PWO, Wrapper)]
pub struct ColourManipulatorGUI {
    vbox: gtk::Box,
    colour_manipulator: RefCell<ColourManipulator>,
    drawing_area: gtk::DrawingArea,
    incr_value_btn: gtk::Button,
    decr_value_btn: gtk::Button,
    hue_left_btn: gtk::Button,
    hue_right_btn: gtk::Button,
    decr_chroma_btn: gtk::Button,
    incr_chroma_btn: gtk::Button,
    delta_size: Cell<DeltaSize>,
    samples: RefCell<Vec<Sample>>,
    auto_match_btn: gtk::Button,
    auto_match_on_paste_btn: gtk::CheckButton,
    popup_menu: WrappedMenu,
    popup_menu_posn: Cell<Point>,
    change_callbacks: RefCell<Vec<ChangeCallback>>,
}

impl ColourManipulatorGUI {
    pub fn set_rgb(&self, rgb: &RGB) {
        self.colour_manipulator.borrow_mut().set_rgb(rgb);
        self.incr_value_btn
            .set_widget_colour_rgb(&(*rgb * 0.8 + RGB::WHITE * 0.2));
        self.decr_value_btn.set_widget_colour_rgb(&(*rgb * 0.8));
        if rgb.is_grey() {
            self.incr_chroma_btn.set_widget_colour_rgb(rgb);
            self.decr_chroma_btn.set_widget_colour_rgb(rgb);
            self.hue_left_btn.set_widget_colour_rgb(rgb);
            self.hue_right_btn.set_widget_colour_rgb(rgb);
        } else {
            let low_chroma_rgb = *rgb * 0.8 + rgb.monochrome_rgb() * 0.2;
            let high_chroma_rgb = *rgb * 0.8 + rgb.max_chroma_rgb() * 0.2;
            self.incr_chroma_btn.set_widget_colour_rgb(&high_chroma_rgb);
            self.decr_chroma_btn.set_widget_colour_rgb(&low_chroma_rgb);

            self.hue_left_btn
                .set_widget_colour_rgb(&rgb.components_rotated(Degrees::DEG_30));
            self.hue_right_btn
                .set_widget_colour_rgb(&rgb.components_rotated(-Degrees::DEG_30));
        }
        self.drawing_area.queue_draw();
    }

    fn set_rgb_and_inform(&self, rgb: &RGB) {
        self.set_rgb(rgb);
        for callback in self.change_callbacks.borrow().iter() {
            callback(*rgb)
        }
    }

    fn draw(&self, cairo_context: &cairo::Context) {
        let rgb = self.colour_manipulator.borrow().rgb();
        cairo_context.set_source_rgb(rgb[CCI::Red], rgb[CCI::Green], rgb[CCI::Blue]);
        cairo_context.paint();
        for sample in self.samples.borrow().iter() {
            let buffer = sample
                .pixbuf
                .save_to_bufferv("png", &[])
                .expect("pixbuf to png error");
            let mut reader = std::io::Cursor::new(buffer);
            let surface = cairo::ImageSurface::create_from_png(&mut reader).unwrap();
            cairo_context.set_source_surface(&surface, sample.position.x, sample.position.y);
            cairo_context.paint();
        }
    }

    fn auto_match_samples(&self) {
        let mut red: u64 = 0;
        let mut green: u64 = 0;
        let mut blue: u64 = 0;
        let mut npixels: u32 = 0;
        for sample in self.samples.borrow().iter() {
            assert_eq!(sample.pixbuf.get_bits_per_sample(), 8);
            let nc = sample.pixbuf.get_n_channels() as usize;
            let rs = sample.pixbuf.get_rowstride() as usize;
            let width = sample.pixbuf.get_width() as usize;
            let n_rows = sample.pixbuf.get_height() as usize;
            unsafe {
                let data = sample.pixbuf.get_pixels();
                for row_num in 0..n_rows {
                    let row_start = row_num * rs;
                    let row_end = row_start + width * nc;
                    for chunk in (&data[row_start..row_end]).chunks(nc) {
                        red += chunk[0] as u64;
                        green += chunk[1] as u64;
                        blue += chunk[2] as u64;
                    }
                }
            }
            npixels += (width * n_rows) as u32;
        }
        if npixels > 0 {
            let divisor = (npixels * 255) as f64;
            let array: [f64; 3] = [
                red as f64 / divisor,
                green as f64 / divisor,
                blue as f64 / divisor,
            ];
            self.set_rgb_and_inform(&array.into());
        }
    }

    pub fn reset(&self) {
        self.delete_samples();
        self.set_rgb_and_inform(&(RGB::WHITE * 0.5));
    }

    pub fn delete_samples(&self) {
        self.samples.borrow_mut().clear();
    }

    pub fn rgb(&self) -> RGB {
        self.colour_manipulator.borrow().rgb()
    }

    pub fn connect_changed<F: Fn(RGB) + 'static>(&self, callback: F) {
        self.change_callbacks.borrow_mut().push(Box::new(callback))
    }
}

#[derive(Clone, Copy)]
pub enum ChromaLabel {
    Chroma,
    Greyness,
    Both,
}

impl Default for ChromaLabel {
    fn default() -> ChromaLabel {
        ChromaLabel::Chroma
    }
}

#[derive(Default)]
pub struct ColourManipulatorGUIBuilder {
    chroma_label: ChromaLabel,
    extra_buttons: Vec<gtk::Button>,
    clamped: bool,
}

impl ColourManipulatorGUIBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn chroma_label(&mut self, chroma_label: ChromaLabel) -> &mut Self {
        self.chroma_label = chroma_label;
        self
    }

    pub fn clamped(&mut self, clamped: bool) -> &mut Self {
        self.clamped = clamped;
        self
    }

    pub fn extra_buttons(&mut self, extra_buttons: &[gtk::Button]) -> &mut Self {
        self.extra_buttons = extra_buttons.to_vec();
        self
    }

    pub fn build(&self) -> Rc<ColourManipulatorGUI> {
        let vbox = gtk::BoxBuilder::new()
            .orientation(gtk::Orientation::Vertical)
            .events(
                gdk::EventMask::KEY_PRESS_MASK
                    | gdk::EventMask::KEY_RELEASE_MASK
                    | gdk::EventMask::ENTER_NOTIFY_MASK,
            )
            .receives_default(true)
            .build();
        let colour_manipulator = RefCell::new(
            ColourManipulatorBuilder::new()
                .clamped(self.clamped)
                .build(),
        );
        let drawing_area = DrawingAreaBuilder::new()
            .events(gdk::EventMask::BUTTON_PRESS_MASK)
            .height_request(150)
            .width_request(150)
            .build();
        let rgbm_gui = Rc::new(ColourManipulatorGUI {
            vbox,
            colour_manipulator,
            drawing_area,
            incr_value_btn: gtk::Button::with_label("Value++"),
            decr_value_btn: gtk::Button::with_label("Value--"),
            hue_left_btn: gtk::Button::with_label("<"),
            hue_right_btn: gtk::Button::with_label(">"),
            decr_chroma_btn: gtk::Button::with_label("Chroma--"),
            incr_chroma_btn: gtk::Button::with_label("Chroma++"),
            delta_size: Cell::new(DeltaSize::Normal),
            samples: RefCell::new(vec![]),
            auto_match_btn: gtk::Button::with_label("Auto Match"),
            auto_match_on_paste_btn: gtk::CheckButton::with_label("On Paste?"),
            popup_menu: WrappedMenu::new(&[]),
            popup_menu_posn: Cell::new((0.0, 0.0).into()),
            change_callbacks: RefCell::new(Vec::new()),
        });

        rgbm_gui
            .vbox
            .pack_start(&rgbm_gui.incr_value_btn, false, false, 0);

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        hbox.pack_start(&rgbm_gui.hue_left_btn, false, false, 0);
        hbox.pack_start(&rgbm_gui.drawing_area, true, true, 0);
        hbox.pack_start(&rgbm_gui.hue_right_btn, false, false, 0);
        rgbm_gui.vbox.pack_start(&hbox, true, true, 0);

        rgbm_gui
            .vbox
            .pack_start(&rgbm_gui.decr_value_btn, false, false, 0);

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        match self.chroma_label {
            ChromaLabel::Chroma => {
                rgbm_gui.incr_chroma_btn.set_label("Chroma++");
                rgbm_gui.decr_chroma_btn.set_label("Chroma--");
                hbox.pack_start(&rgbm_gui.decr_chroma_btn, true, true, 0);
                hbox.pack_start(&rgbm_gui.incr_chroma_btn, true, true, 0);
            }
            ChromaLabel::Greyness => {
                rgbm_gui.incr_chroma_btn.set_label("Greyness--");
                rgbm_gui.decr_chroma_btn.set_label("Greyness++");
                hbox.pack_start(&rgbm_gui.incr_chroma_btn, true, true, 0);
                hbox.pack_start(&rgbm_gui.decr_chroma_btn, true, true, 0);
            }
            ChromaLabel::Both => {
                rgbm_gui.incr_chroma_btn.set_label("Chroma++/Greyness--");
                rgbm_gui.decr_chroma_btn.set_label("Chroma--/Greyness++");
                hbox.pack_start(&rgbm_gui.decr_chroma_btn, true, true, 0);
                hbox.pack_start(&rgbm_gui.incr_chroma_btn, true, true, 0);
            }
        }
        rgbm_gui.vbox.pack_start(&hbox, false, false, 0);

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        for button in self.extra_buttons.iter() {
            hbox.pack_start(button, true, true, 0);
        }
        hbox.pack_start(&rgbm_gui.auto_match_btn, true, true, 0);
        hbox.pack_start(&rgbm_gui.auto_match_on_paste_btn, false, false, 0);
        rgbm_gui.vbox.pack_start(&hbox, false, false, 0);

        rgbm_gui.vbox.show_all();

        let rgbm_gui_c = Rc::clone(&rgbm_gui);
        rgbm_gui.vbox.connect_key_press_event(move |_, event| {
            let key = event.get_keyval();
            if key == gdk::keys::constants::Shift_L {
                rgbm_gui_c.delta_size.set(DeltaSize::Large);
            } else if key == gdk::keys::constants::Shift_R {
                rgbm_gui_c.delta_size.set(DeltaSize::Small);
            };
            gtk::Inhibit(false)
        });
        let rgbm_gui_c = Rc::clone(&rgbm_gui);
        rgbm_gui.vbox.connect_key_release_event(move |_, event| {
            let key = event.get_keyval();
            if key == gdk::keys::constants::Shift_L || key == gdk::keys::constants::Shift_R {
                rgbm_gui_c.delta_size.set(DeltaSize::Normal);
            };
            gtk::Inhibit(false)
        });
        let rgbm_gui_c = Rc::clone(&rgbm_gui);
        rgbm_gui.vbox.connect_enter_notify_event(move |_, _| {
            rgbm_gui_c.delta_size.set(DeltaSize::Normal);
            gtk::Inhibit(false)
        });

        let rgbm_gui_c = Rc::clone(&rgbm_gui);
        rgbm_gui.drawing_area.connect_draw(move |_, cctx| {
            rgbm_gui_c.draw(cctx);
            gtk::Inhibit(true)
        });

        connect_button!(rgbm_gui, incr_value_btn, for_value, incr_value);
        connect_button!(rgbm_gui, decr_value_btn, for_value, decr_value);
        connect_button!(rgbm_gui, incr_chroma_btn, for_chroma, incr_chroma);
        connect_button!(rgbm_gui, decr_chroma_btn, for_chroma, decr_chroma);
        connect_button!(rgbm_gui, hue_left_btn, for_hue_anticlockwise, rotate);
        connect_button!(rgbm_gui, hue_right_btn, for_hue_clockwise, rotate);

        let rgbm_gui_c = Rc::clone(&rgbm_gui);
        rgbm_gui
            .auto_match_btn
            .connect_clicked(move |_| rgbm_gui_c.auto_match_samples());

        // POPUP
        let rgbm_gui_c = Rc::clone(&rgbm_gui);
        rgbm_gui
            .popup_menu
            .append_item(
                "paste",
                "Paste Sample",
                "Paste image sample from the clipboard at this position",
            )
            .connect_activate(move |_| {
                let cbd = gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD);
                if let Some(pixbuf) = cbd.wait_for_image() {
                    let sample = Sample {
                        pixbuf,
                        position: rgbm_gui_c.popup_menu_posn.get(),
                    };
                    rgbm_gui_c.samples.borrow_mut().push(sample);
                    if rgbm_gui_c.auto_match_on_paste_btn.get_active() {
                        rgbm_gui_c.auto_match_samples();
                    } else {
                        rgbm_gui_c.drawing_area.queue_draw();
                    };
                    rgbm_gui_c.auto_match_btn.set_sensitive(true);
                } else {
                    rgbm_gui_c.inform_user("No image data on clipboard.", None);
                }
            });
        let rgbm_gui_c = Rc::clone(&rgbm_gui);
        rgbm_gui
            .popup_menu
            .append_item(
                "remove",
                "Remove Sample(s)",
                "Remove all image samples from the sample area",
            )
            .connect_activate(move |_| {
                rgbm_gui_c.samples.borrow_mut().clear();
                rgbm_gui_c.drawing_area.queue_draw();
                rgbm_gui_c.auto_match_btn.set_sensitive(false);
            });
        let rgbm_gui_c = Rc::clone(&rgbm_gui);
        rgbm_gui
            .drawing_area
            .connect_button_press_event(move |_, event| {
                if event.get_event_type() == gdk::EventType::ButtonPress && event.get_button() == 3
                {
                    let position = Point::from(event.get_position());
                    let n_samples = rgbm_gui_c.samples.borrow().len();
                    let cbd = gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD);
                    rgbm_gui_c
                        .popup_menu
                        .set_sensitivities(cbd.wait_is_image_available(), &["paste"]);
                    rgbm_gui_c
                        .popup_menu
                        .set_sensitivities(n_samples > 0, &["remove"]);
                    rgbm_gui_c.popup_menu_posn.set(position);
                    rgbm_gui_c.popup_menu.popup_at_event(event);
                    return Inhibit(true);
                }
                Inhibit(false)
            });

        rgbm_gui.reset();

        rgbm_gui
    }
}
