use std::{f64::consts::PI, rc::Rc};

use crate::{
    boundingbox::BoundingBox3, vec::Normalized, Interval, Material, Point2, Point3, Ray3, Ray4,
    Vec3,
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
        u: f64,
        v: f64,
        material: Rc<dyn Material>,
    ) -> Self {
        let front_face = Vec3::dot(&ray.direction(), normal) < 0.0;
        let normal = if front_face { *normal } else { -*normal };
        Self {
            point: *point,
            normal,
            t,
            u,
            v,
            front_face,
            material,
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

    fn get_uv(point: &Point3) -> Point2 {
        // `point` must be inside the unit sphere
        assert!((Point3::origin() - point).len_squared() <= 1.0001);

        let theta = f64::acos(-point.y());
        let phi = f64::atan2(-point.z(), point.x()) + PI;

        // u and v range from [0.0, 1.0]
        let u = phi / (2.0 * PI);
        let v = theta / PI;

        Point2::new(u, v)
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

        let (u, v) = Sphere::get_uv(&Vec3::from(normal).into()).into();
        Some(HitRecord::from_incoming_ray(
            ray,
            &point,
            &normal,
            root,
            u,
            v,
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

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            objects: Vec::with_capacity(cap),
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

#[derive(Debug)]
pub struct Parallelogram {
    corner: Point3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    d: f64,
    normal: Vec3<Normalized>,
    material: Rc<dyn Material>,
    bounding_box: BoundingBox3,
}

impl Parallelogram {
    pub fn new(corner: Point3, u: Vec3, v: Vec3, material: Rc<dyn Material>) -> Self {
        let diag_1 = BoundingBox3::bounded_by(&corner, &(corner + u + v));
        let diag_2 = BoundingBox3::bounded_by(&(corner + u), &(corner + v));

        let bounding_box = BoundingBox3::extending(&diag_1, &diag_2);

        let n = Vec3::cross(&u, &v);
        let normal = n.as_unit();
        let d = Vec3::dot(&normal, &Vec3::from(corner));

        let w = n / Vec3::dot(&n, &n);

        Self {
            corner,
            u,
            v,
            d,
            w,
            normal,
            material,
            bounding_box,
        }
    }

    /// Checks whether the object is hit, assuming the plane it exists on is hit
    /// and given (a, b), the coordinates on the plane relative to the
    /// object's u and v vectors.
    ///
    /// Returns None if the object is missed, otherwise Some(u, v) where u, v are the
    /// appropriate UV coordinates.
    pub fn is_interior(&self, a: f64, b: f64) -> Option<(f64, f64)> {
        let range = 0.0..=1.0;

        if !range.contains(&a) || !range.contains(&b) {
            return None;
        }

        // a, b are identical to u, v coordinates;
        // both are in fractional space
        Some((a, b))
    }
}

impl Hittable for Parallelogram {
    fn hit(&self, ray: &Ray4, ray_t: Interval) -> Option<HitRecord> {
        let demon = Vec3::dot(&self.normal, &ray.direction());

        // ray is parallel to the plane; no hit
        if demon.abs() < 1e-8 {
            return None;
        }

        let t = (self.d - Vec3::dot(&self.normal, &Vec3::from(ray.origin()))) / demon;
        if !ray_t.contains(t) {
            return None;
        }

        // check if the hit point is within the planar shape using its planar coordinates
        let intersection = ray.at(t);
        let planar_hit_vec = intersection - self.corner;
        let alpha = Vec3::dot(&self.w, &Vec3::cross(&planar_hit_vec, &self.v));
        let beta = Vec3::dot(&self.w, &Vec3::cross(&self.u, &planar_hit_vec));

        let Some((u, v)) = self.is_interior(alpha, beta) else {
            return None;
        };

        Some(HitRecord::from_incoming_ray(
            ray,
            &intersection,
            &self.normal,
            t,
            u,
            v,
            Rc::clone(&self.material),
        ))
    }

    fn bounding_box(&self) -> Option<&BoundingBox3> {
        Some(&self.bounding_box)
    }
}

pub fn box3(a: &Point3, b: &Point3, mat: Rc<dyn Material>) -> Rc<dyn Hittable> {
    let mut sides = HittableVec::with_capacity(6);

    // Construct the two opposite vertices with the minimum and maximum coordinates.
    let min = Point3::new(
        f64::min(a.x(), b.x()),
        f64::min(a.y(), b.y()),
        f64::min(a.z(), b.z()),
    );
    let max = Point3::new(
        f64::max(a.x(), b.x()),
        f64::max(a.y(), b.y()),
        f64::max(a.z(), b.z()),
    );

    let dx = Vec3::new(max.x() - min.x(), 0.0, 0.0);
    let dy = Vec3::new(0.0, max.y() - min.y(), 0.0);
    let dz = Vec3::new(0.0, 0.0, max.z() - min.z());

    sides.add(Rc::new(Parallelogram::new(
        Point3::new(min.x(), min.y(), max.z()),
        dx,
        dy,
        Rc::clone(&mat),
    )));
    sides.add(Rc::new(Parallelogram::new(
        Point3::new(max.x(), min.y(), max.z()),
        -dz,
        dy,
        Rc::clone(&mat),
    )));
    sides.add(Rc::new(Parallelogram::new(
        Point3::new(max.x(), min.y(), min.z()),
        -dx,
        dy,
        Rc::clone(&mat),
    )));
    sides.add(Rc::new(Parallelogram::new(
        Point3::new(min.x(), min.y(), min.z()),
        dz,
        dy,
        Rc::clone(&mat),
    )));
    sides.add(Rc::new(Parallelogram::new(
        Point3::new(min.x(), max.y(), max.z()),
        dx,
        -dz,
        Rc::clone(&mat),
    )));
    sides.add(Rc::new(Parallelogram::new(
        Point3::new(min.x(), min.y(), min.z()),
        dx,
        dz,
        Rc::clone(&mat),
    )));

    Rc::new(sides)
}

// TODO: make constructor functions more ergonomic (f.e. define three points)
#[derive(Debug)]
pub struct Triangle {
    corner: Point3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    d: f64,
    normal: Vec3<Normalized>,
    material: Rc<dyn Material>,
    bounding_box: BoundingBox3,
}

impl Triangle {
    pub fn new(corner: Point3, u: Vec3, v: Vec3, material: Rc<dyn Material>) -> Self {
        let diag_1 = BoundingBox3::bounded_by(&corner, &(corner + u + v));
        let diag_2 = BoundingBox3::bounded_by(&(corner + u), &(corner + v));

        let bounding_box = BoundingBox3::extending(&diag_1, &diag_2);

        let n = Vec3::cross(&u, &v);
        let normal = n.as_unit();
        let d = Vec3::dot(&normal, &Vec3::from(corner));

        let w = n / Vec3::dot(&n, &n);

        Self {
            corner,
            u,
            v,
            d,
            w,
            normal,
            material,
            bounding_box,
        }
    }

    /// Checks whether the object is hit, assuming the plane it exists on is hit
    /// and given (a, b), the coordinates on the plane relative to the
    /// object's u and v vectors.
    ///
    /// Returns None if the object is missed, otherwise Some(u, v) where u, v are the
    /// appropriate UV coordinates.
    pub fn is_interior(&self, a: f64, b: f64) -> Option<(f64, f64)> {
        if a < 0.0 || b < 0.0 || (a + b) > 1.0 {
            return None;
        }

        // a, b are identical to u, v coordinates;
        // both are in fractional space
        Some((a, b))
    }
}

impl Hittable for Triangle {
    fn hit(&self, ray: &Ray4, ray_t: Interval) -> Option<HitRecord> {
        let demon = Vec3::dot(&self.normal, &ray.direction());

        // ray is parallel to the plane; no hit
        if demon.abs() < 1e-8 {
            return None;
        }

        let t = (self.d - Vec3::dot(&self.normal, &Vec3::from(ray.origin()))) / demon;
        if !ray_t.contains(t) {
            return None;
        }

        // check if the hit point is within the planar shape using its planar coordinates
        let intersection = ray.at(t);
        let planar_hit_vec = intersection - self.corner;
        let alpha = Vec3::dot(&self.w, &Vec3::cross(&planar_hit_vec, &self.v));
        let beta = Vec3::dot(&self.w, &Vec3::cross(&self.u, &planar_hit_vec));

        let Some((u, v)) = self.is_interior(alpha, beta) else {
            return None;
        };

        Some(HitRecord::from_incoming_ray(
            ray,
            &intersection,
            &self.normal,
            t,
            u,
            v,
            Rc::clone(&self.material),
        ))
    }

    fn bounding_box(&self) -> Option<&BoundingBox3> {
        Some(&self.bounding_box)
    }
}

// TODO: make constructor functions more ergonomic (f.e. define center, radius, etc.)
#[derive(Debug)]
pub struct Disc {
    corner: Point3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    d: f64,
    normal: Vec3<Normalized>,
    material: Rc<dyn Material>,
    bounding_box: BoundingBox3,
}

impl Disc {
    pub fn new(corner: Point3, u: Vec3, v: Vec3, material: Rc<dyn Material>) -> Self {
        let diag_1 = BoundingBox3::bounded_by(&corner, &(corner + u + v));
        let diag_2 = BoundingBox3::bounded_by(&(corner + u), &(corner + v));

        let bounding_box = BoundingBox3::extending(&diag_1, &diag_2);

        let n = Vec3::cross(&u, &v);
        let normal = n.as_unit();
        let d = Vec3::dot(&normal, &Vec3::from(corner));

        let w = n / Vec3::dot(&n, &n);

        Self {
            corner,
            u,
            v,
            d,
            w,
            normal,
            material,
            bounding_box,
        }
    }

    /// Checks whether the object is hit, assuming the plane it exists on is hit
    /// and given (a, b), the coordinates on the plane relative to the
    /// object's u and v vectors.
    ///
    /// Returns None if the object is missed, otherwise Some(u, v) where u, v are the
    /// appropriate UV coordinates.
    pub fn is_interior(&self, a: f64, b: f64) -> Option<(f64, f64)> {
        let a_c_dist = (0.5 - a).abs();
        let b_c_dist = (0.5 - b).abs();

        if a < 0.0 || b < 0.0 || (a_c_dist.powi(2) + b_c_dist.powi(2)).sqrt() > 0.5 {
            return None;
        }

        // a, b are identical to u, v coordinates;
        // both are in fractional space
        Some((a, b))
    }
}

impl Hittable for Disc {
    fn hit(&self, ray: &Ray4, ray_t: Interval) -> Option<HitRecord> {
        let demon = Vec3::dot(&self.normal, &ray.direction());

        // ray is parallel to the plane; no hit
        if demon.abs() < 1e-8 {
            return None;
        }

        let t = (self.d - Vec3::dot(&self.normal, &Vec3::from(ray.origin()))) / demon;
        if !ray_t.contains(t) {
            return None;
        }

        // check if the hit point is within the planar shape using its planar coordinates
        let intersection = ray.at(t);
        let planar_hit_vec = intersection - self.corner;
        let alpha = Vec3::dot(&self.w, &Vec3::cross(&planar_hit_vec, &self.v));
        let beta = Vec3::dot(&self.w, &Vec3::cross(&self.u, &planar_hit_vec));

        let Some((u, v)) = self.is_interior(alpha, beta) else {
            return None;
        };

        Some(HitRecord::from_incoming_ray(
            ray,
            &intersection,
            &self.normal,
            t,
            u,
            v,
            Rc::clone(&self.material),
        ))
    }

    fn bounding_box(&self) -> Option<&BoundingBox3> {
        Some(&self.bounding_box)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn sphere_uv_conversions() {
        let uv_tests = [
            (Point3::new(1.0, 0.0, 0.0), Point2::new(0.5, 0.5)),
            (Point3::new(-1.0, 0.0, 0.0), Point2::new(0.0, 0.5)),
            (Point3::new(0.0, 1.0, 0.0), Point2::new(0.5, 1.0)),
            (Point3::new(0.0, -1.0, 0.0), Point2::new(0.5, 0.0)),
            (Point3::new(0.0, 0.0, 1.0), Point2::new(0.25, 0.5)),
            (Point3::new(0.0, 0.0, -1.0), Point2::new(0.75, 0.5)),
        ];
        for (point, res) in uv_tests {
            assert_eq!(Sphere::get_uv(&point), res);
        }
    }
}
