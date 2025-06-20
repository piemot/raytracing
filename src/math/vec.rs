use crate::{math::macros::forward_ref_binop, Axis};
use rand::distr::Distribution;
use std::{
    fmt::Display,
    marker::PhantomData,
    ops::{Add, AddAssign, Neg, Sub, SubAssign},
};

pub mod normal;
pub mod two_d;
use normal::NormalizationState;
pub use normal::{Normalized, Unknown};

/// Represents a vector in 3 dimensional space, with its origin at (0, 0, 0).
///
/// Despite the similar naming, a [`Vec3`] is **very different** from a [`std::vec::Vec`].
/// A [`Vec3`] represents a vector in the mathematical sense; it can be
/// visualized as an arrow from (0, 0, 0) to (x, y, z) in a 3d plane.
///
/// # Mathematical Functions
///
/// Vectors can be:
/// * added together to create another vector,
/// * negated (in which case `x`, `y`, and `z` are all negated independenly),
/// * subtracted (which adds the first vector to an negated second vector),
/// * multiplied by a number, either integer or float (multiplying `x`, `y`, and `z` by the provided number),
/// * or divided by a number (equivalent to multiplying by `1/n` where n is the number).
///
/// Dot and cross products between two vectors can also be computed with [`Vec3::dot`] and [`Vec3::cross`], respectively.
///
/// # Typestate Normalization
/// [`Vec3`]s store whether or not they are normalized at the type level:
///
/// ```
/// use raytracing::Vec3;
/// let vec = Vec3::new(1.0, 2.0, 3.0);
/// assert!(!((vec.len() - 1.0).abs() < 0.000001));
/// let vec = vec.normalize();
/// // length is very close to 1.00
/// assert!((vec.len() - 1.0).abs() < 0.000001);
/// ```
///
/// This snippet will fail to compile, because [`Vec3::new`] returns a `Vec3<Unknown>`.
///
/// ```compile_fail
/// # use raytracing::{Vec3, math::vec::Normalized};
/// fn use_normalized(vec: &Vec3<Normalized>) {
///     // ...
/// }
/// use_normalized(&Vec3::new(1.0, 2.0, 3.0)); // shouldn't compile!
/// ```
///
/// This will work, because [`Vec3::as_unit()`] returns a vector that is known to be normalized!
///
/// ```
/// # use raytracing::{Vec3, math::vec::Normalized};
/// fn use_normalized(vec: &Vec3<Normalized>) {
///     // ...
/// }
/// use_normalized(&Vec3::new(1.0, 2.0, 3.0).as_unit());
/// ```
///
/// [`Vec3`]: crate::Vec3
///
#[derive(Debug, Default)]
#[must_use]
pub struct Vec3<N: NormalizationState = Unknown> {
    x: f64,
    y: f64,
    z: f64,
    normalized: PhantomData<N>,
}

// PartialEq does not depend on `normalized`
impl<N: NormalizationState> PartialEq for Vec3<N> {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}

// Because of the use of PhantomData, manual implementaitons of Clone and Copy are necessary.
// See https://stackoverflow.com/questions/31371027/copy-trait-and-phantomdata-should-this-really-move
// and https://github.com/rust-lang/rust/issues/26925.
impl<T: NormalizationState> Clone for Vec3<T> {
    fn clone(&self) -> Vec3<T> {
        *self
    }
}

impl<T: NormalizationState> Copy for Vec3<T> {}

impl<T: NormalizationState> Vec3<T> {
    #[inline]
    pub fn x(&self) -> f64 {
        self.x
    }

    #[inline]
    pub fn y(&self) -> f64 {
        self.y
    }

    #[inline]
    pub fn z(&self) -> f64 {
        self.z
    }

    /// Returns the length of the vector, squared.
    /// This is required to calculate the true length of the vector.
    #[inline]
    pub fn len_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Returns the length of the vector.
    pub fn len(&self) -> f64 {
        self.len_squared().sqrt()
    }

    /// Returns the dot product of two [`Vec3`]s.
    #[inline]
    pub fn dot<A: NormalizationState>(&self, rhs: &Vec3<A>) -> f64 {
        self.x.mul_add(rhs.x, self.y.mul_add(rhs.y, self.z * rhs.z))
    }

    /// Returns true if the vector is close to zero (within `1e-8`) in all dimensions.
    pub fn near_zero(&self) -> bool {
        const THRESHOLD: f64 = 1e-8;
        self.x.abs() < THRESHOLD && self.y.abs() < THRESHOLD && self.z.abs() < THRESHOLD
    }
}

impl Vec3<Unknown> {
    pub fn new(x: f64, y: f64, z: f64) -> Vec3<Unknown> {
        Vec3 {
            x,
            y,
            z,
            normalized: PhantomData,
        }
    }

    /// Creates a vector pointing at its origin `(0, 0, 0)`.
    /// Adding, subtracting, or negating with this vector will do nothing.
    pub const fn empty() -> Vec3 {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            normalized: PhantomData,
        }
    }

    /// Reflects this vector with the normal vector of the surface.
    /// Assumes perfect specular reflection: `θ_in` = `θ_out`.
    pub fn reflect(&self, normal: &Vec3<Normalized>) -> Vec3 {
        let mul = 2.0 * Vec3::dot(self, normal);
        *self - normal * mul
    }

    /// Returns the cross product of two [`Vec3`]s.
    #[inline]
    pub fn cross(&self, rhs: &Vec3) -> Vec3 {
        Vec3 {
            x: self.y.mul_add(rhs.z, -(self.z * rhs.y)),
            y: self.z.mul_add(rhs.x, -(self.x * rhs.z)),
            z: self.x.mul_add(rhs.y, -(self.y * rhs.x)),
            normalized: PhantomData,
        }
    }

    /// Create a new Vec3, pointing in a random direction between
    /// `(0.0, 0.0, 0.0)` and `(1.0, 1.0, 1.0)`.
    ///
    /// Note that the distribution for each axis is uniform in the half-open interval of `[0.0, 1.0)`.
    /// See [`rand::distr::StandardUniform`] for more details.
    pub fn random() -> Vec3 {
        let mut rng = rand::rng();
        let distr = rand::distr::StandardUniform;
        let x = distr.sample(&mut rng);
        let y = distr.sample(&mut rng);
        let z = distr.sample(&mut rng);

        Vec3::new(x, y, z)
    }

    /// Create a new Vec3, pointing in a random direction where each axis is randomly sampled
    /// from a [`Uniform`] distribution over the provided range.
    ///
    /// # Panics
    /// May panic if the `range` provided cannot be parsed into a [`rand::distr::Uniform<f64>`].
    ///
    /// [`Uniform`]: rand::distr::Uniform
    pub fn random_range(range: impl TryInto<rand::distr::Uniform<f64>>) -> Vec3 {
        let mut rng = rand::rng();
        // note: this isn't the cleanest way to make this generic over ranges but
        // other methods like `rand::distr::Uniform::try_from(range)` don't seem
        // to work due to vague errors about requiring `From<T>` impls.
        let distr: rand::distr::Uniform<f64> = range
            .try_into()
            .map_err(|_| "Invalid `range` provided to Vec3::random_range")
            .unwrap();
        let x = distr.sample(&mut rng);
        let y = distr.sample(&mut rng);
        let z = distr.sample(&mut rng);

        Vec3::new(x, y, z)
    }

    /// Return a vector to a random point inside the unit sphere - i.e. a sphere with radius 1.0.
    pub fn random_in_unit_sphere() -> Vec3 {
        loop {
            let p = Self::random_range(-1.0..=1.0);
            let len_sq = p.len_squared();
            // Since floats have finite precision, there is a risk that len_sq is equal to zero.
            // This would result in an invalid vector when normalized,
            // so we skip if the length is too small.
            // If len_sq is greater than 1.0, it is inside the unit cube but outside the unit sphere.
            if 1.0e-160 < len_sq && len_sq < 1.0 {
                return p;
            }
        }
    }

    /// Return a unit [`Vec3`] to a random point on the edge of the unit sphere.
    #[inline]
    pub fn random_unit_vector() -> Vec3<Normalized> {
        Self::random_in_unit_sphere().as_unit()
    }

    /// Return a unit [`Vec3`] to a random point on the edge of the same hemisphere as the given normal vector.
    #[inline]
    pub fn random_on_hemisphere<T: NormalizationState>(normal: &Vec3<T>) -> Vec3<Normalized> {
        let on_unit_sphere = Self::random_unit_vector();
        if Vec3::dot(&on_unit_sphere, normal) > 0.0 {
            // same hemisphere as the normal
            on_unit_sphere
        } else {
            -on_unit_sphere
        }
    }

    /// Return a unit(?) [`Vec3`] to a random point on the edge of the unit sphere,
    /// weighted by cos(θ) where θ is the angle from the z-axis.
    #[inline]
    pub fn random_on_sphere_cosine() -> Vec3 {
        let r1: f64 = rand::random();
        let r2 = rand::random();

        let phi = std::f64::consts::TAU * r1;
        let x = f64::cos(phi) * f64::sqrt(r2);
        let y = f64::sin(phi) * f64::sqrt(r2);
        let z = f64::sqrt(1.0 - r2);

        Vec3::new(x, y, z)
    }

    /// Consumes this [`Vec3`] and produces a [`Vec3`] in the same direction, normalized to a length of `1.0`.
    pub fn normalize(self) -> Vec3<Normalized> {
        let r = self / self.len();
        Vec3::<Normalized> {
            x: r.x,
            y: r.y,
            z: r.z,
            normalized: PhantomData,
        }
    }

    /// Returns a [`Vec3`] pointing in the same direction, normalized a length of `1.0`.
    pub fn as_unit(&self) -> Vec3<Normalized> {
        self.normalize()
    }

    /// Asserts that the current Vec3 is normalized, and returns a Vec3 marked as Normalized.
    /// This function is NOT CHECKED. It should be used for performance-critical applications.
    pub fn assert_is_normalized(self) -> Vec3<Normalized> {
        Vec3::<Normalized> {
            x: self.x,
            y: self.y,
            z: self.z,
            normalized: PhantomData,
        }
    }
}

impl Vec3<Normalized> {
    /// Refracts this normalized vector, given the normal vector of the surface and
    /// the ratio of refractive indices [η / η′].
    pub fn refract(&self, normal: &Vec3<Normalized>, index_ratio: f64) -> Vec3 {
        let cos_theta = (-self).dot(normal).min(1.0);
        let rayout_perpendicular = index_ratio * (self + (cos_theta * normal));
        let rayout_parallel = -(1.0 - rayout_perpendicular.len_squared()).abs().sqrt() * normal;

        rayout_perpendicular + rayout_parallel
    }

    /// Returns the cross product of two normalized [`Vec3`]s, which is itself a normalized vector.
    #[inline]
    pub fn cross(&self, rhs: &Vec3<Normalized>) -> Vec3<Normalized> {
        Vec3 {
            x: self.y.mul_add(rhs.z, -(self.z * rhs.y)),
            y: self.z.mul_add(rhs.x, -(self.x * rhs.z)),
            z: self.x.mul_add(rhs.y, -(self.y * rhs.x)),
            normalized: PhantomData,
        }
    }
}

impl<T: NormalizationState> std::ops::Index<Axis> for Vec3<T> {
    type Output = f64;

    fn index(&self, index: Axis) -> &Self::Output {
        match index {
            Axis::X => &self.x,
            Axis::Y => &self.y,
            Axis::Z => &self.z,
        }
    }
}

// Vectors can be negated keeping their normalization states.
impl<T: NormalizationState> Neg for Vec3<T> {
    type Output = Vec3<T>;

    fn neg(self) -> Self::Output {
        Vec3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            normalized: PhantomData,
        }
    }
}

impl<T: NormalizationState> Neg for &Vec3<T> {
    type Output = <Vec3<T> as Neg>::Output;
    #[inline]
    fn neg(self) -> <Vec3<T> as Neg>::Output {
        Neg::neg(*self)
    }
}

// However, they lose their normalization state when added or subtracted.
impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            normalized: PhantomData,
        }
    }
}

impl Add<Vec3<Normalized>> for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Vec3<Normalized>) -> Vec3 {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            normalized: PhantomData,
        }
    }
}

impl Add<Vec3> for Vec3<Normalized> {
    type Output = Vec3;

    fn add(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            normalized: PhantomData,
        }
    }
}

// adding two normalized vectors does not result in a normalized vector
impl Add<Vec3<Normalized>> for Vec3<Normalized> {
    type Output = Vec3;

    fn add(self, rhs: Vec3<Normalized>) -> Vec3 {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            normalized: PhantomData,
        }
    }
}

forward_ref_binop! {impl Add, add for Vec3, Vec3}
forward_ref_binop! {impl Add, add for Vec3<Normalized>, Vec3}
forward_ref_binop! {impl Add, add for Vec3, Vec3<Normalized>}
forward_ref_binop! {impl Add, add for Vec3<Normalized>, Vec3<Normalized>}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.add(rhs);
    }
}

impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Self) -> Self::Output {
        self.add(-rhs)
    }
}

impl Sub<Vec3> for Vec3<Normalized> {
    type Output = Vec3;

    fn sub(self, rhs: Vec3) -> Vec3 {
        self.add(-rhs)
    }
}

impl Sub<Vec3<Normalized>> for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Vec3<Normalized>) -> Vec3 {
        self.add(-rhs)
    }
}

impl Sub<Vec3<Normalized>> for Vec3<Normalized> {
    type Output = Vec3;

    fn sub(self, rhs: Vec3<Normalized>) -> Vec3 {
        self.add(-rhs)
    }
}

forward_ref_binop! {impl Sub, sub for Vec3, Vec3}
forward_ref_binop! {impl Sub, sub for Vec3<Normalized>, Vec3}
forward_ref_binop! {impl Sub, sub for Vec3, Vec3<Normalized>}
forward_ref_binop! {impl Sub, sub for Vec3<Normalized>, Vec3<Normalized>}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.sub(rhs);
    }
}

// Implement Mul and Div by each core number type
macro_rules! impl_vec_scalar_ops {
    ($($ty:ident),*) => {
        use std::ops::{Mul, Div, MulAssign, DivAssign};
        $(
            impl<T: NormalizationState> Mul<$ty> for Vec3<T> {
                type Output = Vec3;
                fn mul(self, rhs: $ty) -> Self::Output {
                    let mul: f64 = rhs.into();
                    Vec3 {
                        x: self.x * mul,
                        y: self.y * mul,
                        z: self.z * mul,
                        normalized: PhantomData,
                    }
                }
            }

            impl<T: NormalizationState> Mul<Vec3<T>> for $ty {
                type Output = Vec3;
                fn mul(self, rhs: Vec3<T>) -> Self::Output {
                    rhs.mul(self)
                }
            }

            // mulAssign cannot be implemented for Vec3<Normalized>
            // because it would require changing the type.
            impl MulAssign<$ty> for Vec3 {
                fn mul_assign(&mut self, rhs: $ty) {
                    *self = self.mul(rhs);
                }
            }

            impl<T: NormalizationState> Div<$ty> for Vec3<T> {
                type Output = Vec3;
                fn div(self, rhs: $ty) -> Self::Output {
                    let divisor: f64 = rhs.into();
                    self.mul(1.0 / divisor)
                }
            }

            impl<T: NormalizationState> Div<Vec3<T>> for $ty {
                type Output = Vec3;
                fn div(self, rhs: Vec3<T>) -> Self::Output {
                    rhs.div(self)
                }
            }

            impl DivAssign<$ty> for Vec3 {
                fn div_assign(&mut self, rhs: $ty) {
                    *self = self.div(rhs);
                }
            }

            forward_ref_binop! {impl Mul, mul for Vec3, $ty}
            forward_ref_binop! {impl Mul, mul for $ty, Vec3}
            forward_ref_binop! {impl Mul, mul for Vec3<Normalized>, $ty}
            forward_ref_binop! {impl Mul, mul for $ty, Vec3<Normalized>}
            forward_ref_binop! {impl Div, div for Vec3, $ty}
            forward_ref_binop! {impl Div, div for $ty, Vec3}
            forward_ref_binop! {impl Div, div for Vec3<Normalized>, $ty}
            forward_ref_binop! {impl Div, div for $ty, Vec3<Normalized>}
        )*
    };
}

impl_vec_scalar_ops!(f32, f64, u8, u16, u32, i8, i16, i32);

impl Display for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Vec3")
            .field(&self.x())
            .field(&self.y())
            .field(&self.z())
            .finish()
    }
}

impl From<Vec3<Normalized>> for Vec3 {
    fn from(value: Vec3<Normalized>) -> Self {
        Vec3 {
            x: value.x,
            y: value.y,
            z: value.z,
            normalized: PhantomData,
        }
    }
}

impl From<Vec3> for Vec3<Normalized> {
    fn from(value: Vec3) -> Self {
        Vec3 {
            x: value.x,
            y: value.y,
            z: value.z,
            normalized: PhantomData,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn normalization() {
        let vec = Vec3::new(1.0, 2.0, 3.0);
        assert!(!((vec.len() - 1.0).abs() < 0.001));
        let vec = vec.normalize();
        // length is =~ 1.00
        assert!((vec.len() - 1.0).abs() < 0.001);
    }
}
