use crate::{Axis, HitRecord, Hittable, Interval, Point3, Ray3, Ray4};
use std::{cmp::Ordering, rc::Rc};

#[derive(Debug, Clone)]
pub struct BoundingBox3 {
    x: Interval,
    y: Interval,
    z: Interval,
}

impl BoundingBox3 {
    /// Create a bounding box with the provided intervals.
    /// Ensure that all intervals are positive (start <= end).
    pub fn new(x: Interval, y: Interval, z: Interval) -> Self {
        // no interval may be narrower than DELTA
        const DELTA: f64 = 0.0001;

        let expand = |int: Interval| {
            if int.size() < DELTA {
                int.expand(DELTA)
            } else {
                int
            }
        };

        let x = expand(x);
        let y = expand(y);
        let z = expand(z);
        Self { x, y, z }
    }

    /// Create an empty bounding box.
    pub const fn empty() -> Self {
        Self {
            x: Interval::empty(),
            y: Interval::empty(),
            z: Interval::empty(),
        }
    }

    /// Create a bounding box including everything possible.
    pub const fn universe() -> Self {
        Self {
            x: Interval::universe(),
            y: Interval::universe(),
            z: Interval::universe(),
        }
    }

    /// Create a bounding box with the provided points as opposite corners.
    pub fn bounded_by(a: &Point3, b: &Point3) -> Self {
        let x = Interval::positive(a.x(), b.x());
        let y = Interval::positive(a.y(), b.y());
        let z = Interval::positive(a.z(), b.z());
        Self::new(x, y, z)
    }

    pub fn extending(a: &BoundingBox3, b: &BoundingBox3) -> Self {
        let x = Interval::surrounding(&a.x, &b.x);
        let y = Interval::surrounding(&a.y, &b.y);
        let z = Interval::surrounding(&a.z, &b.z);
        Self::new(x, y, z)
    }

    pub fn extending_opt(a: Option<&BoundingBox3>, b: Option<&BoundingBox3>) -> Self {
        match (a, b) {
            (None, None) => Self::empty(),
            (None, Some(b)) | (Some(b), None) => Self::new(b.x.clone(), b.y.clone(), b.z.clone()),
            (Some(a), Some(b)) => Self::extending(a, b),
        }
    }

    /// Gets the longest axis of the bounding box.
    /// If this bounding box is empty, this function will arbitrarily return [`Axis::X`].
    pub fn longest_axis(&self) -> Axis {
        let axes = [
            (Axis::X, self.x.size()),
            (Axis::Y, self.y.size()),
            (Axis::Z, self.z.size()),
        ];

        let (axis, _) = axes.iter().fold(
            (Axis::X, 0.0),
            |max, this| {
                if this.1 > max.1 {
                    *this
                } else {
                    max
                }
            },
        );

        axis
    }

    fn hit(&self, ray: &Ray3, ray_t: Interval) -> bool {
        let mut ray_t = ray_t;
        for axis in Axis::iter() {
            let ax = &self[axis];
            let adinv = 1.0 / ray.direction()[axis];

            let t0 = (ax.start() - ray.origin()[axis]) * adinv;
            let t1 = (ax.end() - ray.origin()[axis]) * adinv;

            let t_int = Interval::positive(t0, t1);
            if let Some(new_int) = t_int.overlap(&ray_t) {
                ray_t = new_int;
            } else {
                return false;
            }
        }
        true
    }

    pub fn x(&self) -> &Interval {
        &self.x
    }

    pub fn y(&self) -> &Interval {
        &self.y
    }

    pub fn z(&self) -> &Interval {
        &self.z
    }
}

impl std::ops::Index<Axis> for BoundingBox3 {
    type Output = Interval;

    fn index(&self, index: Axis) -> &Self::Output {
        match index {
            Axis::X => self.x(),
            Axis::Y => self.y(),
            Axis::Z => self.z(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BVHNode {
    left: Rc<dyn Hittable>,
    right: Rc<dyn Hittable>,
    bbox: BoundingBox3,
}

impl BVHNode {
    pub fn new(mut objects: Vec<Rc<dyn Hittable>>) -> Self {
        let mut bbox = BoundingBox3::empty();
        for object in &objects {
            bbox = BoundingBox3::extending_opt(Some(bbox).as_ref(), object.bounding_box());
        }

        let axis = bbox.longest_axis();

        let comparator = |a: &Rc<dyn Hittable>, b: &Rc<dyn Hittable>| Self::cmp_box(a, b, axis);

        let (left, right) = match objects.len() {
            1 => (Rc::clone(&objects[0]), Rc::clone(&objects[0])),
            2 => (Rc::clone(&objects[0]), Rc::clone(&objects[1])),
            _ => {
                objects.sort_unstable_by(comparator);

                let mid = objects.len() / 2;
                let split = objects.split_off(mid);

                let left: Rc<dyn Hittable> = Rc::new(BVHNode::new(objects));
                let right: Rc<dyn Hittable> = Rc::new(BVHNode::new(split));

                (left, right)
            }
        };

        Self { left, right, bbox }
    }

    fn cmp_box<'a>(a: &'a Rc<dyn Hittable>, b: &'a Rc<dyn Hittable>, axis: Axis) -> Ordering {
        let a_ax_int = &a.bounding_box().unwrap()[axis];
        let b_ax_int = &b.bounding_box().unwrap()[axis];
        a_ax_int
            .start()
            .partial_cmp(b_ax_int.start())
            .expect("Tried to cmp a NaN value")
    }
}

impl Hittable for BVHNode {
    fn hit(&self, ray: &Ray4, ray_t: Interval) -> Option<HitRecord> {
        if !self.bbox.hit(&ray.ignore_time(), ray_t.clone()) {
            return None;
        }

        let hit_left = self.left.hit(ray, ray_t.clone());
        let hit_right = match hit_left {
            Some(ref hit) => self.right.hit(ray, Interval::new(*ray_t.start(), hit.t())),
            None => self.right.hit(ray, ray_t),
        };

        hit_right.or(hit_left)
    }

    fn bounding_box(&self) -> Option<&BoundingBox3> {
        Some(&self.bbox)
    }
}
