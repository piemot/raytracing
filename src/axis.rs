#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    pub fn iter() -> IterAxis {
        IterAxis(None)
    }
}

pub struct IterAxis(Option<Axis>);

impl Iterator for IterAxis {
    type Item = Axis;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            None => {
                self.0 = Some(Axis::X);
                Some(Axis::X)
            }
            Some(Axis::X) => {
                self.0 = Some(Axis::Y);
                Some(Axis::Y)
            }
            Some(Axis::Y) => {
                self.0 = Some(Axis::Z);
                Some(Axis::Z)
            }
            Some(Axis::Z) => None,
        }
    }
}

impl TryFrom<u8> for Axis {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::X),
            1 => Ok(Self::Y),
            2 => Ok(Self::Z),
            _ => Err(()),
        }
    }
}
