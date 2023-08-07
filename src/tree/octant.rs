use crate::{read_bitrange, set_bit, bitcheck, write_bitrange};

/// Bit 0 - 15 | first_child_idx
/// Bit 16 - 23 | Child bitmask
/// Bit 24 | Leaf?
/// Bit 25 | Subdivide?

pub trait Octant {
    fn set_subdiv(&self, subdiv: bool) -> Self;
    fn set_leaf(&self, leaf: bool) -> Self;
    fn has_children(&self) -> bool;
    fn is_leaf(&self) -> bool;
    fn is_subdiv(&self) -> bool;
    fn check_child_filled(&self, child_idx: u32) -> bool;
    fn set_child_filled(&self, child_idx: u32, filled: bool) -> Self;
    fn get_child_bitmask(&self) -> u32;
    fn get_first_child_idx(&self) -> Self;
    fn set_first_child_idx(&self, child_offset: u32) -> Self;
}

impl Octant for u32 {
    fn set_leaf(&self, leaf: bool) -> Self {
        set_bit!(self, 24, leaf)
    }

    fn set_subdiv(&self, subdiv: bool) -> Self {
        set_bit!(self, 25, subdiv)
    }

    fn has_children(&self) -> bool {
        // Extract child bitmask bitrange from self
        // Check if no value = 1
        read_bitrange!(self, 16, 23) > 0
    }

    fn is_leaf(&self) -> bool {
        bitcheck!(self, 24)
    }

    fn is_subdiv(&self) -> bool {
        bitcheck!(self, 25)
    }

    fn check_child_filled(&self, child_idx: u32) -> bool {
        bitcheck!(self, 16 + child_idx)
    }

    fn set_child_filled(&self, child_idx: u32, filled: bool) -> Self {
        set_bit!(self, 16 + child_idx, filled)
    }

    fn get_child_bitmask(&self) -> u32 {
        read_bitrange!(self, 16, 23)
    }

    fn get_first_child_idx(&self) -> Self {
        read_bitrange!(self, 0, 15)
    }

    fn set_first_child_idx(&self, child_offset: u32) -> Self {
        write_bitrange!(self, child_offset, 0, 15)
    }
}
