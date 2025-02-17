use std::ops::{Deref, RangeInclusive};

#[derive(Debug, Clone)]
/// A custom struct extending Range, with some extra utilities.
pub struct Interval(RangeInclusive<f64>);

impl Interval {
    pub const fn new(start: f64, end: f64) -> Self {
        Self(start..=end)
    }

    pub const fn empty() -> Self {
        Self(f64::INFINITY..=f64::NEG_INFINITY)
    }

    pub const fn universe() -> Self {
        Self(f64::NEG_INFINITY..=f64::INFINITY)
    }

    pub fn size(&self) -> f64 {
        self.0.end() - self.0.start()
    }

    /// Returns `true` if `item` is contained in the range.
    pub fn contains(&self, item: f64) -> bool {
        self.0.contains(&item)
    }

    /// Returns `true` if `item` is contained in the range, but not equal to `self.start` or `self.end`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use raytracing::Interval;
    /// let inter = Interval::new(5.0, 10.0);
    /// assert!( inter.surrounds(5.1));
    /// assert!( inter.contains(5.1));
    ///
    /// assert!(!inter.surrounds(5.0));
    /// assert!( inter.contains(5.0));
    ///
    /// assert!( inter.surrounds(9.99999999));
    /// assert!( inter.contains(9.99999999));
    ///
    /// assert!(!inter.surrounds(10.0));
    /// assert!( inter.contains(10.0));
    ///
    /// assert!(!inter.surrounds(4.99));
    /// assert!(!inter.contains(4.99));
    /// assert!(!inter.surrounds(10.001));
    /// assert!(!inter.contains(10.001));
    /// ```
    pub fn surrounds(&self, item: f64) -> bool {
        *self.0.start() < item && item < *self.0.end()
    }

    /// Returns the provided number `item` if `item` is between `self.start` and `self.end`;
    /// otherwise, returns `self.start` or `self.end` if `item` is too low or too high, respectively.
    ///
    /// # Examples
    ///
    /// ```
    /// # use raytracing::Interval;
    /// let inter = Interval::new(5.0, 10.0);
    /// assert_eq!(inter.clamp(7.0), 7.0);
    /// assert_eq!(inter.clamp(4.0), 5.0);
    /// assert_eq!(inter.clamp(20.0), 10.0);
    /// ```
    pub fn clamp(&self, item: f64) -> f64 {
        if item < *self.0.start() {
            *self.0.start()
        } else if item > *self.0.end() {
            *self.0.end()
        } else {
            item
        }
    }
}

impl Deref for Interval {
    type Target = RangeInclusive<f64>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<RangeInclusive<f64>> for Interval {
    fn from(value: RangeInclusive<f64>) -> Self {
        Self(value)
    }
}

impl From<Interval> for RangeInclusive<f64> {
    fn from(value: Interval) -> Self {
        value.0
    }
}

#[cfg(test)]
mod test {
    use std::ops::RangeBounds;

    use super::*;
    #[test]
    fn deref() {
        assert_eq!(*Interval::new(0.0, 1.0).start(), 0.0);
        assert_eq!(*Interval::new(0.0, 1.0).end(), 1.0);
        assert_eq!(
            Interval::new(0.0, 1.0).end_bound(),
            std::ops::Bound::Included(&1.0)
        );
    }
}
