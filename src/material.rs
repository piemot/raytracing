use crate::{Color, HitRecord, Ray3, Vec3};

pub struct MaterialResult {
    pub attenuation: Color,
    pub scattered: Ray3,
}

pub trait Material: std::fmt::Debug {
    fn scatter(&self, ray_in: &Ray3, record: &HitRecord) -> Option<MaterialResult>;
}

#[derive(Debug)]
pub struct Lambertian {
    albedo: Color,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    // Lambertian materials are independant of the incoming ray due to Lambert's Cosine Law.
    fn scatter(&self, _ray_in: &Ray3, record: &HitRecord) -> Option<MaterialResult> {
        let mut scatter_direction = record.normal() + Vec3::random_unit_vector();

        // Catch random_unit_vector being opposite record.normal()
        if scatter_direction.near_zero() {
            scatter_direction = record.normal().into();
        }

        let scattered = Ray3::new(record.point(), scatter_direction);

        return Some(MaterialResult {
            attenuation: self.albedo,
            scattered,
        });
    }
}

#[derive(Debug)]
pub struct Metal {
    albedo: Color,
}

impl Metal {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

impl Material for Metal {
    fn scatter(&self, ray_in: &Ray3, record: &HitRecord) -> Option<MaterialResult> {
        let reflected = Vec3::reflect(&ray_in.direction(), &record.normal());
        let scattered = Ray3::new(record.point(), reflected);
        Some(MaterialResult {
            attenuation: self.albedo,
            scattered,
        })
    }
}
