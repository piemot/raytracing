use std::rc::Rc;

use rand::random;

use crate::{texture::SolidColor, Color, HitRecord, OrthonormalBasis, Point3, Ray4, Texture, Vec3};

#[derive(Debug)]
pub struct MaterialResult {
    pub attenuation: Color,
    pub scattered: Ray4,
    pub pdf: f64,
}

pub trait Material: std::fmt::Debug {
    fn scatter(&self, ray_in: &Ray4, record: &HitRecord) -> Option<MaterialResult>;
    fn emitted(&self, ray_in: &Ray4, record: &HitRecord, u: f64, v: f64, point: &Point3) -> Color {
        Color::black()
    }

    fn into_mat(self) -> Rc<dyn Material>
    where
        Self: Sized + 'static,
    {
        Rc::new(self)
    }

    fn scattering_pdf(&self, ray_in: &Ray4, record: &HitRecord, scattered: &Ray4) -> f64 {
        unimplemented!();
    }
}

#[derive(Debug)]
pub struct Lambertian(Rc<dyn Texture>);

impl Lambertian {
    pub fn new(texture: Rc<dyn Texture>) -> Self {
        Self(texture)
    }

    pub fn solid(albedo: Color) -> Self {
        Self(Rc::new(SolidColor::new(albedo)))
    }
}

impl Material for Lambertian {
    // Lambertian materials are independant of the incoming ray due to Lambert's Cosine Law.
    fn scatter(&self, ray_in: &Ray4, record: &HitRecord) -> Option<MaterialResult> {
        let uvw = OrthonormalBasis::new(&record.normal().into());
        let scatter_dir = uvw.transform(&Vec3::random_on_sphere_cosine());

        let scattered = Ray4::new(record.point(), scatter_dir.as_unit().into(), ray_in.time());
        Some(MaterialResult {
            attenuation: self.0.value(record.u(), record.v(), &record.point()),
            pdf: Vec3::dot(&uvw.w(), &scattered.direction()) / std::f64::consts::PI,
            scattered,
        })
    }

    fn scattering_pdf(&self, ray_in: &Ray4, record: &HitRecord, scattered: &Ray4) -> f64 {
        let cos_theta = Vec3::dot(&record.normal(), &scattered.direction().as_unit());
        return f64::max(0.0, cos_theta / std::f64::consts::PI);
    }
}

#[derive(Debug)]
pub struct DiffuseLight(Rc<dyn Texture>);

impl DiffuseLight {
    pub fn new(texture: Rc<dyn Texture>) -> Self {
        Self(texture)
    }

    pub fn solid(albedo: Color) -> Self {
        Self(Rc::new(SolidColor::new(albedo)))
    }
}

impl Material for DiffuseLight {
    // DiffuseLight does not scatter.
    fn scatter(&self, _ray_in: &Ray4, _record: &HitRecord) -> Option<MaterialResult> {
        None
    }

    fn emitted(&self, _ray_in: &Ray4, record: &HitRecord, u: f64, v: f64, point: &Point3) -> Color {
        // light is unidirectional
        if record.front_face() {
            self.0.value(u, v, point)
        } else {
            Color::black()
        }
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
    fn scatter(&self, ray_in: &Ray4, record: &HitRecord) -> Option<MaterialResult> {
        let reflected = Vec3::reflect(&ray_in.direction(), &record.normal());
        let reflected = reflected.as_unit() + (self.fuzz * Vec3::random_in_unit_sphere());
        let scattered = Ray4::new(record.point(), reflected, ray_in.time());

        if Vec3::dot(&reflected, &record.normal()) < 0.0 {
            // if the ray has been scattered below the surface of the object
            return None;
        }

        Some(MaterialResult {
            attenuation: self.albedo,
            pdf: todo!(),
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
    fn scatter(&self, ray_in: &Ray4, record: &HitRecord) -> Option<MaterialResult> {
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
            pdf: todo!(),
            scattered: Ray4::new(record.point(), direction, ray_in.time()),
        })
    }
}

#[derive(Debug)]
pub struct Isotropic(Rc<dyn Texture>);

impl Isotropic {
    pub fn new(texture: Rc<dyn Texture>) -> Self {
        Self(texture)
    }

    pub fn colored(color: Color) -> Self {
        Self(SolidColor::new(color).into_texture())
    }
}

impl Material for Isotropic {
    fn scatter(&self, ray_in: &Ray4, record: &HitRecord) -> Option<MaterialResult> {
        let scattered = Ray4::new(record.point(), Vec3::random_in_unit_sphere(), ray_in.time());
        let attenuation = self.0.value(record.u(), record.v(), &record.point());

        Some(MaterialResult {
            pdf: 1.0 / (4.0 * std::f64::consts::PI),
            attenuation,
            scattered,
        })
    }

    fn scattering_pdf(&self, _: &Ray4, _: &HitRecord, _: &Ray4) -> f64 {
        1.0 / (4.0 * std::f64::consts::PI)
    }
}
