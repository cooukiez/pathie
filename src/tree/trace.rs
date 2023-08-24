use nalgebra_glm::Vec4;

use crate::{mask_to_vec, read_bitrange, vec_to_mask, vector::Vector};

use super::{octant::Octant, octree::MAX_DEPTH};

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Ray {
    pub origin: Vec4,
    pub dir: Vec4,
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct BranchInfo {
    pub parent_idx: u32,
    pub parent: u32,

    pub idx: u32,
    pub node: u32,

    pub span: f32,
    pub mask: u32,

    pub padding: [u32; 3],
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct PosInfo {
    pub local_pos: Vec4,   // Origin in CurNode
    pub pos_on_edge: Vec4, // Origin on first edge of CurNode

    pub depth: u32,

    pub padding: [u32; 3],
}

impl BranchInfo {
    pub fn idx(&self) -> usize {
        self.idx as usize
    }

    pub fn parent_idx(&self) -> usize {
        self.parent_idx as usize
    }

    pub fn first_child_idx(&self) -> u32 {
        read_bitrange!(self.parent, 0, 15)
    }

    pub fn get_child(&self, octant_data: &Vec<u32>, child_mask: u32) -> (u32, u32) {
        let child_idx = self.first_child_idx() + child_mask;
        (child_idx, octant_data[child_idx as usize])
    }

    pub fn move_to_neighbor(&self, octant_data: &Vec<u32>, neighbor_mask: u32) -> BranchInfo {
        let mut branch = self.clone();
        // First get index of first child
        // After that, select neighbor based on mask
        branch.idx = branch.idx - branch.mask + neighbor_mask;
        branch.node = octant_data[branch.idx()];
        branch.mask = neighbor_mask;

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
            let new_mask = branch.mask | dir_mask;
            let move_up = branch.mask & dir_mask;

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

                        (branch.idx, branch.node) =
                            branch.get_child(&octant_data, branch.mask);

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

        let mask_vec = mask_to_vec!(branch.mask);
        self.pos_on_edge -= mask_vec * branch.span;
        self.local_pos += mask_vec * branch.span;

        self.depth -= 1;
    }

    /// This function will auto update the depth and create new branch
    /// with parent, parent index, local pos, pos on edge and span auto set, the rest has to be done
    /// in the function update

    pub fn update_branch_to_child<Function: FnOnce(&BranchInfo) -> BranchInfo>(
        &mut self,
        branch_data: &mut [BranchInfo; MAX_DEPTH],
        update: Function,
    ) {
        let old_branch = branch_data[self.depth_idx()].clone();

        self.depth += 1;

        let mut branch = BranchInfo {
            parent: old_branch.node,
            parent_idx: old_branch.idx,
            span: old_branch.span * 0.5,
            ..Default::default()
        };

        branch = update(&branch);

        let mask_vec = mask_to_vec!(branch.mask);
        self.pos_on_edge -= mask_vec * branch.span;
        self.local_pos += mask_vec * branch.span;

        branch_data[self.depth_idx()] = branch;
    }

    /// Expect parent to be subdivide

    pub fn move_into_child<Function: FnOnce(BranchInfo) -> BranchInfo>(
        &mut self,
        branch_data: &mut [BranchInfo; MAX_DEPTH],
        select_idx: Function,
    ) {
        let local_pos = self.local_pos;

        self.update_branch_to_child(branch_data, |branch| {
            let mut branch = branch.clone();
            // Get which child node to choose
            branch.mask = vec_to_mask!(local_pos.step(Vec4::ftv(branch.span)));

            select_idx(branch)
        });
    }
}

impl Default for Ray {
    fn default() -> Self {
        Self {
            origin: Default::default(),
            dir: Default::default(),
        }
    }
}

impl Default for BranchInfo {
    fn default() -> Self {
        Self {
            parent_idx: Default::default(),
            parent: Default::default(),
            idx: Default::default(),
            node: Default::default(),
            span: Default::default(),
            mask: Default::default(),
            padding: Default::default(),
        }
    }
}

impl Default for PosInfo {
    fn default() -> Self {
        Self {
            local_pos: Default::default(),
            pos_on_edge: Default::default(),
            depth: Default::default(),
            padding: Default::default(),
        }
    }
}
