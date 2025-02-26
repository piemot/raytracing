use raytracing::{
    hittable::{HittableVec, Sphere},
    material::{Dielectric, Lambertian, Metal},
    Camera, Color, Material, Point3,
};

fn main() {
    let cam = Camera::new(400, 16.0 / 9.0);

    let mat_ground: Box<dyn Material> = Box::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));
    let mat_center: Box<dyn Material> = Box::new(Lambertian::new(Color::new(0.1, 0.2, 0.5)));
    let mat_left: Box<dyn Material> = Box::new(Dielectric::new(1.5));
    let mat_bubble: Box<dyn Material> = Box::new(Dielectric::new(1.0 / 1.5));
    let mat_right: Box<dyn Material> = Box::new(Metal::with_fuzz(Color::new(0.8, 0.6, 0.2), 1.0));

    let mut world = HittableVec::new();
    let a = Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0, &*mat_ground);
    world.add(&a);
    let a = Sphere::new(Point3::new(0.0, 0.0, -1.2), 0.5, &*mat_center);
    world.add(&a);
    let a = Sphere::new(Point3::new(-1.0, 0.0, -1.0), 0.5, &*mat_left);
    world.add(&a);
    let a = Sphere::new(Point3::new(-1.0, 0.0, -1.0), 0.4, &*mat_bubble);
    world.add(&a);
    let a = Sphere::new(Point3::new(1.0, 0.0, -1.0), 0.5, &*mat_right);
    world.add(&a);

    cam.render(&world);
}
