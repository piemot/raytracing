use std::{io::Read, rc::Rc};

use png::Decoder;

use crate::{Color, Point3};

pub trait Texture: std::fmt::Debug {
    fn value(&self, u: f64, v: f64, point: &Point3) -> Color;
    fn into_texture(self) -> Rc<dyn Texture>
    where
        Self: Sized + 'static,
    {
        Rc::new(self)
    }
}

#[derive(Debug)]
pub struct SolidColor(Color);

impl SolidColor {
    pub fn new(albedo: Color) -> Self {
        Self(albedo)
    }
}

impl Texture for SolidColor {
    fn value(&self, _u: f64, _v: f64, _point: &Point3) -> Color {
        self.0
    }
}

impl From<Color> for SolidColor {
    fn from(value: Color) -> Self {
        Self::new(value)
    }
}

#[derive(Debug)]
pub struct Checkerboard {
    scale: f64,
    even: Rc<dyn Texture>,
    odd: Rc<dyn Texture>,
}

impl Checkerboard {
    pub fn new(scale: f64, even: Rc<dyn Texture>, odd: Rc<dyn Texture>) -> Self {
        Self { scale, even, odd }
    }

    pub fn solid(scale: f64, even: SolidColor, odd: SolidColor) -> Self {
        Self {
            scale,
            even: Rc::new(even),
            odd: Rc::new(odd),
        }
    }
}

impl Texture for Checkerboard {
    fn value(&self, u: f64, v: f64, point: &Point3) -> Color {
        let x = f64::floor((1.0 / self.scale) * point.x());
        let y = f64::floor((1.0 / self.scale) * point.y());
        let z = f64::floor((1.0 / self.scale) * point.z());

        let is_even = (x as i32 + y as i32 + z as i32) % 2 == 0;

        match is_even {
            true => self.even.value(u, v, point),
            false => self.odd.value(u, v, point),
        }
    }
}

#[derive(Debug)]
pub struct ImageTexture {
    image_data: Vec<u8>,
    width: u32,
    height: u32,
}

impl ImageTexture {
    pub fn new(image_data: Vec<u8>, dimensions: (u32, u32)) -> Self {
        assert_eq!(
            image_data.len(),
            (dimensions.0 * dimensions.1 * 3) as usize,
            "sanity check; image_data.len() = # of pixels * 3 channels per pixel"
        );
        Self {
            image_data,
            width: dimensions.0,
            height: dimensions.1,
        }
    }

    pub fn load<R: Read>(mut decoder: Decoder<R>) -> Self {
        decoder.set_transformations(png::Transformations::normalize_to_color8());
        let mut reader = decoder.read_info().unwrap();

        assert!(
            reader.info().frame_control.is_none(),
            "Cannot accept APNGs."
        );
        assert!(
            matches!(reader.info().color_type, png::ColorType::Rgb),
            "Must be 8-bit PNG."
        );

        let mut buf = vec![0; reader.output_buffer_size()];
        reader.next_frame(&mut buf).unwrap();
        let info = reader.info();

        assert_eq!(
            buf.len(),
            usize::try_from(info.width * info.height * 3).unwrap()
        );

        Self {
            image_data: buf,
            width: info.width,
            height: info.height,
        }
    }
}

impl Texture for ImageTexture {
    fn value(&self, u: f64, v: f64, _point: &Point3) -> Color {
        assert!((0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&v));
        // Flip v to image coordinates
        let v = 1.0 - v;

        let i = (u * f64::from(self.width)) as u32;
        let j = (v * f64::from(self.height)) as u32;
        let ind = ((j * self.width + i) * 3) as usize;
        let [r, g, b] = &self.image_data[ind..ind + 3] else {
            panic!("Failed to deserialize texture")
        };

        Color::new_ints(*r, *g, *b)
    }
}
