// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use pw_gtk_ext::{
    cairo, gdk, gdk_pixbuf,
    gtk::{self, prelude::*, DrawingAreaBuilder},
    gtkx::menu::{ManagedMenu, ManagedMenuBuilder, MenuItemSpec},
    sav_state::{MaskedCondns, SAV_NEXT_CONDN},
    wrapper::*,
};

use colour_math::{
    fdrn::Prop,
    hcv::HCV,
    hue::angle::Angle,
    manipulator::{ColourManipulator, ColourManipulatorBuilder},
    LightLevel, Value, RGB,
};
use colour_math_cairo::Point;

use crate::colour::ManipGdkColour;
use crate::coloured::Colourable;

macro_rules! connect_button {
    ( $ed:ident, $btn:ident, $delta:ident, $apply:ident ) => {
        let ced_c = Rc::clone(&$ed);
        $ed.$btn.connect_clicked(move |btn| {
            let delta = ced_c.delta_size.get().$delta();
            let changed = ced_c.colour_manipulator.borrow_mut().$apply(delta);
            if changed {
                let new_hcv = ced_c.colour_manipulator.borrow().hcv();
                ced_c.set_colour_and_inform(&new_hcv);
            } else {
                btn.error_bell();
            }
        });
    };
}

pub const CAN_PASTE: u64 = SAV_NEXT_CONDN;
pub const CAN_REMOVE: u64 = SAV_NEXT_CONDN << 1;
pub const COMBINED_MASK: u64 = CAN_PASTE | CAN_REMOVE;

#[derive(Debug, PartialEq, Clone, Copy)]
enum DeltaSize {
    Small,
    Normal,
    Large,
}

impl DeltaSize {
    fn for_value(self) -> Prop {
        match self {
            DeltaSize::Small => 0.0025.into(),
            DeltaSize::Normal => 0.005.into(),
            DeltaSize::Large => 0.01.into(),
        }
    }

    fn for_chroma(self) -> Prop {
        match self {
            DeltaSize::Small => 0.0025.into(),
            DeltaSize::Normal => 0.005.into(),
            DeltaSize::Large => 0.01.into(),
        }
    }

    fn for_hue_anticlockwise(self) -> Angle {
        match self {
            DeltaSize::Small => 0.5.into(),
            DeltaSize::Normal => 1.0.into(),
            DeltaSize::Large => 5.0.into(),
        }
    }

    fn for_hue_clockwise(self) -> Angle {
        -self.for_hue_anticlockwise()
    }
}

struct Sample {
    pixbuf: gdk_pixbuf::Pixbuf,
    position: Point,
}

type ChangeCallback = Box<dyn Fn(HCV)>;

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
    popup_menu: ManagedMenu,
    popup_menu_posn: Cell<Point>,
    change_callbacks: RefCell<Vec<ChangeCallback>>,
}

impl ColourManipulatorGUI {
    pub fn set_colour(&self, colour: &impl ManipGdkColour) {
        self.colour_manipulator.borrow_mut().set_colour(colour);
        let offset: Prop = (Prop::ONE / 10 * 2).into();
        self.incr_value_btn
            .set_widget_colour(&colour.lightened(offset));
        self.decr_value_btn
            .set_widget_colour(&colour.darkened(offset));
        self.decr_chroma_btn
            .set_widget_colour(&colour.greyed(offset));
        self.incr_chroma_btn
            .set_widget_colour(&colour.saturated(offset));
        let angle_offset = Angle::from(45);
        self.hue_left_btn
            .set_widget_colour(&colour.rotated(angle_offset));
        self.hue_right_btn
            .set_widget_colour(&colour.rotated(-angle_offset));
        self.drawing_area.queue_draw();
    }

    fn set_colour_and_inform(&self, colour: &impl ManipGdkColour) {
        self.set_colour(colour);
        for callback in self.change_callbacks.borrow().iter() {
            callback(colour.hcv())
        }
    }

    fn draw(&self, cairo_context: &cairo::Context) {
        let rgb = self.colour_manipulator.borrow().rgb();
        cairo_context.set_source_rgb(rgb[0], rgb[1], rgb[2]);
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
        let mut npixels: u64 = 0;
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
                    for chunk in (data[row_start..row_end]).chunks(nc) {
                        red += chunk[0] as u64;
                        green += chunk[1] as u64;
                        blue += chunk[2] as u64;
                    }
                }
            }
            npixels += (width * n_rows) as u64;
        }
        if npixels > 0 {
            let divisor = npixels; //(npixels * 255) as u64;
            let array: [u8; 3] = [
                (red / divisor) as u8,
                (green / divisor) as u8,
                (blue / divisor) as u8,
            ];
            let rgb: RGB<u8> = array.into();
            self.set_colour_and_inform(&rgb);
        }
    }

    pub fn reset(&self) {
        self.delete_samples();
        self.set_colour_and_inform(&(HCV::new_grey(Value::ONE / 2)));
    }

    pub fn delete_samples(&self) {
        self.samples.borrow_mut().clear();
    }

    pub fn rgb<L: LightLevel>(&self) -> RGB<L> {
        self.colour_manipulator.borrow().rgb::<L>()
    }

    pub fn hcv(&self) -> HCV {
        self.colour_manipulator.borrow().hcv()
    }

    pub fn connect_changed<F: Fn(HCV) + 'static>(&self, callback: F) {
        self.change_callbacks.borrow_mut().push(Box::new(callback))
    }
}

#[derive(Clone, Copy, Default)]
pub enum ChromaLabel {
    #[default]
    Chroma,
    Greyness,
    Both,
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
            popup_menu: ManagedMenuBuilder::new().build(),
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
            Inhibit(false)
        });
        let rgbm_gui_c = Rc::clone(&rgbm_gui);
        rgbm_gui.vbox.connect_key_release_event(move |_, event| {
            let key = event.get_keyval();
            if key == gdk::keys::constants::Shift_L || key == gdk::keys::constants::Shift_R {
                rgbm_gui_c.delta_size.set(DeltaSize::Normal);
            };
            Inhibit(false)
        });
        let rgbm_gui_c = Rc::clone(&rgbm_gui);
        rgbm_gui.vbox.connect_enter_notify_event(move |_, _| {
            rgbm_gui_c.delta_size.set(DeltaSize::Normal);
            Inhibit(false)
        });

        let rgbm_gui_c = Rc::clone(&rgbm_gui);
        rgbm_gui.drawing_area.connect_draw(move |_, cctx| {
            rgbm_gui_c.draw(cctx);
            Inhibit(true)
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
        let menu_item_spec = MenuItemSpec::from((
            "Paste Sample",
            None,
            Some("Paste image sample from the clipboard at this position"),
        ));
        let rgbm_gui_c = Rc::clone(&rgbm_gui);
        rgbm_gui
            .popup_menu
            .append_item("paste", &menu_item_spec, CAN_PASTE)
            .expect("Duplicate menu item: paste")
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
        let menu_item_spec = MenuItemSpec::from((
            "Remove Sample(s)",
            None,
            Some("Remove all image samples from the sample area"),
        ));
        let rgbm_gui_c = Rc::clone(&rgbm_gui);
        rgbm_gui
            .popup_menu
            .append_item("remove", &menu_item_spec, CAN_REMOVE)
            .expect("Duplicate menu item: remove")
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
                    let mut condns = if cbd.wait_is_image_available() {
                        CAN_PASTE
                    } else {
                        0
                    };
                    if n_samples > 0 {
                        condns |= CAN_REMOVE
                    };
                    rgbm_gui_c.popup_menu.update_condns(MaskedCondns {
                        condns,
                        mask: COMBINED_MASK,
                    });
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
