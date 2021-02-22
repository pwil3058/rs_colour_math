// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

pub mod colour {
    pub use colour_math_ng::{
        beigui::{self, display},
        RGB,
    };
}

pub mod attributes {
    use std::{cell::RefCell, rc::Rc};

    use pw_gix::{
        gtk::{self, WidgetExt},
        wrapper::*,
    };

    use colour_math_cairo_ng::{Drawer, Size};

    use crate::colour::display;

    #[derive(PWO, Wrapper)]
    pub struct ColourAttributeDisplay<A: display::ColourAttributeDisplayIfce> {
        drawing_area: gtk::DrawingArea,
        attribute: RefCell<A>,
    }

    // pub type ChromaCAD = ColourAttributeDisplay<display::ChromaCAD>;
    // pub type GreynessCAD = ColourAttributeDisplay<display::GreynessCAD>;
    pub type HueCAD = ColourAttributeDisplay<display::HueCAD>;
    pub type ValueCAD = ColourAttributeDisplay<display::ValueCAD>;
    // pub type WarmthCAD = ColourAttributeDisplay<display::WarmthCAD>;

    impl<A> ColourAttributeDisplay<A>
    where
        A: display::ColourAttributeDisplayIfce + 'static,
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
}
