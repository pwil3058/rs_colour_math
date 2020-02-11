// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

pub mod colour {
    pub use colour_math::{
        attributes::ColourAttributeDisplayIfce, ColourInterface, ScalarAttribute,
    };
    pub type RGB = colour_math::rgb::RGB<f64>;
}

pub mod attributes {
    use std::{cell::RefCell, rc::Rc};

    use gtk::{BoxExt, RadioButtonExt, ToggleButtonExt, WidgetExt};

    pub use pw_gix::wrapper::*;

    use colour_math_cairo::{Drawer, Size};

    use crate::colour::{ColourAttributeDisplayIfce, ColourInterface, ScalarAttribute, RGB};

    type ChromaCAD = colour_math::attributes::ChromaCAD<f64>;
    type GreynessCAD = colour_math::attributes::GreynessCAD<f64>;
    type HueCAD = colour_math::attributes::HueCAD<f64>;
    type ValueCAD = colour_math::attributes::ValueCAD<f64>;
    type WarmthCAD = colour_math::attributes::WarmthCAD<f64>;

    pub trait DynColourAttributeDisplay: PackableWidgetObject<PWT = gtk::DrawingArea> {
        fn set_rgb(&self, rgb: Option<&RGB>);
        fn set_target_rgb(&self, rgb: Option<&RGB>);
    }

    #[derive(PWO, Wrapper)]
    pub struct ColourAttributeDisplayStack {
        vbox: gtk::Box,
        cads: Vec<Rc<dyn DynColourAttributeDisplay>>,
    }

    impl ColourAttributeDisplayStack {
        pub fn set_colour(&self, colour: Option<&impl ColourInterface<f64>>) {
            for cad in self.cads.iter() {
                if let Some(colour) = colour {
                    cad.set_rgb(Some(&colour.rgb()));
                } else {
                    cad.set_rgb(None);
                }
            }
        }

        pub fn set_target_colour(&self, colour: Option<&impl ColourInterface<f64>>) {
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
            let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
            let mut cads = vec![];
            let hue_cad: Rc<dyn DynColourAttributeDisplay> =
                ColourAttributeDisplay::<HueCAD>::new();
            vbox.pack_start(&hue_cad.pwo(), true, true, 0);
            cads.push(hue_cad);
            for scalar_attribute in self.attributes.iter() {
                let cad: Rc<dyn DynColourAttributeDisplay> = match scalar_attribute {
                    ScalarAttribute::Value => ColourAttributeDisplay::<ValueCAD>::new(),
                    ScalarAttribute::Chroma => ColourAttributeDisplay::<ChromaCAD>::new(),
                    ScalarAttribute::Warmth => ColourAttributeDisplay::<WarmthCAD>::new(),
                    ScalarAttribute::Greyness => ColourAttributeDisplay::<GreynessCAD>::new(),
                };
                vbox.pack_start(&cad.pwo(), true, true, 0);
                cads.push(cad);
            }
            ColourAttributeDisplayStack { vbox, cads }
        }
    }

    #[derive(PWO, Wrapper)]
    pub struct ColourAttributeDisplay<A: ColourAttributeDisplayIfce<f64> + 'static> {
        drawing_area: gtk::DrawingArea,
        attribute: RefCell<A>,
    }

    impl<A> ColourAttributeDisplay<A>
    where
        A: ColourAttributeDisplayIfce<f64> + 'static,
    {
        pub fn new() -> Rc<Self> {
            let cad = Rc::new(Self {
                drawing_area: gtk::DrawingArea::new(),
                attribute: RefCell::new(A::new()),
            });
            cad.drawing_area.set_size_request(90, 30);
            let cad_c = Rc::clone(&cad);
            cad.drawing_area.connect_draw(move |da, cairo_context| {
                let size: Size = Size {
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
        A: ColourAttributeDisplayIfce<f64> + 'static,
    {
        fn set_rgb(&self, rgb: Option<&RGB>) {
            self.attribute.borrow_mut().set_colour(rgb);
            self.drawing_area.queue_draw();
        }

        fn set_target_rgb(&self, rgb: Option<&RGB>) {
            self.attribute.borrow_mut().set_target_colour(rgb);
            self.drawing_area.queue_draw();
        }
    }

    type SelectionCallback = Box<dyn Fn(ScalarAttribute)>;

    #[derive(PWO)]
    pub struct AttributeSelector {
        gtk_box: gtk::Box,
        callbacks: RefCell<Vec<SelectionCallback>>,
    }

    impl AttributeSelector {
        pub fn connect_changed<F: Fn(ScalarAttribute) + 'static>(&self, callback: F) {
            self.callbacks.borrow_mut().push(Box::new(callback))
        }

        fn notify_changed(&self, attr: ScalarAttribute) {
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
                callbacks: RefCell::new(vec![]),
            });

            let mut first: Option<gtk::RadioButton> = None;
            for attr in self.attributes.iter() {
                let button = gtk::RadioButton::new_with_label(&attr.to_string());
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