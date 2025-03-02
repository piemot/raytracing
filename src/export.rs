use std::{error::Error, io::Write};

use crate::Color;

pub trait ImageWriter: std::fmt::Debug {
    fn write(&mut self, colors: &[Color]) -> Result<(), Box<dyn Error>>;
    fn write_header(&mut self, width: u32, height: u32) -> Result<(), Box<dyn Error>>;
}

pub struct PpmWriter<'a>(&'a mut dyn Write);

impl<'a> PpmWriter<'a> {
    pub fn new(output: &'a mut dyn Write) -> Self {
        Self(output)
    }

    pub fn into_box(self) -> Box<dyn ImageWriter + 'a> {
        Box::new(self)
    }
}

impl std::fmt::Debug for PpmWriter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("PpmWriter").finish()
    }
}

impl ImageWriter for PpmWriter<'_> {
    fn write_header(&mut self, width: u32, height: u32) -> Result<(), Box<dyn Error>> {
        writeln!(self.0, "P3\n{width} {height}\n255")?;
        Ok(())
    }

    fn write(&mut self, colors: &[Color]) -> Result<(), Box<dyn Error>> {
        for color in colors {
            let [r, g, b] = color.as_gamma_corrected().as_rgb_ints();
            writeln!(self.0, "{r} {g} {b}")?;
        }
        Ok(())
    }
}

pub enum PngWriter<'a> {
    Waiting(Option<&'a mut dyn Write>),
    Ready(png::Writer<&'a mut dyn Write>),
}

impl<'a> PngWriter<'a> {
    pub fn new(output: &'a mut dyn Write) -> Self {
        Self::Waiting(Some(output))
    }
    pub fn into_box(self) -> Box<dyn ImageWriter + 'a> {
        Box::new(self)
    }
}

impl std::fmt::Debug for PngWriter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("PngWriter").finish()
    }
}

impl ImageWriter for PngWriter<'_> {
    fn write_header(&mut self, width: u32, height: u32) -> Result<(), Box<dyn Error>> {
        if let PngWriter::Waiting(w) = self {
            let mut encoder = png::Encoder::new(std::mem::take(w).unwrap(), width, height);
            encoder.set_color(png::ColorType::Rgb);
            encoder.set_depth(png::BitDepth::Eight);
            let writer = encoder.write_header()?;
            *self = PngWriter::Ready(writer);
            Ok(())
        } else {
            panic!();
        }
    }

    fn write(&mut self, colors: &[Color]) -> Result<(), Box<dyn Error>> {
        if let PngWriter::Ready(w) = self {
            let mut buf: Vec<u8> = Vec::with_capacity(colors.len() * 3);
            buf.extend(
                colors
                    .into_iter()
                    .flat_map(|c| c.as_gamma_corrected().as_rgb_ints()),
            );
            w.write_image_data(&buf)?;
            Ok(())
        } else {
            panic!();
        }
    }
}
