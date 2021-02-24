// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

use pw_gix::{
    cairo, gdk,
    gtk::{self, prelude::*},
    gtkx::menu_ng::{ManagedMenu, ManagedMenuBuilder, MenuItemSpec},
    sav_state::MaskedCondns,
    wrapper::*,
};

use colour_math_cairo_ng::*;

use crate::{
    attributes::{AttributeSelector, AttributeSelectorBuilder},
    colour::{ColourBasics, ColouredShape, HueWheel, ScalarAttribute},
};

type PopupCallback = Box<dyn Fn(&str)>;

#[derive(PWO, Wrapper)]
pub struct GtkHueWheel {
    vbox: gtk::Box,
    drawing_area: gtk::DrawingArea,
    hue_wheel: RefCell<HueWheel>,
    chosen_item: RefCell<Option<String>>,
    attribute_selector: Rc<AttributeSelector>,
    popup_menu: ManagedMenu,
    callbacks: RefCell<HashMap<String, Vec<PopupCallback>>>,
    origin_offset: Cell<Point>,
    last_xy: Cell<Option<Point>>,
}

impl GtkHueWheel {
    fn current_transform_matrix(&self) -> cairo::Matrix {
        let origin_offset = self.origin_offset.get();
        let mut ctm = CairoCartesian::cartesian_transform_matrix(
            self.drawing_area.get_allocated_width() as f64,
            self.drawing_area.get_allocated_height() as f64,
        );
        ctm.translate(origin_offset.x.into(), origin_offset.y.into());
        ctm
    }

    fn device_to_user(&self, x: f64, y: f64) -> Point {
        let mut ctm = self.current_transform_matrix();
        ctm.invert();
        ctm.transform_point(x, y).into()
    }

    fn device_to_user_delta(&self, point: Point) -> Point {
        let mut ctm = self.current_transform_matrix();
        ctm.invert();
        ctm.transform_distance(point.x, point.y).into()
    }

    fn shift_origin_offset(&self, device_delta: Point) {
        let delta = self.device_to_user_delta(device_delta);
        self.origin_offset.set(self.origin_offset.get() + delta);
    }

    pub fn add_item(&self, coloured_item: ColouredShape) {
        self.hue_wheel.borrow_mut().add_item(coloured_item);
        self.drawing_area.queue_draw();
    }

    pub fn remove_item(&self, id: &str) {
        self.hue_wheel.borrow_mut().remove_item(id);
        self.drawing_area.queue_draw();
    }

    pub fn remove_all(&self) {
        self.hue_wheel.borrow_mut().remove_all();
        self.drawing_area.queue_draw();
    }

    pub fn set_target_colour(&self, colour: Option<&impl ColourBasics>) {
        self.hue_wheel.borrow_mut().set_target_colour(colour);
    }

    pub fn update_popup_condns(&self, changed_condns: MaskedCondns) {
        self.popup_menu.update_condns(changed_condns)
    }

    pub fn connect_popup_menu_item<F: Fn(&str) + 'static>(&self, name: &str, callback: F) {
        self.callbacks
            .borrow_mut()
            .get_mut(name)
            .expect("invalid name")
            .push(Box::new(callback));
    }

    fn menu_item_selected(&self, name: &str) {
        if let Some(ref item) = *self.chosen_item.borrow() {
            for callback in self
                .callbacks
                .borrow()
                .get(name)
                .expect("invalid name")
                .iter()
            {
                callback(item)
            }
        }
    }
}

#[derive(Default)]
pub struct GtkHueWheelBuilder {
    menu_item_specs: Vec<(&'static str, MenuItemSpec, u64)>,
    attributes: Vec<ScalarAttribute>,
}

impl GtkHueWheelBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn menu_item_specs(
        &mut self,
        menu_item_specs: &[(&'static str, MenuItemSpec, u64)],
    ) -> &mut Self {
        self.menu_item_specs = menu_item_specs.to_vec();
        self
    }

    pub fn attributes(&mut self, attributes: &[ScalarAttribute]) -> &mut Self {
        self.attributes.extend(attributes.iter());
        self
    }

    pub fn build(&self) -> Rc<GtkHueWheel> {
        let default_attributes = vec![ScalarAttribute::Value];
        let attributes = if self.attributes.is_empty() {
            &default_attributes
        } else {
            &self.attributes
        };

        let attribute_selector = AttributeSelectorBuilder::new()
            .orientation(gtk::Orientation::Horizontal)
            .attributes(attributes)
            .build();

        let drawing_area = gtk::DrawingAreaBuilder::new()
            .height_request(200)
            .width_request(200)
            .has_tooltip(true)
            .events(
                gdk::EventMask::SCROLL_MASK
                    | gdk::EventMask::BUTTON_PRESS_MASK
                    | gdk::EventMask::BUTTON_MOTION_MASK
                    | gdk::EventMask::LEAVE_NOTIFY_MASK
                    | gdk::EventMask::BUTTON_RELEASE_MASK,
            )
            .build();

        let popup_menu = ManagedMenuBuilder::new().build();

        let gtk_hue_wheel = Rc::new(GtkHueWheel {
            vbox: gtk::Box::new(gtk::Orientation::Vertical, 0),
            drawing_area,
            hue_wheel: RefCell::new(HueWheel::new()),
            chosen_item: RefCell::new(None),
            attribute_selector,
            popup_menu,
            callbacks: RefCell::new(HashMap::new()),
            origin_offset: Cell::new(Point::default()),
            last_xy: Cell::new(None),
        });

        for (name, menu_item_spec, condns) in self.menu_item_specs.iter() {
            let gtk_hue_wheel_c = Rc::clone(&gtk_hue_wheel);
            let name_c = (*name).to_string();
            gtk_hue_wheel
                .popup_menu
                .append_item(*name, &menu_item_spec, *condns)
                .connect_activate(move |_| gtk_hue_wheel_c.menu_item_selected(&name_c));
            gtk_hue_wheel
                .callbacks
                .borrow_mut()
                .insert((*name).to_string(), vec![]);
        }

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        hbox.pack_start(&gtk::Label::new(Some("Attribute: ")), false, false, 0);
        hbox.pack_start(&gtk_hue_wheel.attribute_selector.pwo(), true, true, 0);

        gtk_hue_wheel.vbox.pack_start(&hbox, false, false, 0);
        gtk_hue_wheel
            .vbox
            .pack_start(&gtk_hue_wheel.drawing_area, true, true, 0);

        let gtk_hue_wheel_c = Rc::clone(&gtk_hue_wheel);
        gtk_hue_wheel
            .attribute_selector
            .connect_changed(move |_| gtk_hue_wheel_c.drawing_area.queue_draw());

        let gtk_hue_wheel_c = Rc::clone(&gtk_hue_wheel);
        gtk_hue_wheel
            .drawing_area
            .connect_draw(move |da, cairo_context| {
                cairo_context.transform(gtk_hue_wheel_c.current_transform_matrix());
                let size = Size {
                    width: da.get_allocated_width() as f64,
                    height: da.get_allocated_height() as f64,
                };
                let cartesian = Drawer::new(cairo_context, size);
                gtk_hue_wheel_c
                    .hue_wheel
                    .borrow()
                    .draw(gtk_hue_wheel_c.attribute_selector.attribute(), &cartesian);
                gtk::Inhibit(false)
            });

        // ZOOM
        let gtk_hue_wheel_c = Rc::clone(&gtk_hue_wheel);
        gtk_hue_wheel
            .drawing_area
            .connect_scroll_event(move |da, scroll_event| {
                if let Some(device) = scroll_event.get_device() {
                    if device.get_source() == gdk::InputSource::Mouse {
                        match scroll_event.get_direction() {
                            gdk::ScrollDirection::Up => {
                                gtk_hue_wheel_c.hue_wheel.borrow_mut().decr_zoom();
                                da.queue_draw();
                                return gtk::Inhibit(true);
                            }
                            gdk::ScrollDirection::Down => {
                                gtk_hue_wheel_c.hue_wheel.borrow_mut().incr_zoom();
                                da.queue_draw();
                                return gtk::Inhibit(true);
                            }
                            _ => (),
                        }
                    }
                };
                gtk::Inhibit(false)
            });

        // COMMENCE MOVE ORIGIN OR POPUP MENU
        let gtk_hue_wheel_c = Rc::clone(&gtk_hue_wheel);
        gtk_hue_wheel
            .drawing_area
            .connect_button_press_event(move |_, event| {
                if event.get_event_type() != gdk::EventType::ButtonPress {
                    return gtk::Inhibit(false);
                };
                match event.get_button() {
                    1 => {
                        gtk_hue_wheel_c
                            .last_xy
                            .set(Some(event.get_position().into()));
                        gtk::Inhibit(true)
                    }
                    3 => {
                        let device_point: Point = event.get_position().into();
                        if let Some(item) = gtk_hue_wheel_c.hue_wheel.borrow().item_at_point(
                            gtk_hue_wheel_c
                                .device_to_user(device_point.x, device_point.y)
                                .into(),
                            gtk_hue_wheel_c.attribute_selector.attribute(),
                        ) {
                            *gtk_hue_wheel_c.chosen_item.borrow_mut() = Some(item.id().to_string());
                            gtk_hue_wheel_c.popup_menu.update_hover_condns(true);
                        } else {
                            *gtk_hue_wheel_c.chosen_item.borrow_mut() = None;
                            gtk_hue_wheel_c.popup_menu.update_hover_condns(false);
                        };
                        gtk_hue_wheel_c.popup_menu.popup_at_event(event);
                        gtk::Inhibit(true)
                    }
                    _ => gtk::Inhibit(false),
                }
            });

        // MOVE ORIGIN
        let gtk_hue_wheel_c = Rc::clone(&gtk_hue_wheel);
        gtk_hue_wheel
            .drawing_area
            .connect_motion_notify_event(move |da, event| {
                if let Some(last_xy) = gtk_hue_wheel_c.last_xy.get() {
                    let this_xy: Point = event.get_position().into();
                    let delta_xy = this_xy - last_xy;
                    gtk_hue_wheel_c.last_xy.set(Some(this_xy));
                    gtk_hue_wheel_c.shift_origin_offset(delta_xy);
                    da.queue_draw();
                    gtk::Inhibit(true)
                } else {
                    gtk::Inhibit(false)
                }
            });
        let gtk_hue_wheel_c = Rc::clone(&gtk_hue_wheel);
        gtk_hue_wheel
            .drawing_area
            .connect_button_release_event(move |_, event| {
                debug_assert_eq!(event.get_event_type(), gdk::EventType::ButtonRelease);
                if event.get_button() == 1 {
                    gtk_hue_wheel_c.last_xy.set(None);
                    gtk::Inhibit(true)
                } else {
                    gtk::Inhibit(false)
                }
            });
        let gtk_hue_wheel_c = Rc::clone(&gtk_hue_wheel);
        gtk_hue_wheel
            .drawing_area
            .connect_leave_notify_event(move |_, _| {
                gtk_hue_wheel_c.last_xy.set(None);
                gtk::Inhibit(false)
            });

        // TOOLTIP
        let gtk_hue_wheel_c = Rc::clone(&gtk_hue_wheel);
        gtk_hue_wheel
            .drawing_area
            .connect_query_tooltip(move |_, x, y, _, tooltip| {
                let point = gtk_hue_wheel_c.device_to_user(x as f64, y as f64);
                if let Some(text) = gtk_hue_wheel_c
                    .hue_wheel
                    .borrow()
                    .tooltip_for_point(point.into(), gtk_hue_wheel_c.attribute_selector.attribute())
                {
                    tooltip.set_text(Some(&text));
                    true
                } else {
                    false
                }
            });

        gtk_hue_wheel
    }
}
