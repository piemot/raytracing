use std::io;

use crate::{color::write_color, Color, Hittable, Interval, Point3, Ray3, Vec2, Vec3};

#[derive(Debug)]
#[must_use]
pub struct Camera {
    /// The width, in pixels, of the rendered image
    image_width: u32,
    /// The **true** aspect ratio of the camera: should be a ratio of width over height.
    aspect_ratio: f64,
    /// The height, in pixels, of the rendered image. Calculated based on `target_aspect_ratio` and `image_width`.
    image_height: u32,
    /// The centre of the camera; where rays are shot from.
    camera_center: Point3,
    /// The point (in 3d space) of the centre of the top-left pixel.
    pixel_00: Point3,
    /// A vector pointing across the "top" of the viewport
    pxdelta_u: Vec2,
    /// A vector pointing down the left "side" of the viewport
    pxdelta_v: Vec2,
}

impl Camera {
    /// Create a new [`Camera`] that can render an image with a width of `width` pixels and with an approximate
    /// aspect ratio of `aspect_ratio`. The true `aspect_ratio` value will be as close as possible so that the
    /// height and width of the image are both integers.
    pub fn new(width: u32, aspect_ratio: f64) -> Self {
        let height = f64::from(width) / aspect_ratio;
        Self::new_dimensions(width, height.round() as u32)
    }

    /// Create a new [`Camera`] that can render an image with a width of `width` pixels and a height of `height` pixels.
    pub fn new_dimensions(width: u32, height: u32) -> Self {
        assert!(
            height > 0,
            "Cannot render image with height of {height} < 1 px."
        );

        let fwidth = f64::from(width);
        let fheight = f64::from(height);

        // This is the true aspect ratio, not the recommended one provided in [`Self::new()`]
        let aspect_ratio = fwidth / fheight;

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
        let pxdelta_u = viewport_u / width;
        let pxdelta_v = viewport_v / height;

        // the top-left corner of the viewport, in 3d space.
        // n.b: viewport_u and viewport_v are used to shift along the z axis,
        // so [`Vec3::from()`]'s assumption that z = 0.0 is correct
        let viewport_corner: Point3 = camera_center.shift_z(-focal_length)
            - Vec3::from(viewport_u / 2.0)
            - Vec3::from(viewport_v / 2.0);

        // The point (in 3d space) of the centre of the top-left pixel.
        let pixel_00: Point3 = viewport_corner + Vec3::from((pxdelta_u + pxdelta_v) / 2.0);

        Self {
            aspect_ratio,
            camera_center,
            image_height: height,
            image_width: width,
            pixel_00,
            pxdelta_u,
            pxdelta_v,
        }
    }

    pub fn render(&self, world: &impl Hittable) {
        let Self {
            image_width,
            image_height,
            ..
        } = self;

        println!("P3\n{image_width} {image_height}\n255");

        let mut stdout = io::stdout().lock();
        for j in 0..*image_height {
            for i in 0..*image_width {
                // px_center is offset in the 3d plane by 2d vectors i(Δu) and j(Δv).
                let px_center =
                    self.pixel_00 + Vec3::from(i * self.pxdelta_u) + Vec3::from(j * self.pxdelta_v);
                // A vector pointing from the camera to to the center of the pixel in 3d space.
                let ray_direction = px_center - self.camera_center;

                let ray = Ray3::new(self.camera_center, ray_direction);

                let px_color = Self::ray_color(&ray, world);

                write_color(&mut stdout, &px_color);
            }
        }
    }

    fn ray_color(ray: &Ray3, world: &impl Hittable) -> Color {
        if let Some(hit) = world.hit(ray, Interval::new(0.0, f64::INFINITY)) {
            let color_vec = 0.5 * (hit.normal() + Vec3::new(1.0, 1.0, 1.0));
            return Color::from_vec3(&color_vec);
        }

        // "sky" colouring
        let nd = ray.direction().as_unit();
        let intensity = (nd.y() + 1.0) * 0.5;

        let whiteness = Vec3::new(1.0, 1.0, 1.0) * (1.0 - intensity);
        let coloring = Vec3::new(0.5, 0.7, 1.0) * intensity;

        let color_vec = whiteness + coloring;
        Color::from_vec3(&color_vec)
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new_dimensions(100, 100)
    }
}
