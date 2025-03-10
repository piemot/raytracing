use png::Decoder;
use raytracing::{
    camera::AntialiasingType,
    export::PngWriter,
    hittable::{HittableVec, Sphere},
    material::Lambertian,
    texture::ImageTexture,
    CameraBuilder, Material, Point3, Texture,
};
use std::{fs::File, io, rc::Rc};

fn main() {
    let mut stdout = io::stdout().lock();
    let mut cam = CameraBuilder::new()
        .with_aspect_ratio(400, 16.0 / 9.0)
        .max_depth(50)
        .antialias(AntialiasingType::Square, 200)
        .camera_center(Point3::new(0.0, 0.0, 12.0))
        .camera_target(Point3::origin())
        .vfov(20.0)
        .defocus_angle(0.0)
        .writer(PngWriter::new(&mut stdout).into_box())
        .build()
        .unwrap();

    let mut world = HittableVec::new();

    let earth_img = Decoder::new(File::open("assets/textures/earth.png").unwrap());
    let earth_tex = ImageTexture::load(earth_img).into_texture();
    let earth_surface: Rc<dyn Material> = Rc::new(Lambertian::new(earth_tex));

    world.add(Rc::new(Sphere::stationary(
        Point3::new(0.0, 0.0, 0.0),
        2.0,
        earth_surface.clone(),
    )));

    cam.render(&world);
}
