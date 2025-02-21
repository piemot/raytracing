pub mod camera;
pub mod hittable;
pub mod material;
pub mod math;

pub use camera::Camera;

pub use material::Material;

pub use math::point;
pub use math::point::Point2;
pub use math::point::Point3;

pub use math::vec;
pub use math::vec::two_d;
pub use math::vec::two_d::Vec2;
pub use math::vec::Vec3;

pub use math::interval;
pub use math::interval::Interval;

pub use math::color;
pub use math::color::Color;

pub use math::ray;
pub use math::ray::Ray3;

pub use hittable::{HitRecord, Hittable};
