use raytracing::{color::write_color, Color, Point3, Ray3, Vec2, Vec3};
use std::io;

fn main() {
    // Image variables
    let image_width = 400;
    let fimage_width = image_width as f64;

    // This is the target aspect ratio; it may not be completely accurate.
    let aspect_ratio = 16.0 / 9.0;

    let fimage_height = (fimage_width / aspect_ratio).round();
    let image_height = fimage_height as i32;

    // This is the *true* aspect ratio of the image.
    let aspect_ratio = fimage_width / fimage_height;

    if image_height <= 0 {
        panic!(
            "Cannot render image with height of {} (less than 1) px.",
            image_height
        );
    }

    // |> Camera Variables <|

    // The distance between the camera and the viewport plane
    let focal_length = 1.0;
    // The height (in 3d space) of the viewport plane
    let viewport_height = 2.0;
    let viewport_width = viewport_height * aspect_ratio;
    let camera_center = Point3::origin();

    // |> Viewport Calculations <|

    // A vector pointing across the "top" of the viewport
    let viewport_u = Vec2::new(viewport_width, 0.0);
    // A vector pointing down the left "side" of the viewport
    let viewport_v = Vec2::new(0.0, -viewport_height);

    // Vectors describing the size of each pixel on the viewport
    let pxdelta_u = viewport_u / image_width;
    let pxdelta_v = viewport_v / image_height;

    // the top-left corner of the viewport, in 3d space.
    // n.b: viewport_u and viewport_v are used to shift along the z axis,
    // so [`Vec3::from()`]'s assumption that z = 0.0 is correct
    let viewport_corner: Point3 = camera_center.shift_z(-focal_length)
        - Vec3::from(viewport_u / 2.0)
        - Vec3::from(viewport_v / 2.0);

    // The point (in 3d space) of the centre of the top-left pixel.
    let pixel_00: Point3 = viewport_corner + Vec3::from((pxdelta_u + pxdelta_v) / 2.0);

    // |> Render <|

    // header
    print!("P3\n{} {}\n255\n", image_width, image_height);

    let mut stdout = io::stdout().lock();
    for j in 0..image_height {
        for i in 0..image_width {
            // px_center is offset in the 3d plane by 2d vectors i(Δu) and j(Δv).
            let px_center = pixel_00 + Vec3::from(i * pxdelta_u) + Vec3::from(j * pxdelta_v);
            // A vector pointing from the camera to to the center of the pixel in 3d space.
            let ray_direction = px_center - camera_center;

            let ray = Ray3::new(camera_center, ray_direction);

            let px_color = ray_color(&ray);

            write_color(&mut stdout, &px_color);
        }
    }
}

fn ray_color(ray: &Ray3) -> Color {
    let nd = ray.direction().as_unit();
    let intensity = (nd.y() + 1.0) * 0.5;

    let whiteness = Vec3::new(1.0, 1.0, 1.0) * (1.0 - intensity);
    let coloring = Vec3::new(0.5, 0.7, 1.0) * intensity;

    Color::from_vec3(&(whiteness + coloring))
}
