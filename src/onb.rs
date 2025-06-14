use crate::{vec::Normalized, Vec3};

#[derive(Debug)]
pub struct OrthonormalBasis {
    u: Vec3<Normalized>,
    v: Vec3<Normalized>,
    w: Vec3<Normalized>,
}

impl OrthonormalBasis {
    /// Construct a new [`OrthonormalBasis`] utility class from a reference
    /// vector (usually the surface normal)
    pub fn new(vec: &Vec3) -> Self {
        let w = vec.as_unit();

        let a = if w.x().abs() > 0.9 {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        }
        .as_unit();

        let v: Vec3 = Vec3::<Normalized>::cross(&w, &a).into();
        let v = v.as_unit();
        let u = Vec3::<Normalized>::cross(&w, &v);

        Self { u, v, w }
    }

    pub fn u(&self) -> Vec3<Normalized> {
        self.u
    }

    pub fn v(&self) -> Vec3<Normalized> {
        self.v
    }

    pub fn w(&self) -> Vec3<Normalized> {
        self.w
    }

    /// Transform a vector from basis coordinates to local space.
    pub fn transform(&self, vec: &Vec3) -> Vec3 {
        (vec.x() * self.u()) + (vec.y() * self.v()) + (vec.z() * self.w())
    }
}
