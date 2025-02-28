use rand::Rng;
use raytracing::{
    camera::AntialiasingType,
    hittable::{HittableVec, Sphere},
    material::{Dielectric, Lambertian, Metal},
    CameraBuilder, Color, Material, Point3,
};

fn main() {
    let cam = CameraBuilder::new()
        .with_aspect_ratio(400, 16.0 / 9.0)
        .max_depth(50)
        .antialias(AntialiasingType::Square, 200)
        .camera_center(Point3::new(13.0, 2.0, 3.0))
        .camera_target(Point3::origin())
        .vfov(20.0)
        .defocus_angle(0.6)
        .focal_length(10.0)
        .build()
        .unwrap();

    let mut world = HittableVec::new();

    let ground_mat = Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)));
    let ground = Sphere::new(Point3::new(0.0, -1000.0, 0.0), 1000.0, &*ground_mat);
    world.add(&ground);

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

            let material: Box<dyn Material> = match rng.random() {
                0.00..0.80 => {
                    let albedo = Color::new(rng.random(), rng.random(), rng.random());
                    Box::new(Lambertian::new(albedo))
                }
                0.80..=0.95 => {
                    let albedo = Color::new(rng.random(), rng.random(), rng.random());
                    let fuzz = rng.random();
                    Box::new(Metal::with_fuzz(albedo, fuzz))
                }
                0.95..=1.00 => Box::new(Dielectric::new(1.50)),
                _ => unreachable!(),
            };

            let mat = Box::leak(material);
            let sphere = Box::leak(Box::new(Sphere::new(center, 0.2, mat)));
            world.add(sphere);
        }
    }

    cam.render(&world);
}
