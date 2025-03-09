use rand::{random_range, Rng};
use raytracing::{
    boundingbox::BVHNode,
    camera::AntialiasingType,
    export::PngWriter,
    hittable::{HittableVec, Sphere},
    material::{Dielectric, DoubleLambertian, Lambertian, Metal},
    CameraBuilder, Color, Hittable, Material, Point3, Ray3, Vec3,
};
use std::{io, rc::Rc};

fn main() {
    let mut stdout = io::stdout().lock();
    let mut cam = CameraBuilder::new()
        .with_aspect_ratio(400, 16.0 / 9.0)
        .max_depth(50)
        .antialias(AntialiasingType::Square, 200)
        .camera_center(Point3::new(-2.0, 0.0, 0.0))
        .camera_target(Point3::origin())
        .vfov(90.0)
        .defocus_angle(0.0)
        .focal_length(10.0)
        .writer(PngWriter::new(&mut stdout).into_box())
        .build()
        .unwrap();

    let mut world = HittableVec::new();

    let ground_mat: &mut dyn Material =
        Box::leak(Box::new(Lambertian::new(Color::new(0.8, 0.8, 0.0))));
    let ground: Rc<dyn Hittable> = Rc::new(Sphere::stationary(
        Point3::new(0.0, -100.5, 0.0),
        100.0,
        ground_mat,
    ));
    world.add(ground);

    let obj_mat: &mut dyn Material = Box::leak(Box::new(DoubleLambertian::new(
        Color::hex(0xfaa),
        Color::hex(0x34d),
        0.0,
    )));
    let obj: Rc<dyn Hittable> =
        Rc::new(Sphere::stationary(Point3::new(0.0, 0.0, 0.0), 0.5, obj_mat));
    world.add(obj);

    cam.render(&world);
}
