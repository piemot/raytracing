use crate::{texture::SolidColor, Interval, Vec3};
use std::{fmt::Display, ops::AddAssign};

use super::vec::normal::NormalizationState;

/// Represents an RGB color with 3 floats, ranging from `(0.0, 0.0, 0.0)` (black) to `(1.0, 1.0, 1.0)` (white).
/// Invalid colors (colors outside the `0.0..=1.0` range) may be constructed; check [`Color::is_valid()`]
/// if a color must be valid.
/// Alternatively, the color can be clamped when converting to RGB with [`Color::as_rgb_ints()`].
#[derive(Debug, PartialEq, Clone, Copy)]
#[must_use]
pub struct Color {
    r: f64,
    g: f64,
    b: f64,
}

impl Color {
    /// Create a new Color with the given RGB values.
    /// ```
    /// use raytracing::Color;
    /// let cyan = Color::new(0.0, 1.0, 1.0);
    /// ```
    pub const fn new(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b }
    }

    /// Create a new Color with the given RGB values.
    /// ```
    /// use raytracing::Color;
    /// let cyan = Color::new(0, 255, 255);
    /// ```
    pub fn new_ints(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: f64::from(r) / 255.0,
            g: f64::from(g) / 255.0,
            b: f64::from(b) / 255.0,
        }
    }

    /// Create a new Color with the given hex value.
    /// ```
    /// use raytracing::Color;
    /// let cyan = Color::hex(0x0ff);
    /// ```
    pub fn hex(color: u32) -> Self {
        let (r, g, b) = if color <= 0xfff {
            let r = (color & 0xf00) >> 8;
            let g = (color << 4 & 0xf00) >> 8;
            let b = (color << 8 & 0xf00) >> 8;
            (r * 16, g * 16, b * 16)
        } else {
            let r = (color & 0xff0000) >> 16;
            let g = (color << 8 & 0xff0000) >> 16;
            let b = (color << 16 & 0xff0000) >> 16;
            (r, g, b)
        };

        Self::new(
            f64::from(r) / 255.0,
            f64::from(g) / 255.0,
            f64::from(b) / 255.0,
        )
    }

    /// Create a new Color repesenting black.
    pub const fn black() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        }
    }

    /// Create a new Color repesenting white.
    pub const fn white() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
        }
    }

    /// Create a new Color repesenting white, multiplied n times.
    /// This is useful for representing a very bright light.
    pub const fn over_white(n: f64) -> Self {
        Self {
            r: 1.0 * n,
            g: 1.0 * n,
            b: 1.0 * n,
        }
    }

    pub const fn red() -> Self {
        Self::new(1.0, 0.0, 0.0)
    }

    pub const fn green() -> Self {
        Self::new(0.0, 1.0, 0.0)
    }

    pub const fn blue() -> Self {
        Self::new(0.0, 0.0, 1.0)
    }

    #[cfg(debug_assertions)]
    pub const fn debug_magenta() -> Self {
        Self::new(1.0, 0.0, 1.0)
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

    /// Creates a color from a [`Vec3`], mapping `x` to `r`, `y` to `g`, and `z` to `b`.
    /// To create a valid color, each axis the [`Vec3`] should range from `0.0..=1.0`.
    /// This can most easily be accomplished by normalizing the vector. However,
    /// this function will accept vectors that produce invalid colors.
    pub fn from_vec3<T: NormalizationState>(vec: &Vec3<T>) -> Self {
        Self::new(vec.x(), vec.y(), vec.z())
    }

    /// Multplies all values by the supplied `brightness` value.
    pub fn set_brightness(&mut self, brightness: f64) {
        self.r *= brightness;
        self.g *= brightness;
        self.b *= brightness;
    }

    pub fn add(&self, rhs: &Color) -> Color {
        Color {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
        }
    }

    pub fn mul(&self, rhs: &Color) -> Color {
        Color {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
        }
    }

    pub fn solid_texture(self) -> SolidColor {
        SolidColor::new(self)
    }
}

// Add is intentionally _not implemented_; this is a utility designed for running sums.
impl AddAssign for Color {
    fn add_assign(&mut self, rhs: Self) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
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
    let [r, g, b] = color.as_gamma_corrected().as_rgb_ints();
    writeln!(out, "{r} {g} {b}").unwrap();
}
