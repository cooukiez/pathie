use cgmath::{Vector3, Vector4};
use noise::{Fbm, NoiseFn, Perlin};
use rand::Rng;

use crate::service::{Mask, Vector};

pub const MAX_DEPTH: usize = 10;
pub const MAX_NODE: usize = 8192;

// In struct, Vector four is used because of memory alignment in vulkan.
// Vector three is aligned as vec four in vulkan but as vec three in rust.
// This is problematic therefore we use vec four.

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Material {
    pub base_color: Vector4<f32>,
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Subdivide {
    // Store index, 0 = empty | > 0 = full -> subdiv or leaf
    pub children: [i32; 8],
    // Store index, 0 = empty | 1 = full, store compact with bitshifting
    pub basic_children: u32,

    pub parent: i32,
    pub padding: [u32; 2],
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Leaf {
    // Store material here later on
    pub mat: Material,

    pub parent: i32,
    pub padding: [u32; 3],
}

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

    pub index: i32,
    pub span: f32,
    pub depth: i32,
}

pub struct Octree {
    // RootIndex = 0
    pub branch_data: Vec<Subdivide>, // All subdivide as list
    pub leaf_data: Vec<Leaf>,      // Leaf node list
    pub root_span: f32,
}

impl Subdivide {
    pub fn new(parent: usize) -> Subdivide {
        Subdivide {
            parent: parent as i32,
            ..Default::default()
        }
    }

    pub fn has_children(&self) -> bool {
        self.children[0] > 0
    }

    pub fn get_child_mask(cur_span: f32, local_origin: Vector3<f32>) -> Vector3<f32> {
        local_origin.step(Vector3::from([cur_span; 3]))
    }
}

impl Leaf {
    pub fn new(parent: usize) -> Leaf {
        Self {
            parent: parent as i32,
            ..Default::default()
        }
    }
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
                subdiv.children[index] = -(self.leaf_data.len() as i32);
                self.leaf_data.push(Leaf::new(pos_info.index()));
            }

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

    pub fn insert_node(
        &mut self,
        insert_pos: Vector3<f32>,
        base_color: Vector4<f32>,
        node_type: u32,
    ) -> PosInfo {
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

        self.node_data[pos_info.index()].set(base_color.clone(), node_type);

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
                    2,
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
            2,
        );
    }
}

impl PosInfo {
    pub fn index(&self) -> usize {
        self.index as usize
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
        self.index >= 0
    }

    pub fn parent_idx(&self, octree: &Octree) -> i32 {
        if self.is_subdiv() {
            self.as_subdiv(&octree.branch_data).parent
        } else {
            self.as_leaf(&octree.leaf_data).parent
        }
    }

    pub fn parent(&self, octree: &Octree) -> Subdivide {
        octree.branch_data[self.parent_idx(octree) as usize]
    }

    /// Function not tested

    pub fn neighbor(
        &self,
        octree: &Octree,
        max_depth: usize,
        dir_mask: &Vector3<f32>,
    ) -> Option<PosInfo> {
        let mut pos_info = self.clone();

        for depth in self.depth as usize..max_depth {
            let new_mask = self.mask_info[depth].truncate() + dir_mask.clone();

            // Check if move up
            if new_mask.any(|num| num > 1.0 || num < 0.0) {
                pos_info.move_up(octree);
            } else {
                // Stop moving up and get next node
                let space_index = dir_mask.to_index(2.0);
                pos_info.index = self.parent(octree).children[space_index];

                // Start moving down
                while pos_info.is_subdiv() {
                    pos_info.move_into_child(&octree.branch_data);
                }

                return Some(pos_info);
            }
        }

        None
    }

    pub fn move_up(&mut self, octree: &Octree) {
        let pos_mask = self.mask_info[self.depth_idx()];
        self.pos_on_edge -= pos_mask * self.span;
        self.local_pos += pos_mask * self.span;

        self.span *= 2.0;
        self.depth -= 1;


        // New index is parent
        self.index = self.parent_idx(octree);
    }

    /// Expect child to be subdivide

    pub fn move_into_child(&mut self, branch_data: &Vec<Subdivide>) {
        self.span *= 0.5;
        self.depth += 1;

        // Get which child node to choose
        let child_mask = Subdivide::get_child_mask(self.span, self.local_pos.truncate());

        self.pos_on_edge += (child_mask * self.span).extend(0.0);
        self.local_pos -= (child_mask * self.span).extend(0.0);

        self.mask_info[self.depth_idx()] = (child_mask.clone()).extend(0.0);
        let space_index = child_mask.to_index(2.0);

        // New Index is child node, Save branch info about parent
        self.index = self.as_subdiv(branch_data).children[space_index] as i32;
        self.branch_info[self.depth_idx()] = self.as_subdiv(branch_data);
    }
}

impl Default for Material {
    fn default() -> Self {
        Self {
            base_color: Vector4::default(),
        }
    }
}

impl Default for Subdivide {
    fn default() -> Self {
        Self {
            children: [0; 8],
            basic_children: 0,
            parent: 0,
            padding: [0; 2],
        }
    }
}

impl Default for Leaf {
    fn default() -> Self {
        Self {  
            mat: Material::default(),
            parent: 0,
            padding: [0; 3],
        }
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

            index: 0,
            span: 0.0,
            depth: 0,
        }
    }
}

impl Default for Octree {
    fn default() -> Self {
        Self {
            branch_data: vec![Subdivide::default()],
            leaf_data: vec![Leaf::default()],
            root_span: (1 << MAX_DEPTH) as f32,
        }
    }
}
