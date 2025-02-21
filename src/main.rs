use raytracing::{
    hittable::{HittableVec, Sphere},
    Camera, Point3,
};

fn main() {
    let cam = Camera::new(400, 16.0 / 9.0);

    let mut world = HittableVec::new();
    let a = Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5);
    world.add(&a);
    let a = Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0);
    world.add(&a);

    cam.render(&world);
}
