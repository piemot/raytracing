use crate::math::macros::{forward_ref_binop, forward_ref_unop};
use rand::distr::Distribution;
use std::{
    fmt::Display,
    marker::PhantomData,
    ops::{Add, AddAssign, Neg, Sub, SubAssign},
};

use super::{
    normal::{NormalizationState, Normalized, Unknown},
    Vec3,
};

/// Represents a vector in 2 dimensional space, with its origin at (0, 0).
/// Despite the similar naming, a [`Vec2`] is **very different** from a [`std::vec::Vec`].
/// A [`Vec2`] represents a vector in the mathematical sense; it can be
/// visualized as an arrow from (0, 0) to (x, y) in a 2d plane.
///
/// # Mathematical Functions
///
/// Vectors can be:
/// * added together to create another vector,
/// * negated (in which case `x` and `y` are both negated independenly),
/// * subtracted (which adds the first vector to an negated second vector),
/// * multiplied by a number, either integer or float (multiplying `x` and `y` by the provided number),
/// * or divided by a number (equivalent to multiplying by `1/n` where n is the number).
///
/// The dot product of two vectors can also be computed with [`Vec2::dot`].
/// Cross products are only defined in 3D and 7D spaces.
///
/// # Typestate Normalization
/// [`Vec2`]s store whether or not they are normalized at the type level:
///
/// ```
/// use raytracing::Vec2;
/// let vec = Vec2::new(1.0, 2.0);
/// assert!(!((vec.len() - 1.0).abs() < 0.000001));
/// let vec = vec.normalize();
/// // length is very close to 1.00
/// assert!((vec.len() - 1.0).abs() < 0.000001);
/// ```
///
/// This snippet will fail to compile, because [`Vec2::new`] returns a `Vec2<Unknown>`.
///
/// ```compile_fail
/// # use raytracing::{Vec2, math::vec::Normalized};
/// fn use_normalized(vec: &Vec2<Normalized>) {
///     // ...
/// }
/// use_normalized(&Vec2::new(1.0, 2.0)); // shouldn't compile!
/// ```
///
/// This will work, because [`Vec2::as_unit()`] returns a vector that is known to be normalized!
///
/// ```
/// # use raytracing::{Vec2, math::vec::Normalized};
/// fn use_normalized(vec: &Vec2<Normalized>) {
///     // ...
/// }
/// use_normalized(&Vec2::new(1.0, 2.0).as_unit());
/// ```
///
/// [`Vec2`]: crate::Vec2
///
#[derive(Debug, Default)]
pub struct Vec2<N: NormalizationState = Unknown> {
    x: f64,
    y: f64,
    normalized: PhantomData<N>,
}

// PartialEq does not depend on `normalized`
impl<N: NormalizationState> PartialEq for Vec2<N> {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

// Because of the use of PhantomData, manual implementaitons of Clone and Copy are necessary.
// See https://stackoverflow.com/questions/31371027/copy-trait-and-phantomdata-should-this-really-move
// and https://github.com/rust-lang/rust/issues/26925.
impl<T: NormalizationState> Clone for Vec2<T> {
    fn clone(&self) -> Vec2<T> {
        *self
    }
}

impl<T: NormalizationState> Copy for Vec2<T> {}

impl<T: NormalizationState> Vec2<T> {
    #[inline]
    pub fn x(&self) -> f64 {
        self.x
    }

    #[inline]
    pub fn y(&self) -> f64 {
        self.y
    }

    /// Returns the length of the vector, squared.
    /// This is required to calculate the true length of the vector.
    #[inline]
    pub fn len_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    /// Returns the length of the vector.
    pub fn len(&self) -> f64 {
        self.len_squared().sqrt()
    }

    /// Returns the dot product of two [`Vec2`]s.
    #[inline]
    pub fn dot(&self, rhs: &Vec2) -> f64 {
        self.x * rhs.x + self.y * rhs.y
    }

    /// Returns true if the vector is close to zero (within `1e-8`) in all dimensions.
    pub fn near_zero(&self) -> bool {
        const THRESHOLD: f64 = 1e-8;
        self.x.abs() < THRESHOLD && self.y.abs() < THRESHOLD
    }
}

impl Vec2<Unknown> {
    pub fn new(x: f64, y: f64) -> Vec2<Unknown> {
        Vec2 {
            x,
            y,
            normalized: PhantomData,
        }
    }

    /// Creates a vector pointing at its origin `(0, 0)`.
    /// Adding, subtracting, or negating with this vector will do nothing.
    pub const fn empty() -> Vec2 {
        Self {
            x: 0.0,
            y: 0.0,
            normalized: PhantomData,
        }
    }

    /// Create a new [`Vec2`], pointing in a random direction between
    /// `(0.0, 0.0)` and `(1.0, 1.0)`.
    ///
    /// Note that the distribution for each axis is uniform in the half-open interval of `[0.0, 1.0)`.
    /// See [`rand::distr::StandardUniform`] for more details.
    pub fn random() -> Vec2 {
        let mut rng = rand::rng();
        let distr = rand::distr::StandardUniform;
        let x = distr.sample(&mut rng);
        let y = distr.sample(&mut rng);

        Vec2::new(x, y)
    }

    /// Create a new Vec2, pointing in a random direction where each axis is randomly sampled
    /// from a [`Uniform`] distribution over the provided range.
    ///
    /// # Panics
    /// May panic if the `range` provided cannot be parsed into a [`rand::distr::Uniform<f64>`].
    ///
    /// [`Uniform`]: rand::distr::Uniform
    pub fn random_range(range: impl TryInto<rand::distr::Uniform<f64>>) -> Vec2 {
        let mut rng = rand::rng();
        // note: this isn't the cleanest way to make this generic over ranges but
        // other methods like `rand::distr::Uniform::try_from(range)` don't seem
        // to work due to vague errors about requiring `From<T>` impls.
        let distr: rand::distr::Uniform<f64> = range
            .try_into()
            .map_err(|_| "Invalid `range` provided to Vec2::random_range")
            .unwrap();
        let x = distr.sample(&mut rng);
        let y = distr.sample(&mut rng);

        Vec2::new(x, y)
    }

    /// Return a vector to a random point inside the unit circle - i.e. a circle with radius 1.0.
    pub fn random_in_unit_circle() -> Vec2 {
        loop {
            let p = Self::random_range(-1.0..=1.0);
            if p.len_squared() < 1.0 {
                return p;
            }
        }
    }

    /// Consumes this [`Vec2`] and produces a [`Vec2`] in the same direction, normalized to a length of `1.0`.
    pub fn normalize(self) -> Vec2<Normalized> {
        let r = self / self.len();
        Vec2::<Normalized> {
            x: r.x,
            y: r.y,
            normalized: PhantomData,
        }
    }

    /// Returns a [`Vec2`] pointing in the same direction, normalized a length of `1.0`.
    pub fn as_unit(&self) -> Vec2<Normalized> {
        self.normalize()
    }
}

impl Vec2<Normalized> {}

impl Neg for Vec2 {
    type Output = Vec2;

    fn neg(self) -> Self::Output {
        Vec2 {
            x: -self.x,
            y: -self.y,
            normalized: PhantomData,
        }
    }
}

forward_ref_unop! {impl Neg, neg for Vec2}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            normalized: PhantomData,
        }
    }
}

forward_ref_binop! {impl Add, add for Vec2, Vec2}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.add(rhs);
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        self.add(-rhs)
    }
}

forward_ref_binop! {impl Sub, sub for Vec2, Vec2}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.sub(rhs);
    }
}

// Implement Mul and Div by each core number type
macro_rules! impl_vec_scalar_ops {
    ($($ty:ident),*) => {
        use std::ops::{Mul, Div, MulAssign, DivAssign};
        $(
            impl Mul<$ty> for Vec2 {
                type Output = Vec2;
                fn mul(self, rhs: $ty) -> Self::Output {
                    let mul: f64 = rhs.into();
                    Vec2 {
                        x: self.x * mul,
                        y: self.y * mul,
                        normalized: PhantomData,
                    }
                }
            }

            impl Mul<Vec2> for $ty {
                type Output = Vec2;
                fn mul(self, rhs: Vec2) -> Self::Output {
                    rhs.mul(self)
                }
            }

            impl MulAssign<$ty> for Vec2 {
                fn mul_assign(&mut self, rhs: $ty) {
                    *self = self.mul(rhs);
                }
            }

            impl Div<$ty> for Vec2 {
                type Output = Vec2;
                fn div(self, rhs: $ty) -> Self::Output {
                    let divisor: f64 = rhs.into();
                    self.mul(1.0 / divisor)
                }
            }

            impl Div<Vec2> for $ty {
                type Output = Vec2;
                fn div(self, rhs: Vec2) -> Self::Output {
                    rhs.div(self)
                }
            }

            impl DivAssign<$ty> for Vec2 {
                fn div_assign(&mut self, rhs: $ty) {
                    *self = self.div(rhs);
                }
            }

            forward_ref_binop! {impl Mul, mul for Vec2, $ty}
            forward_ref_binop! {impl Mul, mul for $ty, Vec2}
            forward_ref_binop! {impl Div, div for Vec2, $ty}
            forward_ref_binop! {impl Div, div for $ty, Vec2}
        )*
    };
}

impl_vec_scalar_ops!(f32, f64, u8, u16, u32, i8, i16, i32);

impl Display for Vec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Vec2")
            .field(&self.x())
            .field(&self.y())
            .finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn normalization() {
        let vec = Vec2::new(1.0, 2.0);
        assert!(!((vec.len() - 1.0).abs() < 0.001));
        let vec = vec.normalize();
        // length is =~ 1.00
        assert!((vec.len() - 1.0).abs() < 0.001);
    }
}

impl From<Vec2> for Vec3 {
    fn from(value: Vec2) -> Self {
        Vec3::new(value.x(), value.y(), 0.0)
    }
}
