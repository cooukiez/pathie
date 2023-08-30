use nalgebra_glm::{Vec2, Vec3, Vec4};

use crate::{mask_to_vec, vector::Vector};

use super::{
    octant::Octant,
    trace::{BranchInfo, PosInfo},
};

pub const MAX_DEPTH: usize = 8;
pub const MAX_DEPTH_LIMIT: usize = 16;
pub const TEXTURE_ALIGN: f32 = 16.0;

pub struct Octree {
    // RootIndex = 0
    pub octant_data: Vec<u32>,
    pub root_span: f32,
}

impl Octree {
    pub fn get_new_root_info(&self, pos: Vec4) -> ([BranchInfo; MAX_DEPTH], PosInfo) {
        let mut branch_data = [BranchInfo::default(); MAX_DEPTH];
        branch_data[0] = BranchInfo {
            node: self.octant_data[0],
            parent: self.octant_data[0],
            span: self.root_span,

            ..Default::default()
        };

        let local_pos = Vec4::new(
            pos.x % self.root_span,
            pos.y % self.root_span,
            pos.z % self.root_span,
            0.0,
        );

        let pos_info = PosInfo {
            local_pos,
            pos_on_edge: pos - local_pos,

            ..Default::default()
        };

        (branch_data, pos_info)
    }

    pub fn node_at_pos(&self, pos: Vec4) -> PosInfo {
        let (mut branch_data, mut pos_info) = self.get_new_root_info(pos);

        for _ in 1..MAX_DEPTH {
            if pos_info.branch(&branch_data).node.is_subdiv() {
                pos_info.move_into_child(&mut branch_data, |branch| {
                    let mut branch = branch.clone();

                    (branch.idx, branch.node) =
                        branch.get_child(&self.octant_data, branch.mask);

                    branch
                });
            } else {
                break;
            }
        }

        pos_info
    }

    pub fn insert_node(&mut self, insert_pos: Vec4) -> PosInfo {
        let (mut branch_data, mut pos_info) = self.get_new_root_info(insert_pos);

        for _ in 1..MAX_DEPTH {
            pos_info.move_into_child(&mut branch_data, |branch| {
                let mut branch = branch.clone();

                if !branch.parent.is_subdiv() {
                    branch.parent = branch
                        .parent
                        // Set Nodetype to be subdivide
                        .set_subdiv(true)
                        // Set child offset, offset is index of first child
                        .set_first_child_idx(self.octant_data.len() as u32);

                    // Add new child to octant data
                    for _ in 0..8 {
                        self.octant_data.push(0);
                    }
                }

                // Set child filled and update parent in octant data
                self.octant_data[branch.parent_idx()] =
                    branch.parent.set_child_filled(branch.mask, true);

                (branch.idx, branch.node) = branch.get_child(&self.octant_data, branch.mask);

                branch
            });
        }

        self.octant_data[pos_info.branch(&branch_data).idx()] =
            self.octant_data[pos_info.branch(&branch_data).idx()].set_leaf(true);

        pos_info
    }

    pub fn collect_branch(
        &self,
        branch_data: &[BranchInfo; MAX_DEPTH],
        pos_info: &PosInfo,
        leaf_data: &mut Vec<(PosInfo, [BranchInfo; MAX_DEPTH])>,
        max_depth: u32,
    ) -> [BranchInfo; MAX_DEPTH] {
        let mut branch_data = branch_data.clone();

        for idx in 0..8 {
            let mut pos_info = pos_info.clone();

            pos_info.update_branch_to_child(&mut branch_data, |branch| {
                let mut branch = branch.clone();
                branch.mask = idx;
                (branch.idx, branch.node) = branch.get_child(&self.octant_data, branch.mask);

                branch
            });

            let branch = pos_info.branch(&branch_data);
            
            if branch.node.is_subdiv() && pos_info.depth < max_depth - 1 {
                branch_data = self.collect_branch(&branch_data, &pos_info, leaf_data, max_depth);

            } else if branch.node.is_leaf() || branch.node.is_subdiv() {
                pos_info.move_up(&mut branch_data);

                leaf_data.push((pos_info, branch_data.clone()));
            }
        }

        branch_data
    }

    pub fn write_branch_to_texture(
        &self,
        branch_data: &[BranchInfo; MAX_DEPTH],
        pos_info: &PosInfo,
        img: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
        base_px: Vec2,
        pos_on_edge: Vec4,
        base_span: f32,
        max_depth: u32,
    ) -> [BranchInfo; MAX_DEPTH] {
        let mut branch_data = branch_data.clone();

        for idx in 0..8 {
            let mut pos_info = pos_info.clone();

            pos_info.update_branch_to_child(&mut branch_data, |branch| {
                let mut branch = branch.clone();
                branch.mask = idx;
                (branch.idx, branch.node) = branch.get_child(&self.octant_data, branch.mask);

                branch
            });

            let branch = pos_info.branch(&branch_data);

            if branch.node.is_subdiv() && pos_info.depth < max_depth - 1 {
                branch_data = self.write_branch_to_texture(
                    &branch_data,
                    &pos_info,
                    img,
                    base_px,
                    pos_on_edge,
                    base_span,
                    max_depth,
                );
            } else if branch.node.is_leaf() || branch.node.is_subdiv() {
                pos_info.move_up(&mut branch_data);

                let local_pos = pos_info.local_pos - pos_on_edge;
                let pos = base_px + Vec2::new(local_pos.x, local_pos.y + (local_pos.z * base_span));

                img.put_pixel(pos.x as u32, pos.y as u32, image::Rgba([255, 255, 255, 0]));
                // *img.get_pixel_mut(5, 5) = image::Rgb([255, 255, 255]);
            }
        }

        branch_data
    }

    pub fn test_scene(&mut self) {
        // let fbm = Fbm::<Perlin>::new(0);
        // let mut rng = rand::thread_rng();

        self.insert_node(Vec4::ftv(0.0));

        self.insert_node(Vec4::ftv(8.0));

        //self.insert_node(Vec4::ftv(4.0));

        //self.insert_node(Vec4::ftv(32.0));

        for nude in self.octant_data.clone() {
            log::info!(
                "leaf {} subdiv {} bitmask {:#09b} offset {}",
                nude.is_leaf(),
                nude.is_subdiv(),
                nude.get_child_bitmask(),
                nude.get_first_child_idx()
            );
        }
    }
}

impl Default for Octree {
    fn default() -> Self {
        Self {
            octant_data: vec![0],
            root_span: (1 << MAX_DEPTH) as f32,
        }
    }
}
