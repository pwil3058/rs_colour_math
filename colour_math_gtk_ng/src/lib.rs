// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

pub mod colour {
    pub use colour_math_ng::{
        beigui::{self, attr_display},
        ColourBasics, ScalarAttribute, RGB,
    };
}

pub mod attributes {
    use std::{cell::RefCell, rc::Rc};

    use pw_gix::{
        gtk::{self, BoxExt, WidgetExt},
        wrapper::*,
    };

    use colour_math_cairo_ng::{Drawer, Size};

    use crate::colour::{attr_display, ColourBasics, ScalarAttribute, RGB};

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
        cads: Vec<Rc<dyn DynColourAttributeDisplay>>,
    }

    impl ColourAttributeDisplayStack {
        pub fn set_colour(&self, colour: Option<&impl ColourBasics>) {
            for cad in self.cads.iter() {
                if let Some(colour) = colour {
                    cad.set_rgb(Some(&colour.rgb()));
                } else {
                    cad.set_rgb(None);
                }
            }
        }

        pub fn set_target_colour(&self, colour: Option<&impl ColourBasics>) {
            for cad in self.cads.iter() {
                if let Some(colour) = colour {
                    cad.set_target_rgb(Some(&colour.rgb()));
                } else {
                    cad.set_target_rgb(None);
                }
            }
        }
    }

    pub struct ColourAttributeDisplayStackBuilder {
        // TODO: add orientation as an option for CAD stacks
        attributes: Vec<ScalarAttribute>,
    }

    impl Default for ColourAttributeDisplayStackBuilder {
        fn default() -> Self {
            Self { attributes: vec![] }
        }
    }

    impl ColourAttributeDisplayStackBuilder {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn attributes(&mut self, attributes: &[ScalarAttribute]) -> &mut Self {
            self.attributes = attributes.to_vec();
            self
        }

        pub fn build(&self) -> ColourAttributeDisplayStack {
            let vbox = gtk::Box::new(gtk::Orientation::Vertical, 1);
            let mut cads = vec![];
            let hue_cad: Rc<dyn DynColourAttributeDisplay> = HueCAD::new();
            vbox.pack_start(&hue_cad.pwo(), true, true, 0);
            cads.push(hue_cad);
            for scalar_attribute in self.attributes.iter() {
                let cad: Rc<dyn DynColourAttributeDisplay> = match scalar_attribute {
                    ScalarAttribute::Value => ValueCAD::new(),
                    ScalarAttribute::Chroma => ChromaCAD::new(),
                    ScalarAttribute::Warmth => WarmthCAD::new(),
                    ScalarAttribute::Greyness => GreynessCAD::new(),
                };
                vbox.pack_start(&cad.pwo(), true, true, 0);
                cads.push(cad);
            }
            ColourAttributeDisplayStack { vbox, cads }
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
}
