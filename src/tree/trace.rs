use cgmath::{Vector3, Vector4};

use crate::{vector::{Mask, Vector}};

use super::{octant::Octant};

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

    pub padding: [u32; 3],

    pub mask_info: Vector4<f32>,
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct PosInfo {
    pub local_pos: Vector4<f32>, // Origin in CurNode
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

    pub fn move_up(&mut self, octant_data: &Vec<Octant>) {
        let pos_mask = self.mask_info[self.depth_idx()];
        self.pos_on_edge -= pos_mask * self.span;
        self.local_pos += pos_mask * self.span;

        self.span *= 2.0;
        self.depth -= 1;

        // New index is parent
        self.index = self.octant(octant_data).parent;
    }

    /// Expect child to be subdivide

    pub fn move_into_child<Function: FnOnce(&Self, usize) -> u32>(
        &mut self,
        octant_data: &Vec<Octant>,
        select_idx: Function,
    ) {
        self.branch_info[self.depth_idx()] = self.octant(octant_data);

        self.span *= 0.5;
        self.depth += 1;

        // Get which child node to choose
        let child_mask = Octant::get_child_mask(self.span, self.local_pos.truncate());

        self.pos_on_edge += (child_mask * self.span).extend(0.0);
        self.local_pos -= (child_mask * self.span).extend(0.0);

        self.mask_info[self.depth_idx()] = (child_mask.clone()).extend(0.0);
        let space_index = child_mask.to_index(2.0);

        self.index = select_idx(self, space_index);
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
            padding: [0; 3],

            mask_info: Vector3::default(),
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
