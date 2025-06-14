use crate::{OrthonormalBasis, Vec3};
use std::f64::consts::PI;

pub trait PDF {
    fn value(&self, direction: &Vec3) -> f64;
    fn generate(&self) -> Vec3;
}

pub struct SpherePDF;
impl PDF for SpherePDF {
    fn value(&self, _dir: &Vec3) -> f64 {
        1.0 / (4.0 * PI)
    }

    fn generate(&self) -> Vec3 {
        Vec3::random_in_unit_sphere()
    }
}
pub struct CosinePDF(OrthonormalBasis);

impl CosinePDF {
    pub fn new(w: &Vec3) -> Self {
        CosinePDF(OrthonormalBasis::new(w))
    }
}

impl PDF for CosinePDF {
    fn value(&self, direction: &Vec3) -> f64 {
        let cos_theta = Vec3::dot(&direction.as_unit(), &self.0.w());
        f64::max(0.0, cos_theta / PI)
    }

    fn generate(&self) -> Vec3 {
        self.0.transform(&Vec3::random_on_sphere_cosine())
    }
}
