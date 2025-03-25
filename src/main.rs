use raytracing::{
    camera::AntialiasingType,
    export::PngWriter,
    hittable::{box3, ConstantMedium, HittableVec, Parallelogram, RotateY, Translate},
    material::{DiffuseLight, Lambertian},
    Background, CameraBuilder, Color, Hittable, Material, Point3, Texture, Vec3,
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
        DiffuseLight::new(Color::over_white(15.0).solid_texture().into_texture()).into_mat();

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
        Point3::new(113.0, 554.0, 127.0),
        Vec3::new(330.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 305.0),
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

    let box1 = box3(
        &Point3::origin(),
        &Point3::new(165.0, 330.0, 165.0),
        Rc::clone(&white),
    );
    let box1 = RotateY::new(box1, 15.0f64.to_radians()).hittable();
    let box1 = Translate::new(box1, Vec3::new(265.0, 0.0, 295.0)).hittable();
    world.add(ConstantMedium::colored(box1, 0.01, Color::black()).hittable());

    let box2 = box3(
        &Point3::origin(),
        &Point3::new(165.0, 165.0, 165.0),
        Rc::clone(&white),
    );
    let box2 = RotateY::new(box2, -18.0f64.to_radians()).hittable();
    let box2 = Translate::new(box2, Vec3::new(130.0, 0.0, 65.0)).hittable();
    world.add(ConstantMedium::colored(box2, 0.01, Color::white()).hittable());

    cam.render(&world);
}
