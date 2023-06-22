use crate::{read_bitrange, set_bit, bitcheck};

/// Bit 1 - 16 | Child offset
/// Bit 17 - 24 | Child bitmask
/// Bit 25 | Leaf?
/// Bit 26 | Subdivide?

pub trait Octant {
    fn set_subdiv(&self, subdiv: bool) -> Self;
    fn set_leaf(&self, leaf: bool) -> Self;
    fn has_children(&self) -> bool;
    fn is_subdiv(&self) -> bool;
    fn is_leaf(&self) -> bool;
}

impl Octant for u32 {
    fn set_leaf(&self, leaf: bool) -> Self {
        set_bit!(self, 25, leaf)
    }

    fn set_subdiv(&self, subdiv: bool) -> Self {
        set_bit!(self, 25, subdiv)
    }

    fn has_children(&self) -> bool {
        // Extract child bitmask bitrange from self
        // Check if no value = 1
        read_bitrange!(self, 17, 24) > 0
    }

    fn is_leaf(&self) -> bool {
        bitcheck!(self, 24)
    }

    fn is_subdiv(&self) -> bool {
        bitcheck!(self, 25)
    }
}
