// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

pub mod colour {
    pub use colour_math::ScalarAttribute;
}

pub mod attributes {
    use std::{cell::RefCell, rc::Rc};

    use gtk::{BoxExt, RadioButtonExt, ToggleButtonExt};

    use pw_gix::wrapper::*;

    use crate::colour::ScalarAttribute;

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
