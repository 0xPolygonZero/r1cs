use std::fmt;
use std::fmt::Formatter;
use std::cmp::Ordering;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Wire {
    pub index: u32,
}

impl Wire {
    pub const ONE: Wire = Wire { index: 0 };
}

impl Ord for Wire {
    fn cmp(&self, other: &Self) -> Ordering {
        // We want the 1 wire to be last. Otherwise use ascending index order.
        if *self == Wire::ONE && *other == Wire::ONE {
            Ordering::Equal
        } else if *self == Wire::ONE {
            Ordering::Greater
        } else if *other == Wire::ONE {
            Ordering::Less
        } else {
            self.index.cmp(&other.index)
        }
    }
}

impl PartialOrd for Wire {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Wire {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.index == 0 {
            write!(f, "1")
        } else {
            write!(f, "wire_{}", self.index)
        }
    }
}
