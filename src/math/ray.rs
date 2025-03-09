use crate::{Point3, Vec3};

#[derive(Debug, PartialEq, Clone, Copy)]
/// Represents a 3-dimensional ray, starting at an origin and moving across a vector.
pub struct Ray3 {
    origin: Point3,
    direction: Vec3,
}

impl Ray3 {
    pub fn new(origin: Point3, direction: Vec3) -> Self {
        Self { origin, direction }
    }

    pub fn at(&self, dist: f64) -> Point3 {
        self.origin + self.direction * dist
    }

    pub fn origin(&self) -> Point3 {
        self.origin
    }

    pub fn direction(&self) -> Vec3 {
        self.direction
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
/// Represents a 4-dimensional ray, starting at an origin and moving across a vector at a given time.
pub struct Ray4 {
    origin: Point3,
    direction: Vec3,
    time: f64,
}

impl Ray4 {
    pub fn new(origin: Point3, direction: Vec3, time: f64) -> Self {
        Self {
            origin,
            direction,
            time,
        }
    }

    pub fn at(&self, dist: f64) -> Point3 {
        self.origin + self.direction * dist
    }

    pub fn origin(&self) -> Point3 {
        self.origin
    }

    pub fn direction(&self) -> Vec3 {
        self.direction
    }

    pub fn time(&self) -> f64 {
        self.time
    }

    pub fn ignore_time(&self) -> Ray3 {
        Ray3::new(self.origin(), self.direction())
    }
}
