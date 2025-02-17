use crate::Interval;
use std::fmt::Display;

/// Represents an RGB color with 3 floats, ranging from `(0.0, 0.0, 0.0)` (black) to `(1.0, 1.0, 1.0)` (white).
/// Invalid colors (colors outside the `0.0..=1.0` range) may be constructed; check [`Color::is_valid()`]
/// if a color must be valid.
/// Alternatively, the color can be clamped when converting to RGB with [`Color::as_rgb_ints()`].
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Color {
    r: f64,
    g: f64,
    b: f64,
}

impl Color {
    /// Create a new Color with the given RGB values.
    /// ```
    /// use raytracing::Color;
    /// let green = Color::new(0.0, 1.0, 1.0);
    /// ```
    #[must_use]
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b }
    }

    /// Create a new Color repesenting black.
    #[must_use]
    pub const fn black() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        }
    }

    /// Create a new Color repesenting white.
    #[must_use]
    pub const fn white() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
        }
    }

    #[inline]
    #[must_use]
    pub fn r(&self) -> f64 {
        self.r
    }

    #[inline]
    #[must_use]
    pub fn g(&self) -> f64 {
        self.g
    }

    #[inline]
    #[must_use]
    pub fn b(&self) -> f64 {
        self.b
    }

    #[must_use]
    /// Returns this color as a gamma-corrected color.
    pub fn as_gamma_corrected(&self) -> Self {
        Self {
            r: linear_to_gamma(self.r),
            g: linear_to_gamma(self.g),
            b: linear_to_gamma(self.b),
        }
    }

    /// Returns this [`Color`] as an array of rounded integers.
    /// This may return values greater than 255; you probably want [`Color::as_rgb_ints()`].
    pub fn as_unclamped_rgb_ints(&self) -> [u32; 3] {
        let r = (self.r * 255.0) as u32;
        let g = (self.g * 255.0) as u32;
        let b = (self.b * 255.0) as u32;
        [r, g, b]
    }

    /// Returns this [`Color`] as an array of rounded and clamped integers, from 0 to 255.
    pub fn as_rgb_ints(&self) -> [u8; 3] {
        let intensity: Interval = (0.0..=1.0).into();

        let r = (intensity.clamp(self.r) * 255.0) as u8;
        let g = (intensity.clamp(self.g) * 255.0) as u8;
        let b = (intensity.clamp(self.b) * 255.0) as u8;
        [r, g, b]
    }

    /// Returns whether all properties of this [`Color`] are within the range [0.0, 1.0].
    pub fn is_valid(&self) -> bool {
        let inter: Interval = (0.0..=1.0).into();

        inter.contains(self.r) && inter.contains(self.g) && inter.contains(self.b)
    }
}

fn linear_to_gamma(linear_component: f64) -> f64 {
    if linear_component > 0.0 {
        linear_component.sqrt()
    } else {
        0.0
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_valid() {
            f.write_fmt(format_args!(
                "Color [#{:02x}{:02x}{:02x}]",
                (self.r * 255.0).round() as u8,
                (self.g * 255.0).round() as u8,
                (self.b * 255.0).round() as u8
            ))
        } else {
            f.debug_tuple("Color")
                .field(&self.r)
                .field(&self.g)
                .field(&self.b)
                .finish()
        }
    }
}

pub fn write_color(out: &mut impl std::io::Write, color: &Color) {
    let [r, g, b] = color.as_rgb_ints();
    write!(out, "{r} {g} {b}\n").unwrap();
}
