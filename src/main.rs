use std::rc::Rc;

use raytracing::{
    camera::AntialiasingType,
    config::ConfigModel,
    export::PngWriter,
    hittable::{box3, Parallelogram, RotateY, Translate},
    material::{DiffuseLight, Lambertian},
    CameraBuilder, Color, Hittable, Material, Point3, Vec3,
};

fn main() {
    let mut stdout = std::io::stdout().lock();

    let mut cam = CameraBuilder::new()
        .with_aspect_ratio(600, 1.0)
        .max_depth(50)
        .antialias(AntialiasingType::Square, 20)
        .background(raytracing::Background::Constant(Color::black()))
        .camera_center(Point3::new(278.0, 278.0, -800.0))
        .camera_target(Point3::new(278.0, 278.0, 0.0))
        .vfov(40.0)
        .defocus_angle(0.0)
        .writer(PngWriter::new(&mut stdout).into_box())
        .build()
        .unwrap();

    let cbox: String = std::fs::read_to_string("cornell_box.toml").unwrap();
    let cfg: ConfigModel = cbox.parse().unwrap();
    let mut world = cfg.as_world();

    let white = Lambertian::solid(Color::white()).into_mat();
    let mut light_color = Color::white();
    light_color.set_brightness(20.0);
    let light = DiffuseLight::solid(light_color).into_mat();

    let lightbox = Parallelogram::new(
        Point3::new(343.0, 554.0, 332.0),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        Rc::clone(&light),
    )
    .hittable();
    /* let lightbox = Parallelogram::new(
        Point3::new(213.0, 554.0, 227.0),
        Vec3::new(130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 105.0),
        Rc::clone(&light),
    )
    .hittable(); */
    world.add(Rc::clone(&lightbox));

    let box1 = box3(
        &Point3::origin(),
        &Point3::new(165.0, 330.0, 165.0),
        Rc::clone(&white),
    );
    let box1 = RotateY::new(box1, 15.0_f64.to_radians()).hittable();
    let box1 = Translate::new(box1, Vec3::new(265.0, 0.0, 295.0)).hittable();
    world.add(box1);

    let box2 = box3(
        &Point3::origin(),
        &Point3::new(165.0, 165.0, 165.0),
        Rc::clone(&white),
    );
    let box2 = RotateY::new(box2, -18.0_f64.to_radians()).hittable();
    let box2 = Translate::new(box2, Vec3::new(130.0, 0.0, 65.0)).hittable();
    world.add(box2);

    cam.render(&world, lightbox);
}
