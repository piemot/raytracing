use crate::{Hittable, OrthonormalBasis, Point3, Vec3};
use std::{f64::consts::PI, rc::Rc};

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
pub struct HittablePDF {
    objects: Rc<dyn Hittable>,
    origin: Point3,
}

impl HittablePDF {
    pub fn new(objects: Rc<dyn Hittable>, origin: &Point3) -> Self {
        Self {
            objects,
            origin: *origin,
        }
    }
}

impl PDF for HittablePDF {
    fn value(&self, direction: &Vec3) -> f64 {
        self.objects.pdf_value(&self.origin, direction)
    }

    fn generate(&self) -> Vec3 {
        self.objects.random(&self.origin)
    }
}

pub struct MixedPDF {
    factors: Vec<(Rc<dyn PDF>, f64)>,
}

impl MixedPDF {
    pub fn new(factors: Vec<(Rc<dyn PDF>, f64)>) -> Self {
        let factor_sum: f64 = factors.iter().map(|f| f.1).sum();
        assert!(factor_sum == 1.0);

        Self { factors }
    }

    pub fn equal(factors: Vec<Rc<dyn PDF>>) -> Self {
        debug_assert!(factors.len() < u32::MAX.try_into().unwrap());

        let mul = 1.0 / f64::from(factors.len() as u32);
        Self {
            factors: factors.into_iter().map(|factor| (factor, mul)).collect(),
        }
    }
}

impl PDF for MixedPDF {
    fn value(&self, direction: &Vec3) -> f64 {
        self.factors
            .iter()
            .map(|(pdf, weight)| weight * pdf.value(direction))
            .sum()
    }

    fn generate(&self) -> Vec3 {
        let rand: f64 = rand::random();
        let mut sum = 0.0;
        for (pdf, weight) in &self.factors {
            sum += weight;
            if sum < rand {
                return pdf.generate();
            }
        }
        unreachable!();
    }
}
