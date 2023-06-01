use cgmath::{Vector3, Vector4};
use noise::{Fbm, NoiseFn, Perlin};
use rand::Rng;

use super::{octant::{Octant, Material}, trace::PosInfo};

pub const MAX_DEPTH: usize = 10;
pub const MAX_NODE: usize = 8192;



pub struct Octree {
    // RootIndex = 0
    pub octant_data: Vec<Octant>,
    pub root_span: f32,
}

impl Octree {
    /// Create children for an existing node.
    /// First add the material effect stuff divided by span to the
    /// material of the node to take LOD effect into
    /// account. If node has no children, create them.

    pub fn create_children(&mut self, pos_info: &PosInfo) {
        if pos_info.is_leaf() {
            self.leaf_data.remove(pos_info.index());
            let mut subdiv = Subdivide::default();

            for index in 0..8 {
                log::info!("new leaf index {}", -(self.leaf_data.len() as i32));
                subdiv.children[index] = -(self.leaf_data.len() as i32);
                self.leaf_data.push(Leaf::new(pos_info.index()));
            }

            log::info!("new children {:?}", subdiv.children);

            self.branch_data.push(subdiv);
        }
    }

    pub fn node_at_pos(&self, pos: Vector3<f32>) -> PosInfo {
        let mut pos_info = PosInfo {
            span: self.root_span,

            local_pos: (pos % self.root_span).extend(0.0),
            pos_on_edge: (pos - (pos % self.root_span)).extend(0.0),

            ..Default::default()
        };

        for _ in 1..MAX_DEPTH {
            if pos_info.is_subdiv() {
                pos_info.move_into_child(&self.branch_data);
            } else {
                break;
            }
        }

        pos_info
    }

    pub fn insert_node(&mut self, insert_pos: Vector3<f32>, base_color: Vector4<f32>) -> PosInfo {
        let mut pos_info = PosInfo {
            span: self.root_span,

            local_pos: (insert_pos % self.root_span).extend(0.0),
            pos_on_edge: (insert_pos - (insert_pos % self.root_span)).extend(0.0),

            ..Default::default()
        };

        for _ in 1..MAX_DEPTH {
            self.create_children(&pos_info);
            pos_info.move_into_child(&self.branch_data);
        }

        self.leaf_data[pos_info.index()].set(&Material { base_color });

        pos_info
    }

    pub fn test_scene(&mut self) {
        let fbm = Fbm::<Perlin>::new(0);

        let mut rng = rand::thread_rng();
        for x in 0..1024 {
            for z in 0..1024 {
                let y = (fbm.get([x as f64, z as f64]) + 1.0) * 1024.0;
                self.insert_node(
                    Vector3::new(x as f32, y as f32, z as f32) * 2.0,
                    Vector4::new(
                        rng.gen_range(0.0..1.0),
                        rng.gen_range(0.0..1.0),
                        rng.gen_range(0.0..1.0),
                        1.0,
                    ),
                );
            }
        }

        self.insert_node(
            Vector3::new(0.0, 0.0, 0.0),
            Vector4::new(
                rng.gen_range(0.0..1.0),
                rng.gen_range(0.0..1.0),
                rng.gen_range(0.0..1.0),
                1.0,
            ),
        );
    }
}

impl Default for Octree {
    fn default() -> Self {
        Self {
            branch_data: vec![Subdivide::default(); 1],
            leaf_data: vec![Leaf::default(); 2],
            root_span: (1 << MAX_DEPTH) as f32,
        }
    }
}
