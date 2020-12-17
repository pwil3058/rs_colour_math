// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::cmp::Ordering;

use normalised_angles::Degrees;
use num_traits_plus::float_plus::FloatPlus;

use crate::{ColourComponent, ColourInterface, HueConstants, RGBConstants, ScalarAttribute, RGB};

use super::drawing::{Cartesian, Point};

pub struct Zoom<F: FloatPlus> {
    scale: F,
}

impl<F: FloatPlus> Zoom<F> {
    pub fn decr(&mut self) {
        let new_scale = (self.scale - F::from(0.025).unwrap()).max(F::ONE);
        self.scale = new_scale;
    }

    pub fn incr(&mut self) {
        let new_scale = (self.scale + F::from(0.025).unwrap()).min(F::from(10.0).unwrap());
        self.scale = new_scale;
    }

    pub fn scale(&self) -> F {
        self.scale
    }
}

impl<F: FloatPlus> Default for Zoom<F> {
    fn default() -> Self {
        Self { scale: F::ONE }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Shape {
    Circle,
    Diamond,
    Square,
    BackSight,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Proximity<F: ColourComponent + PartialOrd> {
    Enclosed(F),
    NotEnclosed(F),
}

impl<F: ColourComponent> std::cmp::PartialOrd for Proximity<F> {
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

pub trait ShapeConsts: ColourComponent {
    const SIN_45: Self;
    const SHAPE_SIDE: Self;
    const SHAPE_HALF_SIDE: Self;
    const SHAPE_RADIUS: Self;
}

#[derive(Debug, Clone, Copy)]
enum CachedPoint<F: ColourComponent + ShapeConsts> {
    Hued(Point<F>),
    Grey(Point<F>),
}

#[derive(Debug)]
pub struct ColouredShape<F: ColourComponent + ShapeConsts> {
    id: String,
    rgb: RGB<F>,
    cached_point: CachedPoint<F>,
    tooltip_text: String,
    shape: Shape,
}

impl<F: ColourComponent + ShapeConsts> ColouredShape<F> {
    pub fn new(rgb: RGB<F>, id: &str, tooltip_text: &str, shape: Shape) -> Self {
        let cached_point = if let Some(hue_angle) = rgb.hue_angle() {
            CachedPoint::Hued(Point::from((hue_angle, F::ONE)))
        } else {
            CachedPoint::Grey(Point {
                x: F::from(-1.05).unwrap(),
                y: F::ONE - F::TWO * rgb.value(),
            })
        };
        Self {
            id: id.to_string(),
            rgb,
            cached_point,
            tooltip_text: tooltip_text.to_string(),
            shape,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    fn xy(&self, scalar_attribute: ScalarAttribute, zoom: &Zoom<F>) -> Point<F> {
        match self.cached_point {
            CachedPoint::Hued(point) => {
                point * self.rgb.scalar_attribute(scalar_attribute) * zoom.scale()
            }
            CachedPoint::Grey(point) => point * zoom.scale(),
        }
    }

    pub fn draw_shape(
        &self,
        scalar_attribute: ScalarAttribute,
        zoom: &Zoom<F>,
        cartesian: &impl Cartesian<F>,
    ) {
        cartesian.set_fill_colour(self.rgb);
        cartesian.set_line_colour(self.rgb.best_foreground_rgb());
        cartesian.set_line_width(F::from(0.01).unwrap());
        let xy = self.xy(scalar_attribute, zoom);
        match self.shape {
            Shape::Circle => {
                cartesian.draw_circle(xy, F::SHAPE_RADIUS, true);
                cartesian.draw_circle(xy, F::SHAPE_RADIUS, false);
            }
            Shape::Diamond => {
                cartesian.draw_diamond(xy, F::SHAPE_SIDE, true);
                cartesian.draw_diamond(xy, F::SHAPE_SIDE, false);
            }
            Shape::Square => {
                cartesian.draw_square(xy, F::SHAPE_SIDE, true);
                cartesian.draw_square(xy, F::SHAPE_SIDE, false);
            }
            Shape::BackSight => {
                cartesian.draw_circle(xy, F::SHAPE_RADIUS, true);
                cartesian.draw_circle(xy, F::SHAPE_RADIUS, false);
                cartesian.draw_plus_sign(xy, F::SHAPE_SIDE);
            }
        }
    }

    fn proximity_to(
        &self,
        point: Point<F>,
        scalar_attribute: ScalarAttribute,
        zoom: &Zoom<F>,
    ) -> Proximity<F> {
       let delta = self.xy(scalar_attribute, zoom) - point;
        let distance = delta.hypot();
        match self.shape {
            Shape::Circle | Shape::BackSight => {
                if distance < F::SHAPE_RADIUS {
                    Proximity::Enclosed(distance)
                } else {
                    Proximity::NotEnclosed(distance)
                }
            }
            Shape::Square => {
                let x = delta.x.abs();
                let y = delta.y.abs();
                if x < F::SHAPE_HALF_SIDE && y < F::SHAPE_HALF_SIDE {
                    Proximity::Enclosed(distance)
                } else {
                    Proximity::NotEnclosed(distance)
                }
            }
            Shape::Diamond => {
                // Rotate 45 degrees
                let x = ((delta.x - delta.y) * F::SIN_45).abs();
                let y = ((delta.x + delta.y) * F::SIN_45).abs();
                if x < F::SHAPE_HALF_SIDE && y < F::SHAPE_HALF_SIDE {
                    Proximity::Enclosed(distance)
                } else {
                    Proximity::NotEnclosed(distance)
                }
            }
        }
    }
}

pub trait MakeColouredShape<F: ColourComponent + ShapeConsts> {
    fn coloured_shape(&self) -> ColouredShape<F>;
}

impl<F: ColourComponent + ShapeConsts> From<&RGB<F>> for ColouredShape<F> {
    fn from(rgb: &RGB<F>) -> Self {
        let id = format!("ID: {}", rgb.pango_string());
        let tooltip_text = format!("RGB: {}", id);
        ColouredShape::new(*rgb, &id, &tooltip_text, Shape::Circle)
    }
}

impl<F: ColourComponent + ShapeConsts> Ord for ColouredShape<F> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl<F: ColourComponent + ShapeConsts> PartialOrd for ColouredShape<F> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<F: ColourComponent + ShapeConsts> PartialEq for ColouredShape<F> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<F: ColourComponent + ShapeConsts> Eq for ColouredShape<F> {}

pub trait Graticule<F: ColourComponent + ShapeConsts> {
    fn draw_rings(num_rings: u32, zoom: &Zoom<F>, cartesian: &impl Cartesian<F>) {
        cartesian.set_line_width(F::from(0.01).unwrap());
        cartesian.set_line_colour(RGB::WHITE); // * F::from(0.25).unwrap());
        let divisor = F::from_u32(num_rings).unwrap();
        let centre = Point::<F>::default();
        for num in 1..=num_rings {
            let radius: F = F::from(num).unwrap() / divisor;
            cartesian.draw_circle(centre, radius * zoom.scale(), false);
        }
    }

    fn draw_spokes(start_ring: F, zoom: &Zoom<F>, cartesian: &impl Cartesian<F>) {
        cartesian.set_line_width(F::from(0.015).unwrap());
        let mut hue = RGB::<F>::RED.hue().unwrap();
        for _ in 0..13 {
            cartesian.set_line_colour(hue.max_chroma_rgb());
            let angle = hue.angle();
            let start: Point<F> = (angle, start_ring).into();
            let end: Point<F> = (angle, F::ONE).into();
            cartesian.draw_line(&[start * zoom.scale(), end * zoom.scale()]);
            hue = hue + Degrees::DEG_30;
        }
    }

    fn draw_graticule(&self, zoom: &Zoom<F>, cartesian: &impl Cartesian<F>) {
        cartesian.set_background_colour(RGB::WHITE * F::HALF);
        Self::draw_spokes(F::from(0.1).unwrap(), zoom, cartesian);
        Self::draw_rings(10, zoom, cartesian);
    }
}

pub struct HueWheel<F: ColourComponent + ShapeConsts> {
    shapes: Vec<ColouredShape<F>>,
    target: Option<ColouredShape<F>>,
    zoom: Zoom<F>,
}

impl<F: ColourComponent + ShapeConsts> Default for HueWheel<F> {
    fn default() -> Self {
        Self {
            shapes: vec![],
            target: None,
            zoom: Zoom::<F>::default(),
        }
    }
}

impl<F: ColourComponent + ShapeConsts> Graticule<F> for HueWheel<F> {}

impl<F: ColourComponent + ShapeConsts> HueWheel<F> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn decr_zoom(&mut self) {
        self.zoom.decr();
    }

    pub fn incr_zoom(&mut self) {
        self.zoom.incr();
    }

    pub fn draw(&self, scalar_attribute: ScalarAttribute, cartesian: &impl Cartesian<F>) {
        self.draw_graticule(&self.zoom, cartesian);
        for shape in self.shapes.iter() {
            shape.draw_shape(scalar_attribute, &self.zoom, cartesian);
        }
        if let Some(ref target) = self.target {
            target.draw_shape(scalar_attribute, &self.zoom, cartesian)
        }
    }

    fn nearest_to(
        &self,
        point: Point<F>,
        scalar_attribute: ScalarAttribute,
    ) -> Option<(&ColouredShape<F>, Proximity<F>)> {
        let mut nearest: Option<(&ColouredShape<F>, Proximity<F>)> = None;
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
        point: Point<F>,
        scalar_attribute: ScalarAttribute,
    ) -> Option<&ColouredShape<F>> {
        if let Some((shape, proximity)) = self.nearest_to(point, scalar_attribute) {
            if let Proximity::Enclosed(_) = proximity {
                return Some(shape);
            }
        };
        None
    }

    pub fn tooltip_for_point(
        &self,
        point: Point<F>,
        scalar_attribute: ScalarAttribute,
    ) -> Option<String> {
        if let Some((shape, _)) = self.nearest_to(point, scalar_attribute) {
            return Some(shape.tooltip_text.to_string());
        }
        None
    }

    pub fn add_item(&mut self, coloured_item: ColouredShape<F>) -> Option<ColouredShape<F>> {
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

    pub fn remove_item(&mut self, id: &str) -> ColouredShape<F> {
        match self.shapes.binary_search_by_key(&id, |s| s.id()) {
            Ok(index) => self.shapes.remove(index),
            Err(_) => unreachable!("{}: shape with this id not found", id),
        }
    }

    pub fn remove_all(&mut self) {
        self.shapes.clear();
    }

    pub fn set_target_rgb(&mut self, rgb: Option<&RGB<F>>) {
        if let Some(rgb) = rgb {
            let target =
                ColouredShape::new(*rgb, "###target###", "Target Colour", Shape::BackSight);
            self.target = Some(target);
        } else {
            self.target = None;
        }
    }
}

impl ShapeConsts for f64 {
    const SIN_45: Self = f64::SQRT_2 / 2.0;
    const SHAPE_SIDE: Self = 0.06;
    const SHAPE_HALF_SIDE: Self = Self::SHAPE_SIDE / 2.0;
    const SHAPE_RADIUS: Self = Self::SHAPE_HALF_SIDE;
}

impl ShapeConsts for f32 {
    const SIN_45: Self = f32::SQRT_2 / 2.0;
    const SHAPE_SIDE: Self = 0.06;
    const SHAPE_HALF_SIDE: Self = Self::SHAPE_SIDE / 2.0;
    const SHAPE_RADIUS: Self = Self::SHAPE_HALF_SIDE;
}
