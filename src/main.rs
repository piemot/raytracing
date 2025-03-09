use rand::{random_range, Rng};
use raytracing::{
    boundingbox::BVHNode,
    camera::AntialiasingType,
    export::PngWriter,
    hittable::{HittableVec, Sphere},
    material::{Dielectric, Lambertian, Metal},
    texture::{Checkerboard, SolidColor},
    CameraBuilder, Color, Hittable, Material, Point3, Ray3, Texture, Vec3,
};
use std::{io, rc::Rc};

fn main() {
    let mut stdout = io::stdout().lock();
    let mut cam = CameraBuilder::new()
        .with_aspect_ratio(400, 16.0 / 9.0)
        .max_depth(50)
        .antialias(AntialiasingType::Square, 200)
        .camera_center(Point3::new(13.0, 2.0, 3.0))
        .camera_target(Point3::origin())
        .vfov(20.0)
        .defocus_angle(0.6)
        .focal_length(10.0)
        .writer(PngWriter::new(&mut stdout).into_box())
        .build()
        .unwrap();

    let mut world = HittableVec::new();

    let checker_tex = Checkerboard::solid(
        0.32,
        Color::new(0.2, 0.3, 0.1).into(),
        Color::new(0.9, 0.9, 0.9).into(),
    );

    let ground_mat: &mut dyn Material = Box::leak(Box::new(Lambertian::new(Rc::new(checker_tex))));
    let ground: Rc<dyn Hittable> = Rc::new(Sphere::stationary(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        ground_mat,
    ));
    world.add(ground);

    let mut rng = rand::rng();

    for a in -11..11 {
        for b in -11..11 {
            let (a, b) = (a as f64, b as f64);
            let center = Point3::new(
                a + 0.9 * rng.random::<f64>(),
                0.2,
                b + 0.9 * rng.random::<f64>(),
            );

            if (center - Point3::new(4.0, 0.2, 0.0)).len() < 0.9 {
                continue;
            }

            enum SphereMaterial {
                Lambertian,
                Metal,
                Dielectric,
            }

            let (mat_type, material): (_, Box<dyn Material>) = match rng.random() {
                0.00..0.80 => {
                    let albedo = Color::new(rng.random(), rng.random(), rng.random());
                    let texture: Rc<dyn Texture> = Rc::new(SolidColor::new(albedo));
                    (
                        SphereMaterial::Lambertian,
                        Box::new(Lambertian::new(texture)),
                    )
                }
                0.80..=0.95 => {
                    let albedo = Color::new(rng.random(), rng.random(), rng.random());
                    let fuzz = rng.random();
                    (
                        SphereMaterial::Metal,
                        Box::new(Metal::with_fuzz(albedo, fuzz)),
                    )
                }
                0.95..=1.00 => (SphereMaterial::Dielectric, Box::new(Dielectric::new(1.50))),
                _ => unreachable!(),
            };

            let mat = Box::leak(material);
            let sphere = match mat_type {
                SphereMaterial::Lambertian => {
                    let dir = Vec3::new(0.0, random_range(0.0..0.5), 0.0);
                    Sphere::new(Ray3::new(center, dir), 0.2, mat)
                }
                _ => Sphere::stationary(center, 0.2, mat),
            };
            let sphere = Rc::new(sphere);
            world.add(sphere);
        }
    }

    let bvh_world: Rc<dyn Hittable> = Rc::new(BVHNode::new(world.into()));

    world = [bvh_world].into_iter().collect();

    cam.render(&world);
}
