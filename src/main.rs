use raytracing::{
    camera::AntialiasingType,
    export::PngWriter,
    hittable::{Disc, HittableVec, Parallelogram, Triangle},
    material::Lambertian,
    CameraBuilder, Color, Material as _, Point3, Texture as _, Vec3,
};
use std::{io, rc::Rc};

fn main() {
    let mut stdout = io::stdout().lock();
    let mut cam = CameraBuilder::new()
        .with_aspect_ratio(400, 1.0)
        .max_depth(50)
        .antialias(AntialiasingType::Square, 200)
        .camera_center(Point3::new(0.0, 0.0, 9.0))
        .camera_target(Point3::origin())
        .vfov(80.0)
        .defocus_angle(0.0)
        .writer(PngWriter::new(&mut stdout).into_box())
        .build()
        .unwrap();

    let mut world = HittableVec::new();

    let left_red =
        Lambertian::new(Color::new(1.0, 0.2, 0.2).solid_texture().into_texture()).into_mat();
    let back_green = Rc::new(Lambertian::new(
        Color::new(0.2, 1.0, 0.2).solid_texture().into_texture(),
    ));
    let right_blue = Rc::new(Lambertian::new(
        Color::new(0.2, 0.2, 1.0).solid_texture().into_texture(),
    ));
    let upper_orange = Rc::new(Lambertian::new(
        Color::new(1.0, 0.5, 0.0).solid_texture().into_texture(),
    ));
    let lower_teal = Rc::new(Lambertian::new(
        Color::new(0.2, 0.8, 0.8).solid_texture().into_texture(),
    ));

    world.add(Rc::new(Parallelogram::new(
        Point3::new(-3.0, -2.0, 5.0),
        Vec3::new(0.0, 0.0, -4.0),
        Vec3::new(0.0, 4.0, 0.0),
        left_red,
    )));
    world.add(Rc::new(Triangle::new(
        Point3::new(-2.0, -2.0, 0.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(2.0, 4.0, 0.0),
        right_blue.clone(),
    )));
    world.add(Rc::new(Triangle::from_points(
        Point3::new(-2.0, -2.0, -0.01),
        Point3::new(2.0, -2.0, -0.01),
        Point3::new(0.0, 2.0, -0.01),
        back_green,
    )));
    world.add(Rc::new(Parallelogram::new(
        Point3::new(3.0, -2.0, 1.0),
        Vec3::new(0.0, 0.0, 4.0),
        Vec3::new(0.0, 4.0, 0.0),
        right_blue,
    )));
    world.add(Rc::new(Parallelogram::new(
        Point3::new(-2.0, 3.01, 1.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 4.0),
        lower_teal.clone(),
    )));
    world.add(Rc::new(Disc::from_center(
        Point3::new(0.0, 3.0, 3.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 2.0),
        upper_orange.clone(),
    )));
    world.add(Rc::new(Disc::new(
        Point3::new(-2.0, -3.0, 5.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -4.0),
        lower_teal,
    )));

    cam.render(&world);
}
