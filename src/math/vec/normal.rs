// Sealed trait pattern: https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed
mod private {
    use super::*;
    pub trait Sealed {}

    impl Sealed for Unknown {}
    impl Sealed for Normalized {}
}

#[doc(hidden)]
#[derive(Debug)]
pub struct Unknown {}

#[doc(hidden)]
#[derive(Debug)]
pub struct Normalized {}

#[doc(hidden)]
/// Sealed trait: cannot be implemented. This is used internally to represent a Vec3's normalization state.
pub trait NormalizationState: private::Sealed {}

impl NormalizationState for Unknown {}
impl NormalizationState for Normalized {}
