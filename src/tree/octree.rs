use cgmath::Vector3;

use super::{
    octant::Octant,
    trace::{BranchInfo, PosInfo},
};

pub const MAX_DEPTH: usize = 10;

pub struct Octree {
    // RootIndex = 0
    pub octant_data: Vec<u32>,
    pub root_span: f32,
}

impl Octree {
    pub fn node_at_pos(&self, pos: Vector3<f32>) -> PosInfo {
        let mut branch_data = [BranchInfo::default(); MAX_DEPTH];
        branch_data[0] = BranchInfo {
            node: self.octant_data[0],
            parent: self.octant_data[0],
            span: self.root_span,

            ..Default::default()
        };

        let mut pos_info = PosInfo {
            local_pos: (pos % self.root_span).extend(0.0),
            pos_on_edge: (pos - (pos % self.root_span)).extend(0.0),

            ..Default::default()
        };

        for _ in 1..MAX_DEPTH {
            if pos_info.branch(&branch_data).node.is_subdiv() {
                pos_info.move_into_child(&mut branch_data, |branch| {
                    let mut branch = branch.clone();

                    (branch.index, branch.node) =
                        branch.get_child(&self.octant_data, branch.mask_info);

                    branch
                });
            } else {
                break;
            }
        }

        pos_info
    }

    pub fn insert_node(&mut self, insert_pos: Vector3<f32>) -> PosInfo {
        let mut branch_data = [BranchInfo::default(); MAX_DEPTH];
        branch_data[0] = BranchInfo {
            node: self.octant_data[0],
            parent: self.octant_data[0],
            span: self.root_span,

            ..Default::default()
        };

        let mut pos_info = PosInfo {
            local_pos: (insert_pos % self.root_span).extend(0.0),
            pos_on_edge: (insert_pos - (insert_pos % self.root_span)).extend(0.0),

            ..Default::default()
        };

        for d in 1..MAX_DEPTH {
            pos_info.move_into_child(&mut branch_data, |branch| {
                let mut branch = branch.clone();

                if !branch.parent.is_subdiv() {
                    branch.parent = branch
                        .parent
                        // Set Nodetype to be subdivide
                        .set_subdiv(true)
                        // Set child offset, offset is index of first child
                        .set_child_offset(self.octant_data.len() as u32);

                    // Add new child to octant data
                    for _ in 0..8 {
                        self.octant_data.push(0);
                    }
                }

                log::info!("pi {} d {}, pco {}", branch.parent_idx(), d, branch.parent_child_offset());

                // Set child filled and update parent in octant data
                self.octant_data[branch.parent_idx()] =
                    branch.parent.set_child_filled(branch.mask_info, true);

                (branch.index, branch.node) = branch.get_child(&self.octant_data, branch.mask_info);

                branch
            });
        }

        self.octant_data[pos_info.branch(&branch_data).idx()] =
            self.octant_data[pos_info.branch(&branch_data).idx()].set_leaf(true);

        pos_info
    }

    pub fn test_scene(&mut self) {
        // let fbm = Fbm::<Perlin>::new(0);
        // let mut rng = rand::thread_rng();

        self.insert_node(Vector3::new(0.0, 0.0, 0.0));

        self.insert_node(Vector3::new(10.0, 10.0, 10.0));

        for nude in self.octant_data.clone() {
            log::info!(
                "leaf {} subdiv {} bitmask {:#09b} offset {}",
                nude.is_leaf(),
                nude.is_subdiv(),
                nude.get_child_bitmask(),
                nude.get_child_offset()
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
