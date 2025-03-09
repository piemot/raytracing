use std::rc::Rc;

use crate::{Color, Point3};

pub trait Texture: std::fmt::Debug {
    fn value(&self, u: f64, v: f64, point: &Point3) -> Color;
    fn into_texture(self) -> Rc<dyn Texture>
    where
        Self: Sized + 'static,
    {
        Rc::new(self)
    }
}

#[derive(Debug)]
pub struct SolidColor(Color);

impl SolidColor {
    pub fn new(albedo: Color) -> Self {
        Self(albedo)
    }
}

impl Texture for SolidColor {
    fn value(&self, _u: f64, _v: f64, _point: &Point3) -> Color {
        self.0
    }
}

impl From<Color> for SolidColor {
    fn from(value: Color) -> Self {
        Self::new(value)
    }
}

#[derive(Debug)]
pub struct Checkerboard {
    scale: f64,
    even: Rc<dyn Texture>,
    odd: Rc<dyn Texture>,
}

impl Checkerboard {
    pub fn new(scale: f64, even: Rc<dyn Texture>, odd: Rc<dyn Texture>) -> Self {
        Self { scale, even, odd }
    }

    pub fn solid(scale: f64, even: SolidColor, odd: SolidColor) -> Self {
        Self {
            scale,
            even: Rc::new(even),
            odd: Rc::new(odd),
        }
    }
}

impl Texture for Checkerboard {
    fn value(&self, u: f64, v: f64, point: &Point3) -> Color {
        let x = f64::floor((1.0 / self.scale) * point.x());
        let y = f64::floor((1.0 / self.scale) * point.y());
        let z = f64::floor((1.0 / self.scale) * point.z());

        let is_even = (x as i32 + y as i32 + z as i32) % 2 == 0;

        match is_even {
            true => self.even.value(u, v, point),
            false => self.odd.value(u, v, point),
        }
    }
}
