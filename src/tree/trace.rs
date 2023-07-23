use cgmath::{Vector3, Vector4};

use crate::{mask_to_vec, read_bitrange, vec_to_mask, vector::Vector};

use super::{octant::Octant, octree::MAX_DEPTH};

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
        read_bitrange!(self.node, 16, 23)
    }

    pub fn child_offset(&self) -> u32 {
        read_bitrange!(self.node, 0, 15)
    }

    pub fn parent_bitmask(&self) -> u32 {
        read_bitrange!(self.parent, 16, 23)
    }

    pub fn parent_child_offset(&self) -> u32 {
        read_bitrange!(self.parent, 0, 15)
    }

    pub fn get_child(&self, octant_data: &Vec<u32>, child_mask: u32) -> (u32, u32) {
        let child_idx = self.parent_child_offset() + child_mask;
        log::info!("ci {}", child_idx);
        (child_idx, octant_data[child_idx as usize])
    }

    pub fn move_to_neighbor(&self, octant_data: &Vec<u32>, neighbor_mask: u32) -> BranchInfo {
        let mut branch = self.clone();
        // First get index of first child
        // After that, select neighbor based on mask
        branch.index = branch.index - branch.mask_info + neighbor_mask;
        branch.node = octant_data[branch.idx()];
        branch.mask_info = neighbor_mask;

        branch
    }
}

impl PosInfo {
    pub fn depth_idx(&self) -> usize {
        self.depth as usize
    }

    pub fn branch(&self, branch_data: &[BranchInfo; MAX_DEPTH]) -> BranchInfo {
        branch_data[self.depth_idx()]
    }

    /// Not tested

    pub fn neighbor(
        &self,
        octant_data: &Vec<u32>,
        branch_data: &mut [BranchInfo; MAX_DEPTH],
        max_depth: usize,
        dir_mask: u32,
    ) -> Option<PosInfo> {
        let mut pos_info = self.clone();

        for _ in self.depth as usize..max_depth {
            let mut branch = branch_data[self.depth_idx()];
            let new_mask = branch.mask_info | dir_mask;
            let move_up = branch.mask_info & dir_mask;

            // Check if move up
            if move_up > 0 {
                pos_info.move_up(&branch_data);
            } else {
                // Stop moving up and get next node
                branch = self
                    .branch(branch_data)
                    .move_to_neighbor(octant_data, new_mask);

                // Start moving down
                while branch.node.is_subdiv() {
                    pos_info.move_into_child(branch_data, |branch| {
                        let mut branch = branch.clone();

                        (branch.index, branch.node) =
                            branch.get_child(&octant_data, branch.mask_info);

                        branch
                    });
                }

                return Some(pos_info);
            }
        }

        None
    }

    pub fn move_up(&mut self, branch_data: &[BranchInfo; MAX_DEPTH]) {
        let branch = branch_data[self.depth_idx()].clone();

        let mask_vec = mask_to_vec!(branch.mask_info).extend(0.0);
        self.pos_on_edge -= mask_vec * branch.span;
        self.local_pos += mask_vec * branch.span;

        self.depth -= 1;
    }

    pub fn update_branch_to_child(
        &mut self,
        branch_data: &mut [BranchInfo; MAX_DEPTH],
    ) {
        let old_branch = branch_data[self.depth_idx()].clone();

        self.depth += 1;

        branch_data[self.depth_idx()] = BranchInfo {
            parent: old_branch.node,
            parent_index: old_branch.index,
            span: old_branch.span * 0.5,
            ..Default::default()
        };
    }

    /// Expect parent to be subdivide

    pub fn move_into_child<Function: FnOnce(&BranchInfo) -> BranchInfo>(
        &mut self,
        branch_data: &mut [BranchInfo; MAX_DEPTH],
        select_idx: Function,
    ) {
        self.update_branch_to_child(branch_data);
        let mut branch = self.branch(branch_data);

        // Get which child node to choose
        branch.mask_info =
            vec_to_mask!(self.local_pos.truncate() - Vector3::from([branch.span; 3]));

        self.pos_on_edge += mask_to_vec!(branch.mask_info).extend(0.0) * branch.span;
        self.local_pos -= mask_to_vec!(branch.mask_info).extend(0.0) * branch.span;

        branch = select_idx(&branch);
        branch_data[self.depth_idx()] = branch;
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
