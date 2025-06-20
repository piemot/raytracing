use std::{
    f64::{self, consts::PI},
    rc::Rc,
};

use rand::random;

use crate::{
    boundingbox::BoundingBox3, material::Isotropic, texture::Texture, vec::Normalized, Axis, Color,
    Interval, Material, Point2, Point3, Ray3, Ray4, Vec3,
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

    fn pdf_value(&self, origin: &Point3, direction: &Vec3) -> f64 {
        let _ = (origin, direction);
        unimplemented!();
    }

    fn random(&self, origin: &Point3) -> Vec3 {
        let _ = origin;
        unimplemented!();
    }

    fn hittable(self) -> Rc<dyn Hittable>
    where
        Self: Sized + 'static,
    {
        Rc::new(self)
    }
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

impl From<HittableVec> for Vec<Rc<dyn Hittable>> {
    fn from(val: HittableVec) -> Self {
        val.objects
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

    pub fn len(&self) -> usize {
        self.objects.len()
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
    area: f64,
    normal: Vec3<Normalized>,
    material: Rc<dyn Material>,
    bounding_box: BoundingBox3,
}

impl Parallelogram {
    pub fn new(corner: Point3, u: Vec3, v: Vec3, material: Rc<dyn Material>) -> Self {
        let diag_1 = BoundingBox3::bounded_by(&corner, &(corner + u + v));
        let diag_2 = BoundingBox3::bounded_by(&(corner + u), &(corner + v));

        let bounding_box = BoundingBox3::extending(&diag_1, &diag_2);

        let n = u.cross(&v);
        let normal = n.as_unit();
        let d = Vec3::dot(&normal, &Vec3::from(corner));

        let w = n / Vec3::dot(&n, &n);
        let area = n.len();

        Self {
            corner,
            u,
            v,
            d,
            w,
            area,
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
        let alpha = Vec3::dot(&self.w, &planar_hit_vec.cross(&self.v));
        let beta = Vec3::dot(&self.w, &self.u.cross(&planar_hit_vec));

        let (u, v) = self.is_interior(alpha, beta)?;

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

    fn pdf_value(&self, origin: &Point3, direction: &Vec3) -> f64 {
        let Some(hit) = self.hit(
            &Ray4::new(*origin, *direction, 0.0),
            Interval::new(0.001, f64::INFINITY),
        ) else {
            return 0.0;
        };

        let dist_squared = hit.t() * hit.t() * direction.len_squared();
        let cosine = (direction.dot(&hit.normal()) / direction.len()).abs();

        dist_squared / (cosine * self.area)
    }

    fn random(&self, origin: &Point3) -> Vec3 {
        let p: Point3 =
            self.corner + (rand::random::<f64>() * self.u) + (rand::random::<f64>() * self.v);
        p - origin
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

        let n = u.cross(&v);
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

    pub fn from_points(
        corner1: Point3,
        corner2: Point3,
        corner3: Point3,
        material: Rc<dyn Material>,
    ) -> Self {
        Self::new(corner1, corner2 - corner1, corner3 - corner1, material)
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
        let alpha = Vec3::dot(&self.w, &planar_hit_vec.cross(&self.v));
        let beta = Vec3::dot(&self.w, &self.u.cross(&planar_hit_vec));

        let (u, v) = self.is_interior(alpha, beta)?;

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

        let n = u.cross(&v);
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

    /// Define a [`Disc`] by its center and radius vectors `u` and `v`.
    pub fn from_center(center: Point3, u: Vec3, v: Vec3, material: Rc<dyn Material>) -> Self {
        let diag_1 = BoundingBox3::bounded_by(&(center - u - v), &(center + u + v));
        let diag_2 = BoundingBox3::bounded_by(&(center + u - v), &(center - u + v));

        let bounding_box = BoundingBox3::extending(&diag_1, &diag_2);

        let u2: Vec3 = u * 2;
        let n = u2.cross(&(2 * v));
        let normal = n.as_unit();
        let d = Vec3::dot(&normal, &Vec3::from(center - u - v));

        let w = n / Vec3::dot(&n, &n);

        Self {
            corner: center - u - v,
            u: u * 2,
            v: v * 2,
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
        let alpha = Vec3::dot(&self.w, &planar_hit_vec.cross(&self.v));
        let beta = Vec3::dot(&self.w, &self.u.cross(&planar_hit_vec));

        let (u, v) = self.is_interior(alpha, beta)?;

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

#[derive(Debug)]
pub struct Translate {
    object: Rc<dyn Hittable>,
    offset: Vec3,
    bounding_box: BoundingBox3,
}

impl Translate {
    pub fn new(object: Rc<dyn Hittable>, offset: Vec3) -> Self {
        let bbox = object
            .bounding_box()
            .expect("Objects without bounding boxes should not be Translated")
            + offset;
        Self {
            object,
            offset,
            bounding_box: bbox,
        }
    }
}

impl Hittable for Translate {
    fn hit(&self, ray: &Ray4, ray_t: Interval) -> Option<HitRecord> {
        // Move the ray backwards by the offset
        let offset_ray = Ray4::new(ray.origin() - self.offset, ray.direction(), ray.time());

        // Determine whether an intersection exists along the offset ray (and if so, where)
        let mut hit = self.object.hit(&offset_ray, ray_t)?;

        // Move the intersection point forwards by the offset
        hit.point = hit.point + self.offset;
        Some(hit)
    }

    fn bounding_box(&self) -> Option<&BoundingBox3> {
        Some(&self.bounding_box)
    }
}

#[derive(Debug)]
pub struct RotateY {
    object: Rc<dyn Hittable>,
    sin_theta: f64,
    cos_theta: f64,
    bounding_box: BoundingBox3,
}

impl RotateY {
    pub fn new(object: Rc<dyn Hittable>, angle: f64) -> Self {
        let sin_theta = angle.sin();
        let cos_theta = angle.cos();
        let bbox = object
            .bounding_box()
            .expect("Objects without bounding boxes should not be Rotated");

        let mut min = Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
        let mut max = Point3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);

        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let x = f64::from(i) * bbox.x().end() + f64::from(1 - i) * bbox.x().start();
                    let y = f64::from(j) * bbox.y().end() + f64::from(1 - j) * bbox.y().start();
                    let z = f64::from(k) * bbox.z().end() + f64::from(1 - k) * bbox.z().start();

                    let newx = cos_theta * x + sin_theta * z;
                    let newz = -sin_theta * x + cos_theta * z;

                    let tester = Vec3::new(newx, y, newz);

                    for c in Axis::iter() {
                        min[c] = f64::min(min[c], tester[c]);
                        max[c] = f64::max(max[c], tester[c]);
                    }
                }
            }
        }

        Self {
            object,
            cos_theta,
            sin_theta,
            bounding_box: BoundingBox3::bounded_by(&min, &max),
        }
    }
}

impl Hittable for RotateY {
    fn hit(&self, ray: &Ray4, ray_t: Interval) -> Option<HitRecord> {
        let Self {
            cos_theta,
            sin_theta,
            ..
        } = self;
        // Transform the ray from world space to object space.

        let origin = Point3::new(
            (cos_theta * ray.origin().x()) - (sin_theta * ray.origin().z()),
            ray.origin().y(),
            (sin_theta * ray.origin().x()) + (cos_theta * ray.origin().z()),
        );

        let direction = Vec3::new(
            (cos_theta * ray.direction().x()) - (sin_theta * ray.direction().z()),
            ray.direction().y(),
            (sin_theta * ray.direction().x()) + (cos_theta * ray.direction().z()),
        );

        let rotated_ray = Ray4::new(origin, direction, ray.time());

        // Determine whether an intersection exists in object space (and if so, where).

        let mut hit = self.object.hit(&rotated_ray, ray_t)?;

        // Transform the intersection from object space back to world space.

        let point = Point3::new(
            (cos_theta * hit.point().x()) + (sin_theta * hit.point().z()),
            hit.point().y(),
            (-sin_theta * hit.point().x()) + (cos_theta * hit.point().z()),
        );

        let normal = Vec3::new(
            (cos_theta * hit.normal().x()) + (sin_theta * hit.normal().z()),
            hit.normal().y(),
            (-sin_theta * hit.normal().x()) + (cos_theta * hit.normal().z()),
        );

        hit.point = point;
        // the conversion from object space to world space should not affect the normalization
        // state of the vector.
        hit.normal = normal.assert_is_normalized();

        Some(hit)
    }

    fn bounding_box(&self) -> Option<&BoundingBox3> {
        Some(&self.bounding_box)
    }
}

#[derive(Debug)]
pub struct ConstantMedium {
    boundary: Rc<dyn Hittable>,
    /// equal to -1.0 / density
    inv_density: f64,
    phase_fn: Rc<dyn Material>,
}

impl ConstantMedium {
    pub fn new(boundary: Rc<dyn Hittable>, density: f64, texture: Rc<dyn Texture>) -> Self {
        Self {
            boundary,
            inv_density: -1.0 / density,
            phase_fn: Isotropic::new(texture).into_mat(),
        }
    }

    pub fn colored(boundary: Rc<dyn Hittable>, density: f64, color: Color) -> Self {
        Self {
            boundary,
            inv_density: -1.0 / density,
            phase_fn: Isotropic::colored(color).into_mat(),
        }
    }
}

impl Hittable for ConstantMedium {
    fn hit(&self, ray: &Ray4, ray_t: Interval) -> Option<HitRecord> {
        let mut rec1 = self.boundary.hit(ray, Interval::universe())?;
        let mut rec2 = self
            .boundary
            .hit(ray, Interval::new(rec1.t + 0.0001, f64::INFINITY))?;

        if rec1.t < *ray_t.start() {
            rec1.t = *ray_t.start();
        }
        if rec2.t > *ray_t.end() {
            rec2.t = *ray_t.end();
        }

        if rec1.t >= rec2.t {
            return None;
        }
        if rec1.t < 0.0 {
            rec1.t = 0.0;
        }

        let ray_len = ray.direction().len();
        let dist_inside_boundary = (rec2.t - rec1.t) * ray_len;
        let hit_dist = self.inv_density * f64::ln(random());

        if hit_dist > dist_inside_boundary {
            return None;
        }

        let t = rec1.t + hit_dist / ray_len;

        Some(HitRecord {
            t,
            point: ray.at(t),
            normal: Vec3::new(1.0, 0.0, 0.0).assert_is_normalized(), // arbitrary
            front_face: true,                                        // arbitrary
            material: Rc::clone(&self.phase_fn),
            u: f64::NAN,
            v: f64::NAN,
        })
    }

    fn bounding_box(&self) -> Option<&BoundingBox3> {
        self.boundary.bounding_box()
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
