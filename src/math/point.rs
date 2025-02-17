/* use crate::math::macros::forward_ref_binop;
use crate::Vec3;
use std::ops::{Add, Sub};

/// Represents a point in 3 dimensional space.
///
/// # Mathematical Functions
/// [`Point3`]s can be:
/// * added to a [`Vec3`] to create another [`Point3`],
/// * subtracted with a [`Vec3`] to create another [`Point3`] (by adding the negative of the vector),
/// * or subtracted with another [`Point3`] to create a [`Vec3`] (which __does not store its origin__).
///
/// [`Point3`]s **cannot** be multiplied, divided, negated, or manipulated in other ways.
/// They can be trivially converted back and forth into [`Vec3`]s with the From trait.
///
/// [`Vec3`]: crate::Vec3
/// [`Point3`]: crate::Point3
///
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Point3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Point3 {
    /// Create a new Point3.
    /// ```rs
    /// use crate::Point3;
    /// let pt = Point3::new(0.0, 1.0, 1.0);
    /// ```
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Point3 { x, y, z }
    }

    /// Create a new Point3 at the origin.
    pub const fn origin() -> Self {
        Point3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn z(&self) -> f64 {
        self.z
    }
}

// Note that Point3 + Point3 ≠ Point3,
// so Add<Point3> cannot be defined for Point3.

// Point3 + Vec3 = Point3
impl Add<Vec3> for Point3 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Vec3) -> Self {
        Self {
            x: self.x + rhs.x(),
            y: self.y + rhs.y(),
            z: self.z + rhs.z(),
        }
    }
}

forward_ref_binop! { impl Add, add for Point3, Vec3 }

// Point3 - Point3 = Vec3
impl Sub for Point3 {
    type Output = Vec3;

    #[inline]
    fn sub(self, rhs: Self) -> Vec3 {
        Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

forward_ref_binop! { impl Sub, sub for Point3, Point3 }

// Point3 - Vec3 = Point3
impl Sub<Vec3> for Point3 {
    type Output = Point3;

    #[inline]
    fn sub(self, rhs: Vec3) -> Point3 {
        self + (-rhs)
    }
}

forward_ref_binop! { impl Sub, sub for Point3, Vec3 }

// Vectors and points can be trivially substituted.
impl From<Point3> for Vec3 {
    fn from(value: Point3) -> Self {
        Vec3::new(value.x, value.y, value.z)
    }
}

impl From<Vec3> for Point3 {
    fn from(value: Vec3) -> Self {
        Point3::new(value.x(), value.y(), value.z())
    }
}
 */
