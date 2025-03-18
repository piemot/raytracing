use raytracing::{
    camera::AntialiasingType,
    export::PngWriter,
    hittable::{HittableVec, Parallelogram, Sphere},
    material::{DiffuseLight, Lambertian},
    Background, CameraBuilder, Color, Material, Point3, Texture, Vec3,
};
use std::{io, rc::Rc};

fn main() {
    let mut stdout = io::stdout().lock();
    let mut cam = CameraBuilder::new()
        .with_aspect_ratio(400, 16.0 / 9.0)
        .max_depth(50)
        .antialias(AntialiasingType::Square, 1000)
        .camera_center(Point3::new(26.0, 3.0, 6.0))
        .camera_target(Point3::new(0.0, 2.0, 0.0))
        .background(Background::Constant(Color::black()))
        .vfov(20.0)
        .defocus_angle(0.0)
        .writer(PngWriter::new(&mut stdout).into_box())
        .build()
        .unwrap();

    let mut world = HittableVec::new();
    let ground_tex =
        Lambertian::new(Color::new(0.8, 0.8, 0.8).solid_texture().into_texture()).into_mat();
    let light_tex =
        DiffuseLight::new(Color::new(10.0, 3.0, 7.0).solid_texture().into_texture()).into_mat();

    world.add(Rc::new(Sphere::stationary(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        Rc::clone(&ground_tex),
    )));
    world.add(Rc::new(Sphere::stationary(
        Point3::new(0.0, 2.0, 0.0),
        2.0,
        Rc::clone(&ground_tex),
    )));

    world.add(Rc::new(Sphere::stationary(
        Point3::new(0.0, 7.0, 0.0),
        2.0,
        Rc::clone(&light_tex),
    )));
    world.add(Rc::new(Parallelogram::new(
        Point3::new(3.0, 1.0, -2.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(0.0, 2.0, 0.0),
        Rc::clone(&light_tex),
    )));

    cam.render(&world);
}
