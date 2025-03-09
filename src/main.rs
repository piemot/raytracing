use raytracing::{
    camera::AntialiasingType,
    export::PngWriter,
    hittable::{HittableVec, Sphere},
    material::Lambertian,
    texture::Checkerboard,
    CameraBuilder, Color, Material, Point3, Texture,
};
use std::{io, rc::Rc};

fn main() {
    let mut stdout = io::stdout().lock();
    let mut cam = CameraBuilder::new()
        .with_aspect_ratio(400, 16.0 / 9.0)
        .max_depth(50)
        .antialias(AntialiasingType::Square, 200)
        .camera_center(Point3::new(45.0, -30.0, 3.0))
        .camera_target(Point3::origin())
        .vfov(20.0)
        .defocus_angle(0.0)
        .writer(PngWriter::new(&mut stdout).into_box())
        .build()
        .unwrap();

    let mut world = HittableVec::new();

    let checker = Checkerboard::solid(
        0.32,
        Color::new(0.2, 0.3, 0.1).into(),
        Color::new(0.9, 0.9, 0.9).into(),
    )
    .into_texture();

    let material: Rc<dyn Material> = Rc::new(Lambertian::new(checker));

    world.add(Rc::new(Sphere::stationary(
        Point3::new(0.0, -10.0, 0.0),
        10.0,
        material.clone(),
    )));

    world.add(Rc::new(Sphere::stationary(
        Point3::new(0.0, 10.0, 0.0),
        10.0,
        material.clone(),
    )));

    cam.render(&world);
}
