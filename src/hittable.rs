use std::rc::Rc;

use crate::{
    boundingbox::BoundingBox3, vec::Normalized, Interval, Material, Point3, Ray3, Ray4, Vec3,
};

#[derive(Debug, Clone)]
pub struct HitRecord {
    // The point where the ray hit the object
    point: Point3,
    // The normal vector of the object at the point hit
    normal: Vec3<Normalized>,
    // The material of the hit surface
    material: Rc<dyn Material>,
    // uv texturer coordinates
    u: f64,
    v: f64,
    // Point on the ray that the hit occurred at
    t: f64,
    // Whether the ray hit the front or back face of the object
    front_face: bool,
}

impl HitRecord {
    pub fn point(&self) -> Point3 {
        self.point
    }

    pub fn normal(&self) -> Vec3<Normalized> {
        self.normal
    }

    pub fn t(&self) -> f64 {
        self.t
    }

    pub fn u(&self) -> f64 {
        self.u
    }

    pub fn v(&self) -> f64 {
        self.v
    }

    pub fn material(&self) -> Rc<dyn Material> {
        Rc::clone(&self.material)
    }

    pub fn front_face(&self) -> bool {
        self.front_face
    }

    pub fn from_incoming_ray(
        ray: &Ray4,
        point: &Point3,
        normal: &Vec3<Normalized>,
        t: f64,
        material: Rc<dyn Material>,
    ) -> Self {
        let front_face = Vec3::dot(&ray.direction(), normal) < 0.0;
        let normal = if front_face { *normal } else { -*normal };
        Self {
            point: *point,
            normal,
            t,
            front_face,
            material,
            u: f64::NAN,
            v: f64::NAN,
        }
    }

    pub fn set_face_normal(&mut self, ray: &Ray4, outward_normal: &Vec3<Normalized>) {
        self.front_face = Vec3::dot(&ray.direction(), outward_normal) < 0.0;
        self.normal = if self.front_face {
            *outward_normal
        } else {
            -*outward_normal
        };
    }
}

pub trait Hittable: std::fmt::Debug {
    // Attempts to hit the object, at a given time.
    // If hit, the object should return Hit(HitRecord) describing how the hit occurred.
    fn hit(&self, ray: &Ray4, ray_t: Interval) -> Option<HitRecord>;

    // can return None, but will never recieve any [hit()]s.
    fn bounding_box(&self) -> Option<&BoundingBox3>;
}

#[derive(Debug)]
pub struct Sphere {
    center: Ray3,
    radius: f64,
    material: Rc<dyn Material>,
    bounding_box: BoundingBox3,
}

impl Sphere {
    pub fn stationary(center: Point3, radius: f64, material: Rc<dyn Material>) -> Self {
        Self::new(Ray3::new(center, Vec3::empty()), radius, material)
    }

    pub fn new(center: Ray3, radius: f64, material: Rc<dyn Material>) -> Self {
        assert!(radius >= 0.0);
        let rad_vec = Vec3::new(radius, radius, radius);

        // bb at time = 0
        let box0 =
            BoundingBox3::bounded_by(&(center.origin() - rad_vec), &(center.origin() + rad_vec));

        // bb at time = 1
        let box1 =
            BoundingBox3::bounded_by(&(center.at(1.0) - rad_vec), &(center.at(1.0) + rad_vec));

        Self {
            center,
            radius,
            material,
            bounding_box: BoundingBox3::extending(&box0, &box1),
        }
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray4, ray_t: Interval) -> Option<HitRecord> {
        let current_center = self.center.at(ray.time());
        let oc = current_center - ray.origin();
        let a = ray.direction().len_squared();
        let h = Vec3::dot(&ray.direction(), &oc);
        let c = oc.len_squared() - self.radius * self.radius;

        let discriminant = h * h - a * c;
        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();

        // Find the nearest root that lies in the acceptable range.
        let mut root = (h - sqrtd) / a;
        if !ray_t.surrounds(root) {
            root = (h + sqrtd) / a;
            if !ray_t.surrounds(root) {
                return None;
            }
        }

        let point = ray.at(root);
        // mathematically guaranteed to be normalized
        let normal = ((point - current_center) / self.radius).assert_is_normalized();
        Some(HitRecord::from_incoming_ray(
            ray,
            &point,
            &normal,
            root,
            Rc::clone(&self.material),
        ))
    }

    fn bounding_box(&self) -> Option<&BoundingBox3> {
        Some(&self.bounding_box)
    }
}

#[derive(Debug, Default)]
pub struct HittableVec {
    pub(super) objects: Vec<Rc<dyn Hittable>>,
    pub(super) bounding_box: Option<BoundingBox3>,
}

impl Into<Vec<Rc<dyn Hittable>>> for HittableVec {
    fn into(self) -> Vec<Rc<dyn Hittable>> {
        self.objects
    }
}

impl HittableVec {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            bounding_box: None,
        }
    }

    pub fn add(&mut self, obj: Rc<dyn Hittable>) {
        self.bounding_box = match &self.bounding_box {
            Some(bbox) => Some(BoundingBox3::extending(bbox, obj.bounding_box().unwrap())),
            None => Some(obj.bounding_box().unwrap().clone()),
        };
        self.objects.push(obj);
    }
}

impl Hittable for HittableVec {
    fn hit(&self, ray: &Ray4, ray_t: Interval) -> Option<HitRecord> {
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

    fn bounding_box(&self) -> Option<&BoundingBox3> {
        self.bounding_box.as_ref()
    }
}

impl FromIterator<Rc<dyn Hittable>> for HittableVec {
    fn from_iter<T: IntoIterator<Item = Rc<dyn Hittable>>>(iter: T) -> Self {
        let mut this = HittableVec::new();
        for obj in iter {
            this.add(obj);
        }
        this
    }
}
