use crate::{vec::Normalized, Interval, Material, Point3, Ray3, Vec3};

#[derive(Debug, Clone)]
pub struct HitRecord<'a> {
    point: Point3,
    normal: Vec3<Normalized>,
    material: &'a dyn Material,
    t: f64,
    front_face: bool,
}

impl<'a> HitRecord<'a> {
    pub fn point(&self) -> Point3 {
        self.point
    }

    pub fn normal(&self) -> Vec3<Normalized> {
        self.normal
    }

    pub fn t(&self) -> f64 {
        self.t
    }

    pub fn material(&self) -> &'a (dyn Material + 'a) {
        self.material
    }

    pub fn front_face(&self) -> bool {
        self.front_face
    }

    pub fn from_incoming_ray(
        ray: &Ray3,
        point: &Point3,
        normal: &Vec3<Normalized>,
        t: f64,
        material: &'a dyn Material,
    ) -> Self {
        let front_face = Vec3::dot(&ray.direction(), normal) < 0.0;
        let normal = if front_face { *normal } else { -*normal };
        Self {
            point: *point,
            normal,
            t,
            front_face,
            material,
        }
    }

    pub fn set_face_normal(&mut self, ray: &Ray3, outward_normal: &Vec3<Normalized>) {
        self.front_face = Vec3::dot(&ray.direction(), outward_normal) < 0.0;
        self.normal = if self.front_face {
            *outward_normal
        } else {
            -*outward_normal
        };
    }
}

pub trait Hittable: std::fmt::Debug {
    fn hit(&self, ray: &Ray3, ray_t: Interval) -> Option<HitRecord>;
}

#[derive(Debug)]
pub struct Sphere<'a> {
    center: Point3,
    radius: f64,
    material: &'a dyn Material,
}

impl<'a> Sphere<'a> {
    pub fn new(center: Point3, radius: f64, material: &'a dyn Material) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
}

impl Hittable for Sphere<'_> {
    fn hit(&self, ray: &Ray3, ray_t: Interval) -> Option<HitRecord> {
        let oc = self.center - ray.origin();
        let a = ray.direction().len_squared();
        let h = Vec3::dot(&ray.direction(), &oc);
        let c = oc.len_squared() - self.radius * self.radius;

        let discriminant = h * h - a * c;
        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();

        // Find the nearest root that lies in the acceptable range.
        let root = (h - sqrtd) / a;
        if !ray_t.surrounds(root) {
            let root = (h + sqrtd) / a;
            if !ray_t.surrounds(root) {
                return None;
            }
        }

        let point = ray.at(root);
        // mathematically guaranteed to be normalized
        let normal = ((point - self.center) / self.radius).assert_is_normalized();
        Some(HitRecord::from_incoming_ray(
            ray,
            &point,
            &normal,
            root,
            self.material,
        ))
    }
}

#[derive(Debug, Default)]
pub struct HittableVec<'a> {
    objects: Vec<&'a dyn Hittable>,
}

impl<'a> HittableVec<'a> {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn add(&mut self, obj: &'a dyn Hittable) {
        self.objects.push(obj);
    }
}

impl Hittable for HittableVec<'_> {
    fn hit(&self, ray: &Ray3, ray_t: Interval) -> Option<HitRecord> {
        let mut closest_record: Option<HitRecord> = None;
        let mut closest_dist = *ray_t.end();

        for object in &self.objects {
            if let Some(record) = object.hit(ray, Interval::new(*ray_t.start(), closest_dist)) {
                closest_dist = record.t;
                closest_record = Some(record);
            }
        }

        closest_record
    }
}
