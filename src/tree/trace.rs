use cgmath::{Vector3, Vector4};

use crate::{vector::{Vector}, read_bitrange, bitcheck};

use super::{octree::MAX_DEPTH, octant::Mask};

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Ray {
    pub origin: Vector3<f32>,
    pub dir: Vector3<f32>,
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct BranchInfo {
    pub node: u32,
    pub parent: u32,

    pub index: u32,
    pub parent_index: u32,

    pub span: f32,

    pub mask_info: u32,

    pub padding: [u32; 2],
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct PosInfo {
    pub local_pos: Vector4<f32>,   // Origin in CurNode
    pub pos_on_edge: Vector4<f32>, // Origin on first edge of CurNode

    pub depth: u32,

    pub padding: [u32; 3],
}

impl BranchInfo {
    pub fn idx(&self) -> usize {
        self.index as usize
    }

    pub fn parent_idx(&self) -> usize {
        self.parent_index as usize
    }

    pub fn child_bitmask(&self) -> u32 {
        read_bitrange!(self.node, 17, 24)
    }

    pub fn child_offset(&self) -> u32 {
        read_bitrange!(self.node, 1, 16)
    }

    pub fn parent_bitmask(&self) -> u32 {
        read_bitrange!(self.parent, 17, 24)
    }

    pub fn parent_child_offset(&self) -> u32 {
        read_bitrange!(self.parent, 1, 16)
    }

    fn mask_as_vec(mask: u32) -> Vector3<f32> {
        Vector3 {
            x: bitcheck!(mask, 1) as u32 as f32,
            y: bitcheck!(mask, 3) as u32 as f32,
            z: bitcheck!(mask, 2) as u32 as f32,
        }
    }
}

impl PosInfo {
    pub fn depth_idx(&self) -> usize {
        self.depth as usize
    }

    pub fn neighbor(
        &self,
        octant_data: &Vec<u32>,
        max_depth: usize,
        dir_mask: &Vector3<f32>,
    ) -> Option<PosInfo> {
        let mut pos_info = self.clone();

        for depth in self.depth as usize..max_depth {
            let new_mask = self.mask_info[depth].truncate() + dir_mask.clone();

            // Check if move up
            if new_mask.any(|num| num > 1.0 || num < 0.0) {
                pos_info.move_up(octant_data);
            } else {
                // Stop moving up and get next node
                let space_index = dir_mask.to_index(2.0);
                pos_info.index = self.parent(octant_data).children[space_index];

                // Start moving down
                while pos_info.octant(octant_data).has_children() {
                    pos_info.move_into_child(octant_data, |pos_info, space_idx| {
                        pos_info.parent(octant_data).children[space_idx]
                    });
                }

                return Some(pos_info);
            }
        }

        None
    }

    pub fn move_up(&mut self, branch_data: [BranchInfo; MAX_DEPTH]) {
        let branch = branch_data[self.depth_idx()].clone();

        let mask_vec = BranchInfo::mask_as_vec(branch.mask_info).extend(0.0);
        self.pos_on_edge -= mask_vec * branch.span;
        self.local_pos += mask_vec * branch.span;

        self.depth -= 1;
    }

    /// Expect child to be subdivide

    pub fn move_into_child<Function: FnOnce(&Self, usize) -> u32>(
        &mut self,
        octant_data: &Vec<u32>,
        branch_data: &mut [BranchInfo; MAX_DEPTH],
        select_idx: Function,
    ) {
        let old_branch = branch_data[self.depth_idx()].clone();

        self.depth += 1;

        let mut new_branch = BranchInfo {
            parent: old_branch.node,
            parent_index: old_branch.index,
            span: old_branch.span * 0.5,
            ..Default::default()
        };

        // Get which child node to choose
        new_branch.mask_info = self.local_pos.truncate().get_mask(new_branch.span);

        let mask_vec = BranchInfo::mask_as_vec(new_branch.mask_info).extend(0.0);
        self.pos_on_edge += mask_vec * new_branch.span;
        self.local_pos -= mask_vec * new_branch.span;

        new_branch.index = select_idx(self, new_branch.mask_info as usize);
    }
}

impl Default for Ray {
    fn default() -> Self {
        Self {
            origin: Vector3::default(),
            dir: Vector3::default(),
        }
    }
}

impl Default for BranchInfo {
    fn default() -> Self {
        Self {
            node: 0,
            parent: 0,

            index: 0,
            parent_index: 0,

            span: 0.0,

            mask_info: 0,

            padding: [0; 2],
        }
    }
}

impl Default for PosInfo {
    fn default() -> Self {
        Self {
            local_pos: Vector4::default(),
            pos_on_edge: Vector4::default(),

            depth: 0,

            padding: [0; 3],
        }
    }
}
