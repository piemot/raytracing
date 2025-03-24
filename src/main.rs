use raytracing::{
    camera::AntialiasingType,
    export::PngWriter,
    hittable::{box3, HittableVec, Parallelogram},
    material::{DiffuseLight, Lambertian},
    Background, CameraBuilder, Color, Material, Point3, Texture, Vec3,
};
use std::{io, rc::Rc};

fn main() {
    let mut stdout = io::stdout().lock();
    let mut cam = CameraBuilder::new()
        .with_aspect_ratio(600, 1.0)
        .max_depth(50)
        .antialias(AntialiasingType::Square, 500)
        .camera_center(Point3::new(278.0, 278.0, -800.0))
        .camera_target(Point3::new(278.0, 278.0, 0.0))
        .background(Background::Constant(Color::black()))
        .vfov(40.0)
        .defocus_angle(0.0)
        .writer(PngWriter::new(&mut stdout).into_box())
        .build()
        .unwrap();

    let mut world = HittableVec::new();

    let red =
        Lambertian::new(Color::new(0.65, 0.05, 0.05).solid_texture().into_texture()).into_mat();
    let white =
        Lambertian::new(Color::new(0.73, 0.73, 0.73).solid_texture().into_texture()).into_mat();
    let green =
        Lambertian::new(Color::new(0.12, 0.45, 0.15).solid_texture().into_texture()).into_mat();
    let light =
        DiffuseLight::new(Color::over_white(30.0).solid_texture().into_texture()).into_mat();

    world.add(Rc::new(Parallelogram::new(
        Point3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        Rc::clone(&green),
    )));
    world.add(Rc::new(Parallelogram::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        Rc::clone(&red),
    )));
    world.add(Rc::new(Parallelogram::new(
        Point3::new(343.0, 554.0, 332.0),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        Rc::clone(&light),
    )));
    world.add(Rc::new(Parallelogram::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        Rc::clone(&white),
    )));
    world.add(Rc::new(Parallelogram::new(
        Point3::new(555.0, 555.0, 555.0),
        Vec3::new(-555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -555.0),
        Rc::clone(&white),
    )));
    world.add(Rc::new(Parallelogram::new(
        Point3::new(0.0, 0.0, 555.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Rc::clone(&white),
    )));

    world.add(box3(
        &Point3::new(130.0, 0.0, 65.0),
        &Point3::new(295.0, 165.0, 230.0),
        Rc::clone(&white),
    ));
    world.add(box3(
        &Point3::new(265.0, 0.0, 295.0),
        &Point3::new(430.0, 330.0, 460.0),
        Rc::clone(&white),
    ));

    cam.render(&world);
}
