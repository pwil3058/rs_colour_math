// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

pub mod colour_edit;
pub mod hue_wheel;
pub mod manipulator;
pub mod rgb_entry;

pub mod colour {
    use pw_gtk_ext::gdk;

    use colour_math::{HCV, LightLevel, ManipulatedColour, RGB};

    pub trait GdkColour: colour_math::ColourIfce {
        fn gdk_rgba(&self) -> gdk::RGBA {
            let rgb = self.rgb::<f64>();
            gdk::RGBA {
                red: rgb[0],
                green: rgb[1],
                blue: rgb[2],
                alpha: 1.0,
            }
        }
    }

    impl<L: LightLevel> GdkColour for RGB<L> {}
    impl GdkColour for HCV {}

    pub trait ManipGdkColour: GdkColour + ManipulatedColour {}

    impl<L: LightLevel> ManipGdkColour for RGB<L> {}
    impl ManipGdkColour for HCV {}
}

pub mod coloured {
    use pw_gtk_ext::gtk::{self, prelude::*};

    use crate::colour::*;

    #[allow(deprecated)]
    pub trait Colourable: WidgetExt {
        fn set_widget_colour(&self, colour: &impl GdkColour) {
            let bg_gdk_rgba = colour.gdk_rgba();
            let fg_gdk_rgba = colour.best_foreground().gdk_rgba();
            self.override_background_color(gtk::StateFlags::empty(), Some(&bg_gdk_rgba));
            self.override_color(gtk::StateFlags::empty(), Some(&fg_gdk_rgba));
        }
    }

    #[allow(deprecated)]
    impl Colourable for gtk::Button {
        fn set_widget_colour(&self, colour: &impl GdkColour) {
            let bg_gdk_rgba = colour.gdk_rgba();
            let fg_gdk_rgba = colour.best_foreground().gdk_rgba();
            self.override_background_color(gtk::StateFlags::empty(), Some(&bg_gdk_rgba));
            self.override_color(gtk::StateFlags::empty(), Some(&fg_gdk_rgba));
            for child in self.get_children().iter() {
                child.set_widget_colour(colour);
            }
        }
    }

    impl Colourable for gtk::Bin {}
    impl Colourable for gtk::Box {}
    impl Colourable for gtk::ButtonBox {}
    impl Colourable for gtk::CheckButton {}
    impl Colourable for gtk::ComboBox {}
    impl Colourable for gtk::ComboBoxText {}
    impl Colourable for gtk::Container {}
    impl Colourable for gtk::Entry {}
    impl Colourable for gtk::EventBox {}
    impl Colourable for gtk::FlowBox {}
    impl Colourable for gtk::Frame {}
    impl Colourable for gtk::Grid {}
    impl Colourable for gtk::Label {}
    impl Colourable for gtk::LinkButton {}
    impl Colourable for gtk::MenuBar {}
    impl Colourable for gtk::RadioButton {}
    impl Colourable for gtk::Scrollbar {}
    impl Colourable for gtk::SpinButton {}
    impl Colourable for gtk::ToggleButton {}
    impl Colourable for gtk::ToolButton {}
    impl Colourable for gtk::Toolbar {}
    impl Colourable for gtk::Widget {}
}

pub mod attributes {
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };

    use pw_gtk_ext::{
        gtk::{self, BoxExt, RadioButtonExt, ToggleButtonExt, WidgetExt},
        wrapper::*,
    };

    use colour_math::{attr_display, RGB, ScalarAttribute};
    use colour_math_cairo::{Drawer, Size};

    use crate::colour::GdkColour;

    pub type ChromaCAD = ColourAttributeDisplay<attr_display::ChromaCAD>;
    pub type GreynessCAD = ColourAttributeDisplay<attr_display::GreynessCAD>;
    pub type HueCAD = ColourAttributeDisplay<attr_display::HueCAD>;
    pub type ValueCAD = ColourAttributeDisplay<attr_display::ValueCAD>;
    pub type WarmthCAD = ColourAttributeDisplay<attr_display::WarmthCAD>;

    pub trait DynColourAttributeDisplay: PackableWidgetObject<PWT = gtk::DrawingArea> {
        fn set_rgb(&self, rgb: Option<&RGB<f64>>);
        fn set_target_rgb(&self, rgb: Option<&RGB<f64>>);
    }

    #[derive(PWO, Wrapper)]
    pub struct ColourAttributeDisplayStack {
        vbox: gtk::Box,
        cads: Vec<Rc<dyn DynColourAttributeDisplay<PWT = gtk::DrawingArea>>>,
    }

    impl ColourAttributeDisplayStack {
        pub fn set_colour(&self, colour: Option<&impl GdkColour>) {
            for cad in self.cads.iter() {
                if let Some(colour) = colour {
                    cad.set_rgb(Some(&colour.rgb()));
                } else {
                    cad.set_rgb(None);
                }
            }
        }

        pub fn set_target_colour(&self, colour: Option<&impl GdkColour>) {
            for cad in self.cads.iter() {
                if let Some(colour) = colour {
                    cad.set_target_rgb(Some(&colour.rgb()));
                } else {
                    cad.set_target_rgb(None);
                }
            }
        }
    }

    #[derive(Default)]
    pub struct ColourAttributeDisplayStackBuilder {
        // TODO: add orientation as an option for CAD stacks
        attributes: Vec<ScalarAttribute>,
    }

    impl ColourAttributeDisplayStackBuilder {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn attributes(&mut self, attributes: &[ScalarAttribute]) -> &mut Self {
            self.attributes = attributes.to_vec();
            self
        }

        pub fn build(&self) -> Rc<ColourAttributeDisplayStack> {
            let vbox = gtk::Box::new(gtk::Orientation::Vertical, 1);
            let mut cads = vec![];
            let hue_cad: Rc<dyn DynColourAttributeDisplay<PWT = gtk::DrawingArea>> = HueCAD::new();
            vbox.pack_start(hue_cad.pwo(), true, true, 0);
            cads.push(hue_cad);
            for scalar_attribute in self.attributes.iter() {
                let cad: Rc<dyn DynColourAttributeDisplay<PWT = gtk::DrawingArea>> =
                    match scalar_attribute {
                        ScalarAttribute::Value => ValueCAD::new(),
                        ScalarAttribute::Chroma => ChromaCAD::new(),
                        ScalarAttribute::Warmth => WarmthCAD::new(),
                        ScalarAttribute::Greyness => GreynessCAD::new(),
                    };
                vbox.pack_start(cad.pwo(), true, true, 0);
                cads.push(cad);
            }
            Rc::new(ColourAttributeDisplayStack { vbox, cads })
        }
    }

    #[derive(PWO, Wrapper)]
    pub struct ColourAttributeDisplay<A: attr_display::ColourAttributeDisplayIfce> {
        drawing_area: gtk::DrawingArea,
        attribute: RefCell<A>,
    }

    impl<A> ColourAttributeDisplay<A>
    where
        A: attr_display::ColourAttributeDisplayIfce + 'static,
    {
        pub fn new() -> Rc<Self> {
            let cad = Rc::new(Self {
                drawing_area: gtk::DrawingArea::new(),
                attribute: RefCell::new(A::new()),
            });
            cad.drawing_area.set_size_request(90, 30);
            let cad_c = Rc::clone(&cad);
            cad.drawing_area.connect_draw(move |da, cairo_context| {
                let size = Size {
                    width: da.get_allocated_width() as f64,
                    height: da.get_allocated_height() as f64,
                };
                let drawer = Drawer::new(cairo_context, size);
                cad_c.attribute.borrow().draw_all(&drawer);
                gtk::Inhibit(false)
            });
            cad
        }
    }

    impl<A> DynColourAttributeDisplay for ColourAttributeDisplay<A>
    where
        A: attr_display::ColourAttributeDisplayIfce + 'static,
    {
        fn set_rgb(&self, rgb: Option<&RGB<f64>>) {
            self.attribute.borrow_mut().set_colour(rgb);
            self.drawing_area.queue_draw();
        }

        fn set_target_rgb(&self, rgb: Option<&RGB<f64>>) {
            self.attribute.borrow_mut().set_target_colour(rgb);
            self.drawing_area.queue_draw();
        }
    }

    type SelectionCallback = Box<dyn Fn(ScalarAttribute)>;

    #[derive(PWO)]
    pub struct AttributeSelector {
        gtk_box: gtk::Box,
        attribute: Cell<ScalarAttribute>,
        callbacks: RefCell<Vec<SelectionCallback>>,
    }

    impl AttributeSelector {
        pub fn attribute(&self) -> ScalarAttribute {
            self.attribute.get()
        }

        pub fn connect_changed<F: Fn(ScalarAttribute) + 'static>(&self, callback: F) {
            self.callbacks.borrow_mut().push(Box::new(callback))
        }

        fn notify_changed(&self, attr: ScalarAttribute) {
            self.attribute.set(attr);
            for callback in self.callbacks.borrow().iter() {
                callback(attr);
            }
        }
    }

    pub struct AttributeSelectorBuilder {
        attributes: Vec<ScalarAttribute>,
        orientation: gtk::Orientation,
    }

    impl Default for AttributeSelectorBuilder {
        fn default() -> Self {
            Self {
                attributes: vec![],
                orientation: gtk::Orientation::Horizontal,
            }
        }
    }

    impl AttributeSelectorBuilder {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn attributes(&mut self, attributes: &[ScalarAttribute]) -> &mut Self {
            self.attributes = attributes.to_vec();
            self
        }

        pub fn orientation(&mut self, orientation: gtk::Orientation) -> &mut Self {
            self.orientation = orientation;
            self
        }

        pub fn build(&self) -> Rc<AttributeSelector> {
            let asrb = Rc::new(AttributeSelector {
                gtk_box: gtk::Box::new(self.orientation, 0),
                attribute: Cell::new(*self.attributes.first().expect("programmer error")),
                callbacks: RefCell::new(vec![]),
            });

            let mut first: Option<gtk::RadioButton> = None;
            for attr in self.attributes.iter() {
                let button = gtk::RadioButton::with_label(&attr.to_string());
                asrb.gtk_box.pack_start(&button, false, false, 0);
                if let Some(ref first) = first {
                    button.join_group(Some(first))
                } else {
                    first = Some(button.clone())
                }
                let asrb_c = Rc::clone(&asrb);
                let attr = *attr;
                button.connect_toggled(move |button| {
                    let its_us = button.get_active();
                    if its_us {
                        asrb_c.notify_changed(attr);
                    }
                });
            }

            asrb
        }
    }
}
