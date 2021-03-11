// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::cmp::Ordering;

use crate::{
    beigui::{DrawShapes, Point},
    fdrn::{FDRNumber, UFDRNumber},
    hue::HueIfce,
    ColourAttributes, ColourBasics, Hue, HueConstants, LightLevel, RGBConstants, ScalarAttribute,
    Value, HCV, RGB,
};

#[derive(Debug)]
pub struct Zoom {
    scale: UFDRNumber,
}

impl Zoom {
    const DELTA: UFDRNumber = UFDRNumber(UFDRNumber::ONE.0 * 25 / 1000);
    const MAX: UFDRNumber = UFDRNumber(UFDRNumber::ONE.0 * 10);

    pub fn decr(&mut self) {
        self.scale = (self.scale - Self::DELTA).max(UFDRNumber::ONE);
    }

    pub fn incr(&mut self) {
        self.scale = (self.scale + Self::DELTA).min(Self::MAX);
    }

    pub fn scale(&self) -> UFDRNumber {
        self.scale
    }
}

impl Default for Zoom {
    fn default() -> Self {
        Self {
            scale: UFDRNumber::ONE,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Shape {
    Circle,
    Diamond,
    Square,
    BackSight,
}

#[derive(Debug, PartialEq, Eq, Ord, Clone, Copy)]
pub enum Proximity {
    Enclosed(UFDRNumber),
    NotEnclosed(UFDRNumber),
}

impl std::cmp::PartialOrd for Proximity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self {
            Self::Enclosed(mine) => match other {
                Self::Enclosed(other) => mine.partial_cmp(other),
                Self::NotEnclosed(_) => Some(std::cmp::Ordering::Less),
            },
            Self::NotEnclosed(mine) => match other {
                Self::Enclosed(_) => Some(std::cmp::Ordering::Greater),
                Self::NotEnclosed(other) => mine.partial_cmp(other),
            },
        }
    }
}

pub trait ShapeConsts {
    const SIN_45: Self;
    const SHAPE_SIDE: Self;
    const SHAPE_HALF_SIDE: Self;
    const SHAPE_RADIUS: Self;
}

impl ShapeConsts for UFDRNumber {
    const SIN_45: Self = Self(Self::SQRT_2.0 / 2);
    const SHAPE_SIDE: Self = Self(Self::ONE.0 * 6 / 100);
    const SHAPE_HALF_SIDE: Self = Self(Self::SHAPE_SIDE.0 / 2);
    const SHAPE_RADIUS: Self = Self::SHAPE_HALF_SIDE;
}

impl ShapeConsts for FDRNumber {
    const SIN_45: Self = Self(Self::SQRT_2.0 / 2);
    const SHAPE_SIDE: Self = Self(Self::ONE.0 * 6 / 100);
    const SHAPE_HALF_SIDE: Self = Self(Self::SHAPE_SIDE.0 / 2);
    const SHAPE_RADIUS: Self = Self::SHAPE_HALF_SIDE;
}

#[derive(Debug, Clone, Copy)]
enum CachedPoint {
    Hued(Point),
    Grey(Point),
}

#[derive(Debug)]
pub struct ColouredShape {
    id: String,
    colour: HCV,
    cached_point: CachedPoint,
    tooltip_text: String,
    shape: Shape,
}

impl ColouredShape {
    pub fn new(colour: &impl ColourBasics, id: &str, tooltip_text: &str, shape: Shape) -> Self {
        let cached_point = if let Some(hue_angle) = colour.hue_angle() {
            CachedPoint::Hued(Point::from((hue_angle, UFDRNumber::ONE)))
        } else {
            CachedPoint::Grey(Point {
                x: FDRNumber::from(-1.05),
                y: FDRNumber::ONE - FDRNumber::from(colour.value()) * 2,
            })
        };
        Self {
            id: id.to_string(),
            colour: colour.hcv(),
            cached_point,
            tooltip_text: tooltip_text.to_string(),
            shape,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    fn xy(&self, scalar_attribute: ScalarAttribute, zoom: &Zoom) -> Point {
        match self.cached_point {
            CachedPoint::Hued(point) => {
                point * self.colour.scalar_attribute(scalar_attribute).into() * zoom.scale()
            }
            CachedPoint::Grey(point) => point * zoom.scale(),
        }
    }

    pub fn draw_shape(
        &self,
        scalar_attribute: ScalarAttribute,
        zoom: &Zoom,
        draw_shapes: &impl DrawShapes,
    ) {
        draw_shapes.set_fill_colour(&self.colour);
        draw_shapes.set_line_colour(&self.colour.best_foreground());
        draw_shapes.set_line_width(UFDRNumber::from(0.01));
        let xy = self.xy(scalar_attribute, zoom);
        match self.shape {
            Shape::Circle => {
                draw_shapes.draw_circle(xy, UFDRNumber::SHAPE_RADIUS, true);
                draw_shapes.draw_circle(xy, UFDRNumber::SHAPE_RADIUS, false);
            }
            Shape::Diamond => {
                draw_shapes.draw_diamond(xy, UFDRNumber::SHAPE_SIDE, true);
                draw_shapes.draw_diamond(xy, UFDRNumber::SHAPE_SIDE, false);
            }
            Shape::Square => {
                draw_shapes.draw_square(xy, UFDRNumber::SHAPE_SIDE, true);
                draw_shapes.draw_square(xy, UFDRNumber::SHAPE_SIDE, false);
            }
            Shape::BackSight => {
                draw_shapes.draw_circle(xy, UFDRNumber::SHAPE_RADIUS, true);
                draw_shapes.draw_circle(xy, UFDRNumber::SHAPE_RADIUS, false);
                draw_shapes.draw_plus_sign(xy, UFDRNumber::SHAPE_SIDE);
            }
        }
    }

    fn proximity_to(
        &self,
        point: Point,
        scalar_attribute: ScalarAttribute,
        zoom: &Zoom,
    ) -> Proximity {
        let delta = self.xy(scalar_attribute, zoom) - point;
        let distance = delta.hypot();
        match self.shape {
            Shape::Circle | Shape::BackSight => {
                if distance < UFDRNumber::SHAPE_RADIUS {
                    Proximity::Enclosed(distance)
                } else {
                    Proximity::NotEnclosed(distance)
                }
            }
            Shape::Square => {
                let x = delta.x.abs();
                let y = delta.y.abs();
                if x < FDRNumber::SHAPE_HALF_SIDE && y < FDRNumber::SHAPE_HALF_SIDE {
                    Proximity::Enclosed(distance)
                } else {
                    Proximity::NotEnclosed(distance)
                }
            }
            Shape::Diamond => {
                // Rotate 45 degrees
                let x = ((delta.x - delta.y) * FDRNumber::SIN_45).abs();
                let y = ((delta.x + delta.y) * FDRNumber::SIN_45).abs();
                if x < FDRNumber::SHAPE_HALF_SIDE && y < FDRNumber::SHAPE_HALF_SIDE {
                    Proximity::Enclosed(distance)
                } else {
                    Proximity::NotEnclosed(distance)
                }
            }
        }
    }
}

pub trait MakeColouredShape {
    fn coloured_shape(&self) -> ColouredShape;
}

impl<L: LightLevel> From<&RGB<L>> for ColouredShape {
    fn from(rgb: &RGB<L>) -> Self {
        let id = format!("ID: {}", rgb.pango_string());
        let tooltip_text = format!("RGB: {}", id);
        ColouredShape::new(rgb, &id, &tooltip_text, Shape::Circle)
    }
}

impl Ord for ColouredShape {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for ColouredShape {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ColouredShape {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ColouredShape {}

pub trait Graticule {
    fn draw_rings(num_rings: u8, zoom: &Zoom, draw_shapes: &impl DrawShapes) {
        draw_shapes.set_line_width(UFDRNumber::from(0.01));
        draw_shapes.set_line_colour(&HCV::WHITE); // * UFDRNumber::from(0.25));
        let centre = Point::default();
        for num in 1..=num_rings {
            let radius: UFDRNumber = UFDRNumber::ONE * num as i32 / num_rings as i32;
            draw_shapes.draw_circle(centre, radius * zoom.scale(), false);
        }
    }

    fn draw_spokes(start_ring: UFDRNumber, zoom: &Zoom, draw_shapes: &impl DrawShapes) {
        draw_shapes.set_line_width(UFDRNumber::from(0.015));
        for hue in Hue::PRIMARIES
            .iter()
            .chain(Hue::SECONDARIES.iter())
            .chain(Hue::IN_BETWEENS.iter())
        {
            draw_shapes.set_line_colour(&hue.max_chroma_hcv());
            let angle = hue.angle();
            let start: Point = (angle, start_ring).into();
            let end: Point = (angle, UFDRNumber::ONE).into();
            draw_shapes.draw_line(&[start * zoom.scale(), end * zoom.scale()]);
        }
    }

    fn draw_graticule(&self, zoom: &Zoom, draw_shapes: &impl DrawShapes) {
        draw_shapes.set_background_colour(&HCV::new_grey(Value::ONE / 2));
        Self::draw_spokes(UFDRNumber::from(0.1), zoom, draw_shapes);
        Self::draw_rings(10, zoom, draw_shapes);
    }
}

pub struct HueWheel {
    shapes: Vec<ColouredShape>,
    target: Option<ColouredShape>,
    zoom: Zoom,
}

impl Default for HueWheel {
    fn default() -> Self {
        Self {
            shapes: vec![],
            target: None,
            zoom: Zoom::default(),
        }
    }
}

impl Graticule for HueWheel {}

impl HueWheel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn decr_zoom(&mut self) {
        self.zoom.decr();
    }

    pub fn incr_zoom(&mut self) {
        self.zoom.incr();
    }

    pub fn draw(&self, scalar_attribute: ScalarAttribute, draw_shapes: &impl DrawShapes) {
        self.draw_graticule(&self.zoom, draw_shapes);
        for shape in self.shapes.iter() {
            shape.draw_shape(scalar_attribute, &self.zoom, draw_shapes);
        }
        if let Some(ref target) = self.target {
            target.draw_shape(scalar_attribute, &self.zoom, draw_shapes)
        }
    }

    fn nearest_to(
        &self,
        point: Point,
        scalar_attribute: ScalarAttribute,
    ) -> Option<(&ColouredShape, Proximity)> {
        let mut nearest: Option<(&ColouredShape, Proximity)> = None;
        for shape in self.shapes.iter() {
            let proximity = shape.proximity_to(point, scalar_attribute, &self.zoom);
            if let Some((_, nearest_so_far)) = nearest {
                if proximity < nearest_so_far {
                    nearest = Some((shape, proximity));
                }
            } else {
                nearest = Some((shape, proximity));
            }
        }
        nearest
    }

    pub fn item_at_point(
        &self,
        point: Point,
        scalar_attribute: ScalarAttribute,
    ) -> Option<&ColouredShape> {
        if let Some((shape, proximity)) = self.nearest_to(point, scalar_attribute) {
            if let Proximity::Enclosed(_) = proximity {
                return Some(shape);
            }
        };
        None
    }

    pub fn tooltip_for_point(
        &self,
        point: Point,
        scalar_attribute: ScalarAttribute,
    ) -> Option<String> {
        if let Some((shape, _)) = self.nearest_to(point, scalar_attribute) {
            return Some(shape.tooltip_text.to_string());
        }
        None
    }

    pub fn add_item(&mut self, coloured_item: ColouredShape) -> Option<ColouredShape> {
        //self.shapes.push(coloured_item);
        let id = coloured_item.id();
        match self.shapes.binary_search_by_key(&id, |s| s.id()) {
            Ok(index) => {
                self.shapes.push(coloured_item);
                let old = self.shapes.swap_remove(index);
                Some(old)
            }
            Err(index) => {
                self.shapes.insert(index, coloured_item);
                None
            }
        }
    }

    pub fn remove_item(&mut self, id: &str) -> ColouredShape {
        match self.shapes.binary_search_by_key(&id, |s| s.id()) {
            Ok(index) => self.shapes.remove(index),
            Err(_) => unreachable!("{}: shape with this id not found", id),
        }
    }

    pub fn remove_all(&mut self) {
        self.shapes.clear();
    }

    pub fn set_target_colour(&mut self, colour: Option<&impl ColourBasics>) {
        if let Some(colour) = colour {
            let target =
                ColouredShape::new(colour, "###target###", "Target Colour", Shape::BackSight);
            self.target = Some(target);
        } else {
            self.target = None;
        }
    }
}
