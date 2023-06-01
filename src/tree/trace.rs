use cgmath::{Vector3, Vector4};

use crate::service::{Mask, Vector};

use super::{octant::{Subdivide, Leaf}, octree::MAX_DEPTH};

#[derive(Clone, Debug)]
pub struct Ray {
    pub origin: Vector3<f32>,
    pub dir: Vector3<f32>,
}

#[derive(Clone, Debug)]
pub struct PosInfo {
    pub branch_info: [Subdivide; MAX_DEPTH], // Store visited branch
    pub mask_info: [Vector4<f32>; MAX_DEPTH], // Position in parent at depth

    pub local_pos: Vector4<f32>,   // Origin in CurNode
    pub pos_on_edge: Vector4<f32>, // Origin on first edge of CurNode

    // Index positive -> Subdivide | Index negative -> Leaf
    pub index: i32,
    pub span: f32,
    pub depth: i32,
}

impl PosInfo {
    pub fn index(&self) -> usize {
        log::info!("{}", (self.index * 1));
        (self.index * self.index.signum()) as usize
    }

    pub fn depth_idx(&self) -> usize {
        self.depth as usize
    }

    pub fn as_leaf(&self, leaf_data: &Vec<Leaf>) -> Leaf {
        leaf_data[self.index()]
    }

    pub fn as_subdiv(&self, branch_data: &Vec<Subdivide>) -> Subdivide {
        branch_data[self.index()]
    }

    pub fn is_leaf(&self) -> bool {
        self.index < 0
    }

    pub fn is_subdiv(&self) -> bool {
        self.index > 0
    }    

    pub fn parent_idx(&self, branch_data: &Vec<Subdivide>, leaf_data: &Vec<Leaf>) -> i32 {
        if self.is_subdiv() {
            self.as_subdiv(branch_data).parent
        } else {
            self.as_leaf(leaf_data).parent
        }
    }

    pub fn parent(&self, branch_data: &Vec<Subdivide>, leaf_data: &Vec<Leaf>) -> Subdivide {
        branch_data[self.parent_idx(branch_data, leaf_data) as usize]
    }

    /// Function not tested

    pub fn neighbor(
        &self,
        branch_data: &Vec<Subdivide>,
        leaf_data: &Vec<Leaf>,
        max_depth: usize,
        dir_mask: &Vector3<f32>,
    ) -> Option<PosInfo> {
        let mut pos_info = self.clone();

        for depth in self.depth as usize..max_depth {
            let new_mask = self.mask_info[depth].truncate() + dir_mask.clone();

            // Check if move up
            if new_mask.any(|num| num > 1.0 || num < 0.0) {
                pos_info.move_up(branch_data, leaf_data);
            } else {
                // Stop moving up and get next node
                let space_index = dir_mask.to_index(2.0);
                pos_info.index = self.parent(branch_data, leaf_data).children[space_index];

                // Start moving down
                while pos_info.is_subdiv() {
                    pos_info.move_into_child(branch_data);
                }

                return Some(pos_info);
            }
        }

        None
    }

    pub fn move_up(&mut self, branch_data: &Vec<Subdivide>, leaf_data: &Vec<Leaf>) {
        let pos_mask = self.mask_info[self.depth_idx()];
        self.pos_on_edge -= pos_mask * self.span;
        self.local_pos += pos_mask * self.span;

        self.span *= 2.0;
        self.depth -= 1;

        // New index is parent
        self.index = self.parent_idx(branch_data, leaf_data);
    }

    /// Expect child to be subdivide

    pub fn move_into_child(&mut self, branch_data: &Vec<Subdivide>) {
        let test = self.as_subdiv(branch_data);
        self.branch_info[self.depth_idx()] = self.as_subdiv(branch_data);

        self.span *= 0.5;
        self.depth += 1;

        // Get which child node to choose
        let child_mask = Subdivide::get_child_mask(self.span, self.local_pos.truncate());

        self.pos_on_edge += (child_mask * self.span).extend(0.0);
        self.local_pos -= (child_mask * self.span).extend(0.0);

        self.mask_info[self.depth_idx()] = (child_mask.clone()).extend(0.0);
        let space_index = child_mask.to_index(2.0);

        // New Index is child node
        log::info!("{}", self.as_subdiv(branch_data).children[space_index]);

        self.index = self.branch_info[self.depth_idx()].children[space_index];
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

impl Default for PosInfo {
    fn default() -> Self {
        Self {
            branch_info: [Subdivide::default(); MAX_DEPTH],
            mask_info: [Vector4::default(); MAX_DEPTH],

            local_pos: Vector4::default(),
            pos_on_edge: Vector4::default(),

            index: -1,
            span: 0.0,
            depth: 0,
        }
    }
}
