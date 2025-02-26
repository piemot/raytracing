use rand::random;

use crate::{Color, HitRecord, Ray3, Vec3};

#[derive(Debug)]
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
    fuzz: f64,
}

impl Metal {
    pub fn new(albedo: Color) -> Self {
        Self { albedo, fuzz: 0.0 }
    }

    pub fn with_fuzz(albedo: Color, fuzz: f64) -> Self {
        assert!(
            (0.0..=1.0).contains(&fuzz),
            "Invalid fuzz value (expected 0.0..=1.0)",
        );

        Self { albedo, fuzz }
    }
}

impl Material for Metal {
    fn scatter(&self, ray_in: &Ray3, record: &HitRecord) -> Option<MaterialResult> {
        let reflected = Vec3::reflect(&ray_in.direction(), &record.normal());
        let reflected = reflected.as_unit() + (self.fuzz * Vec3::random_in_unit_sphere());
        let scattered = Ray3::new(record.point(), reflected);

        if Vec3::dot(&reflected, &record.normal()) < 0.0 {
            // if the ray has been scattered below the surface of the object
            return None;
        }

        Some(MaterialResult {
            attenuation: self.albedo,
            scattered,
        })
    }
}

#[derive(Debug)]
pub struct Dielectric {
    /// Refractive index in vacuum or air, or the ratio of the material's refractive index over
    /// the refractive index of the enclosing media
    refraction_index: f64,
}

impl Dielectric {
    pub fn new(refraction_index: f64) -> Self {
        Self { refraction_index }
    }

    fn reflectance(cos: f64, refraction_idx: f64) -> f64 {
        // The likelihood of a ray to reflect, based on the Schlck approximation
        let r0 = (1.0 - refraction_idx) / (1.0 + refraction_idx);
        let r0 = r0 * r0;
        r0 + (1.0 - r0) * (1.0 - cos).powf(5.0)
    }
}

impl Material for Dielectric {
    fn scatter(&self, ray_in: &Ray3, record: &HitRecord) -> Option<MaterialResult> {
        // exiting the material, the refraction index is reversed.
        // air has a refraction index of =~ 1.0
        let ri = if record.front_face() {
            1.0 / self.refraction_index
        } else {
            self.refraction_index
        };
        let direction = ray_in.direction().as_unit();
        let cos_theta = (-direction).dot(&record.normal()).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = ri * sin_theta > 1.0;

        let will_reflect = cannot_refract || (Dielectric::reflectance(cos_theta, ri) > random());

        let direction = if will_reflect {
            Vec3::from(direction).reflect(&record.normal())
        } else {
            direction.refract(&record.normal(), ri)
        };

        Some(MaterialResult {
            attenuation: Color::white(),
            scattered: Ray3::new(record.point(), direction),
        })
    }
}
