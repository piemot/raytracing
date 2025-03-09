use std::ops::{Deref, RangeInclusive};

#[derive(Debug, Clone, PartialEq)]
/// A custom struct extending Range, with some extra utilities.
pub struct Interval(RangeInclusive<f64>);

impl Interval {
    pub const fn new(start: f64, end: f64) -> Self {
        Self(start..=end)
    }

    /// Generates an interval with a positive difference (i.e. start <= end).
    ///
    /// # Examples
    /// ```
    /// use raytracing::Interval;
    ///
    /// let a = Interval::positive(10.0, 20.0);
    /// let b = Interval::positive(20.0, 10.0);
    /// assert_eq!(a, Interval::new(10.0, 20.0));
    /// assert_eq!(b, Interval::new(10.0, 20.0));
    /// ```
    pub fn positive(a: f64, b: f64) -> Self {
        if a <= b {
            Self::new(a, b)
        } else {
            Self::new(b, a)
        }
    }

    /// Given another interval, returns the overlapping inteval of the two.
    ///
    /// # Examples
    /// ```
    /// use raytracing::Interval;
    ///
    /// let a = Interval::positive(10.0, 20.0);
    /// let b = Interval::positive(15.0, 25.0);
    /// let c = Interval::positive(20.0, 25.0);
    /// assert_eq!(a.overlap(&b), Some(Interval::new(15.0, 20.0)));
    /// assert_eq!(b.overlap(&c), Some(Interval::new(20.0, 25.0)));
    /// assert_eq!(c.overlap(&a), None);
    /// ```
    pub fn overlap(&self, rhs: &Self) -> Option<Self> {
        let res = Self::new(
            f64::max(*self.start(), *rhs.start()),
            f64::min(*self.end(), *rhs.end()),
        );
        if res.size() <= 0.0 {
            None
        } else {
            Some(res)
        }
    }

    /// Constructs a new Interval surrounding both provided intervals.
    ///
    /// # Examples
    /// ```
    /// use raytracing::Interval;
    ///
    /// let a = Interval::positive(10.0, 20.0);
    /// let b = Interval::positive(-2.0, 4.0);
    /// assert_eq!(Interval::surrounding(&a, &b), Some(Interval::new(-2.0, 20.0)));
    /// ```
    pub fn surrounding(a: &Self, b: &Self) -> Self {
        Self::positive(
            f64::min(*a.start(), *b.start()),
            f64::max(*a.end(), *b.end()),
        )
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

    /// Creates a new [`Interval`] with a length `delta` greater than
    /// its currernt length. Each side is expanded by `delta / 2`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use raytracing::Interval;
    /// let inter = Interval::new(5.0, 10.0);
    /// let inter = inter.expand(1.0);
    /// assert_eq!(inter, Interval::new(4.5, 10.5))
    /// ```
    pub fn expand(&self, delta: f64) -> Self {
        Self::new(self.start() - delta / 2.0, self.end() + delta / 2.0)
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
