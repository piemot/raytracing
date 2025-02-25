use crate::{color::write_color, Color, Hittable, Interval, Point3, Ray3, Vec2, Vec3};
use std::io;

#[derive(Debug)]
/// How pixels are sampled during antialiasing
pub enum AntialiasingType {
    /// Sample points from a `1px × 1px` square centred on the pixel's centre
    Square,
    /// Sample points from an `r = 0.5px` disc centred on the pixel's centre
    Disc,
}

#[derive(Debug)]
#[must_use]
pub struct Camera {
    /// The width, in pixels, of the rendered image
    image_width: u32,

    #[allow(dead_code)]
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
    /// How pixels are sampled during antialiasing.
    antialiasing_type: AntialiasingType,
    /// How many random samples are made per pixel during antialiasing.
    samples_per_px: u32,
    /// A fraction (`0.0..=1.0`) to multiply each sample by for antialiasing.
    /// Should be equal to `1.0 / samples_per_px`.
    px_sample_scale: f64,
    /// The maximum number of times a ray may bounce in a scene.
    max_depth: u32,
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

        let max_depth = 10;

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

        // |> Antialiasing <|

        let antialiasing_type = AntialiasingType::Square;
        let samples_per_px = 50;
        let px_sample_scale = 1.0 / f64::from(samples_per_px);

        Self {
            aspect_ratio,
            camera_center,
            image_height: height,
            image_width: width,
            pixel_00,
            pxdelta_u,
            pxdelta_v,
            antialiasing_type,
            samples_per_px,
            px_sample_scale,
            max_depth,
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
                let mut px_color = Color::black();

                for _ in 0..self.samples_per_px {
                    let ray = self.get_ray(i, j);
                    px_color += Self::ray_color(&ray, self.max_depth, world);
                }

                px_color.set_brightness(self.px_sample_scale);
                write_color(&mut stdout, &px_color);
            }
        }
    }

    /// Constructs a camera [`Ray3`] originating from the camera's `center` and directed at a
    /// randomly sampled point around the pixel location `(i, j)`.
    fn get_ray(&self, i: u32, j: u32) -> Ray3 {
        let offset = match self.antialiasing_type {
            AntialiasingType::Disc => Vec2::random_in_unit_circle() / 2,
            AntialiasingType::Square => Vec2::random_range(-0.5..0.5),
        };

        // px_sample is equal to the center of the pixel (offset in the 3d plane by 2d vectors i(Δu) and j(Δv))
        // plus the random vector of `offset`.
        let px_sample = self.pixel_00
            + Vec3::from((f64::from(i) + offset.x()) * self.pxdelta_u)
            + Vec3::from((f64::from(j) + offset.y()) * self.pxdelta_v);

        let ray_direction = px_sample - self.camera_center;
        Ray3::new(self.camera_center, ray_direction)
    }

    fn ray_color(ray: &Ray3, depth: u32, world: &impl Hittable) -> Color {
        if depth <= 0 {
            // Exceeded the bounce depth limit :(
            return Color::black();
        }

        if let Some(hit) = world.hit(ray, Interval::new(0.001, f64::INFINITY)) {
            if let Some(scatter) = hit.material().scatter(ray, &hit) {
                let bounce_color = Camera::ray_color(&scatter.scattered, depth - 1, world);
                return Color::mul(&scatter.attenuation, &bounce_color);
            }
            return Color::black();
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
