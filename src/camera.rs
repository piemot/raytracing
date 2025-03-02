use indicatif::{ProgressBar, ProgressStyle};

use crate::{
    export::ImageWriter, vec::Normalized, Color, Hittable, Interval, Point3, Ray3, Vec2, Vec3,
};
use std::error::Error;

#[derive(Debug)]
#[must_use]
pub struct CameraBuilder<'a> {
    /// The width, in pixels, of the rendered image
    image_width: u32,
    /// The height, in pixels, of the rendered image.
    image_height: u32,
    /// Vertical view angle (field of view), in **radians**
    vfov: f64,
    /// How pixels are sampled during antialiasing.
    antialiasing_type: AntialiasingType,
    /// How many random samples are made per pixel during antialiasing.
    samples_per_px: u32,
    /// The maximum number of times a ray may bounce in a scene.
    max_depth: u32,
    /// The centre of the camera; where rays are shot from.
    camera_center: Point3,
    /// The point the camera is looking towards.
    /// The viewport plane does not necessarily intersect this point.
    /// The distance from [`Self::camera_center`] to the viewport plane is
    /// [`Self::focal_length`].
    camera_target: Point3,
    /// The "up" direction, relative to [`Self::camera_center`].
    vup: Vec3<Normalized>,
    /// The variation in angle of fired rays through each pixel, in **radians**.
    defocus_angle: f64,
    /// The distance from [`Self::camera_center`] to the plane of perfect focus.
    focal_length: f64,
    /// The [`ImageWriter`] used for writing the resulting image
    export_writer: Option<Box<dyn ImageWriter + 'a>>,

    errors: Vec<String>,
}

impl<'a> CameraBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    fn add_error(&mut self, condition: bool, err: String) {
        if condition {
            self.errors.push(format!("CameraBuilder::{err}"));
        }
    }

    pub fn dimensions(mut self, width: u32, height: u32) -> Self {
        self.add_error(
            width <= 1,
            format!("dimensions: Invalid width: must be greater than 1, found {width}"),
        );
        self.add_error(
            height <= 1,
            format!("dimensions: Invalid height: must be greater than 1, found {height}"),
        );

        self.image_width = width;
        self.image_height = height;
        self
    }

    pub fn with_aspect_ratio(mut self, width: u32, aspect_ratio: f64) -> Self {
        self.add_error(
            width <= 1,
            format!("with_aspect_ratio: Invalid width: must be greater than 1, found {width}"),
        );
        self.add_error(
            !(0.1..=100.0).contains(&aspect_ratio),
            format!("with_aspect_ratio: Invalid aspect_ratio: must be between 0.1 and 100.0, found {aspect_ratio}"),
        );

        self.image_width = width;
        let height = f64::from(width) / aspect_ratio;
        self.image_height = height.round() as u32;
        self
    }

    pub fn max_depth(mut self, depth: u32) -> Self {
        self.add_error(
            depth < 1,
            format!("max_depth: Invalid depth: must be at least 1, found {depth}"),
        );
        self.max_depth = depth;
        self
    }

    pub fn antialias(mut self, antialiasing_type: AntialiasingType, samples_per_px: u32) -> Self {
        self.add_error(
            samples_per_px < 1,
            format!(
                "antialias: Invalid samples_per_px: must be at least 1, found {samples_per_px}"
            ),
        );
        self.antialiasing_type = antialiasing_type;
        self.samples_per_px = samples_per_px;
        self
    }

    pub fn camera_center(mut self, center: Point3) -> Self {
        self.camera_center = center;
        self
    }

    pub fn focal_length(mut self, length: f64) -> Self {
        self.add_error(
            length <= 0.0,
            format!("focal_length: Invalid length: must be greater than 0.0, found {length}"),
        );
        self.focal_length = length;
        self
    }

    pub fn vfov(mut self, deg: f64) -> Self {
        self.add_error(
            !(0.01..360.0).contains(&deg),
            format!("vfov: Invalid deg: must be between 0.01 and 360.0, found {deg}"),
        );
        self.vfov = deg.to_radians();
        self
    }

    pub fn camera_target(mut self, target: Point3) -> Self {
        self.camera_target = target;
        self
    }

    pub fn vup(mut self, vec: Vec3) -> Self {
        self.vup = vec.as_unit();
        self
    }

    pub fn defocus_angle(mut self, angle: f64) -> Self {
        self.add_error(
            !(0.0..180.0).contains(&angle),
            format!("defocus_angle: Invalid angle: must be between 0.0 and 180.0, found {angle}"),
        );
        self.defocus_angle = angle.to_radians();
        self
    }

    pub fn writer(mut self, writer: Box<dyn ImageWriter + 'a>) -> Self {
        self.export_writer = Some(writer);
        self
    }

    pub fn build(mut self) -> Result<Camera<'a>, Vec<String>> {
        self.add_error(self.export_writer.is_none(),"build: Missing export format: include the `.writer()` parameter to specify the export format".to_string());

        if !self.errors.is_empty() {
            return Err(self.errors);
        }
        Ok(Camera::build(self))
    }
}

impl Default for CameraBuilder<'_> {
    fn default() -> Self {
        Self {
            image_width: 400,
            image_height: 200,
            vfov: 90.0_f64.to_radians(),
            antialiasing_type: AntialiasingType::Square,
            samples_per_px: 10,
            max_depth: 10,
            camera_center: Point3::origin(),
            camera_target: Point3::new(0.0, 0.0, -1.0),
            vup: Vec3::new(0.0, 1.0, 0.0).as_unit(),
            defocus_angle: 0.0_f64.to_radians(),
            focal_length: 1.0,
            export_writer: None,
            errors: Vec::new(),
        }
    }
}

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
pub struct Camera<'a> {
    /// The width, in pixels, of the rendered image
    image_width: u32,
    /// The height, in pixels, of the rendered image. Calculated based on `target_aspect_ratio` and `image_width`.
    image_height: u32,
    /// The centre of the camera; where rays are shot from.
    camera_center: Point3,
    /// The point (in 3d space) of the centre of the top-left pixel.
    pixel_00: Point3,
    /// A 3d vector pointing across the "top" of the viewport
    pxdelta_u: Vec3,
    /// A 3d vector pointing down the left "side" of the viewport
    pxdelta_v: Vec3,
    /// How pixels are sampled during antialiasing.
    antialiasing_type: AntialiasingType,
    /// How many random samples are made per pixel during antialiasing.
    samples_per_px: u32,
    /// A fraction (`0.0..=1.0`) to multiply each sample by for antialiasing.
    /// Should be equal to `1.0 / samples_per_px`.
    px_sample_scale: f64,
    /// The maximum number of times a ray may bounce in a scene.
    max_depth: u32,
    /// The variation in angle of fired rays through each pixel, in radians.
    defocus_angle: f64,
    /// A vector crossing half the width of the defocus disk.
    defocus_disk_u: Vec3,
    /// A vector crossing half the height of the defocus disk.
    defocus_disk_v: Vec3,
    /// The [`ImageWriter`] used for writing the resulting image
    export_writer: ImageWriterWrapper<'a>,
    // export_writer: Box<dyn ImageWriter>,
}

/// This Wrapper is used so that the ImageWriter can be borrowed mutably independently of the
/// rest of the Camera struct. This is necessary in [`Camera::render()`] where [`self.get_ray()`] is
/// called alongside [`self.write()`].
#[derive(Debug)]
struct ImageWriterWrapper<'a>(Box<dyn ImageWriter + 'a>);

// passthrough
impl ImageWriterWrapper<'_> {
    fn write_header(&mut self, width: u32, height: u32) -> Result<(), Box<dyn Error>> {
        self.0.write_header(width, height)
    }

    fn write(&mut self, colors: &[Color]) -> Result<(), Box<dyn Error>> {
        self.0.write(colors)
    }
}

impl<'a> Camera<'a> {
    fn build(builder: CameraBuilder<'a>) -> Self {
        // `builder` should be validated before being passed to this function
        assert!(builder.errors.is_empty());

        let CameraBuilder {
            image_width,
            image_height,
            camera_center,
            camera_target,
            vfov,
            vup,
            antialiasing_type,
            samples_per_px,
            max_depth,
            defocus_angle,
            focal_length,
            export_writer,
            errors: _,
        } = builder;

        let fwidth = f64::from(image_width);
        let fheight = f64::from(image_height);

        let aspect_ratio = fwidth / fheight;

        // |> Camera Variables <|

        // The height (in 3d space) of the viewport plane
        let viewport_height = 2.0 * f64::tan(vfov / 2.0) * focal_length;
        let viewport_width = viewport_height * aspect_ratio;

        // |> Viewport Calculations <|

        // Calculate the u, v, w unit basis vectors for the camera coordinate frame.
        let w = (camera_center - camera_target).as_unit();
        let u = vup.cross(&w.into()).as_unit();
        // cross product of unit vectors is a unit vector
        // TODO: add this as a specialiization in the [`Vec3::cross()`] definition
        let v = w.cross(&u.into()).assert_is_normalized();

        // A 3d vector pointing across the "top" of the viewport
        let viewport_u = viewport_width * u;
        // A 3d vector pointing down the left "side" of the viewport
        let viewport_v = viewport_height * -v;

        // Vectors describing the size of each pixel on the viewport
        let pxdelta_u = viewport_u / image_width;
        let pxdelta_v = viewport_v / image_height;

        // the top-left corner of the viewport, in 3d space.
        let viewport_corner =
            camera_center - (focal_length * w) - viewport_u / 2.0 - viewport_v / 2.0;

        // The point (in 3d space) of the centre of the top-left pixel.
        let pixel_00 = viewport_corner + (pxdelta_u + pxdelta_v) / 2.0;

        // The radius of the disk that rays are projected from.
        let defocus_radius = focal_length * f64::tan(defocus_angle / 2.0);
        let defocus_disk_u = u * defocus_radius;
        let defocus_disk_v = v * defocus_radius;

        // |> Antialiasing <|
        let px_sample_scale = 1.0 / f64::from(samples_per_px);

        Self {
            image_width,
            image_height,
            camera_center,
            pixel_00,
            pxdelta_u,
            pxdelta_v,
            antialiasing_type,
            samples_per_px,
            px_sample_scale,
            max_depth,
            defocus_angle,
            defocus_disk_u,
            defocus_disk_v,
            export_writer: ImageWriterWrapper(export_writer.unwrap()),
        }
    }

    pub fn render(&mut self, world: &impl Hittable) {
        let Self {
            ref image_width,
            ref image_height,
            ..
        } = self;

        let bar = ProgressBar::new((*image_height).into());
        let style = ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} rows ({per_sec}, {eta})").unwrap().progress_chars("=>-");
        bar.set_style(style);

        self.export_writer
            .write_header(*image_width, *image_height)
            .unwrap();

        let mut buf: Vec<Color> =
            Vec::with_capacity((self.image_height * self.image_width).try_into().unwrap());

        for j in 0..*image_height {
            for i in 0..*image_width {
                let mut px_color = Color::black();

                for _ in 0..self.samples_per_px {
                    let ray = self.get_ray(i, j);
                    px_color += Self::ray_color(&ray, self.max_depth, world);
                }

                px_color.set_brightness(self.px_sample_scale);
                buf.push(px_color);
            }
            bar.inc(1);
        }

        self.export_writer.write(&buf).unwrap();
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
            + (f64::from(i) + offset.x()) * self.pxdelta_u
            + (f64::from(j) + offset.y()) * self.pxdelta_v;

        let ray_origin = if self.defocus_angle <= 0.0 {
            self.camera_center
        } else {
            self.sample_defocus_disk()
        };

        let ray_direction = px_sample - ray_origin;
        Ray3::new(ray_origin, ray_direction)
    }

    fn ray_color(ray: &Ray3, depth: u32, world: &impl Hittable) -> Color {
        if depth == 0 {
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

    fn sample_defocus_disk(&self) -> Point3 {
        // returns a random point in the camera's defocus disc.
        let pt = Vec2::random_in_unit_circle();
        self.camera_center + pt.x() * self.defocus_disk_u + pt.y() * self.defocus_disk_v
    }
}
