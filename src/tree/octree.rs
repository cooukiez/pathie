use cgmath::{Vector3};

use super::{
    octant::{Octant},
    trace::PosInfo,
};

pub const MAX_DEPTH: usize = 10;

pub struct Octree {
    // RootIndex = 0
    pub octant_data: Vec<u32>,
    pub root_span: f32,
}

impl Octree {
    pub fn node_at_pos(&self, pos: Vector3<f32>) -> PosInfo {
        let mut pos_info = PosInfo {
            span: self.root_span,

            local_pos: (pos % self.root_span).extend(0.0),
            pos_on_edge: (pos - (pos % self.root_span)).extend(0.0),

            ..Default::default()
        };

        for _ in 1..MAX_DEPTH {
            if pos_info.octant(&self.octant_data).has_children() {
                pos_info.move_into_child(&self.octant_data, |pos_info, space_idx| {
                    pos_info.parent(&self.octant_data).children[space_idx]
                });
            } else {
                break;
            }
        }

        pos_info
    }

    pub fn insert_node(&mut self, insert_pos: Vector3<f32>) -> PosInfo {
        let mut pos_info = PosInfo {
            span: self.root_span,

            local_pos: (insert_pos % self.root_span).extend(0.0),
            pos_on_edge: (insert_pos - (insert_pos % self.root_span)).extend(0.0),

            ..Default::default()
        };

        for _ in 1..MAX_DEPTH {
            pos_info.move_into_child(&self.octant_data.clone(), |pos_info, space_idx| {
                if pos_info.octant(&self.octant_data).children[space_idx] == 0 {
                    // Set Nodetype to be subdivide
                    self.octant_data[pos_info.index()].node_type = 1;

                    // Create child at specified pos -> space_index
                    self.octant_data[pos_info.index()].children[space_idx] =
                        self.octant_data.len() as u32;

                    // Add new child to octant data
                    self.octant_data.push(Octant::new(pos_info.index(), 0));
                }
                
                // Update basic children with bit shifting
                let parent_idx = pos_info.parent_idx(&self.octant_data);
                Octant::update_basic_children(&mut self.octant_data[parent_idx]);

                // Return new child / move down
                pos_info.octant(&self.octant_data).children[space_idx]
            });
        }

        pos_info
    }

    pub fn test_scene(&mut self) {
        // let fbm = Fbm::<Perlin>::new(0);
        let mut rng = rand::thread_rng(); 

        self.insert_node(
            Vector3::new(0.0, 0.0, 0.0),
        );

        self.insert_node(
            Vector3::new(10.0, 10.0, 10.0),
        );
    }
}

impl Default for Octree {
    fn default() -> Self {
        Self {
            octant_data: vec![Octant::default()],
            root_span: (1 << MAX_DEPTH) as f32,
        }
    }
}
