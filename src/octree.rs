use cgmath::{Vector3, Vector4};
use noise::{
    Fbm, NoiseFn, Perlin,
};
use rand::Rng;

use crate::service::{Mask, Vector};

pub const MAX_DEPTH: usize = 10;
pub const ROOT_SPAN: f32 = (1 << MAX_DEPTH) as f32;

// In struct, Vector four is used because of memory alignment in vulkan.
// Vector three is aligned as vec four in vulkan but as vec three in rust.
// This is problematic therefore we use vec four.

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct TreeNode {
    pub children: [u32; 8],

    // 0 = empty | 1 = subdivide | 2 = full | 3 = light
    pub node_type: u32,
    pub parent: u32,

    pub padding: [u32; 2],

    pub base_color: Vector4<f32>,
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Light {
    pub pos: Vector4<f32>,
    pub index: u32,

    pub padding: [u32; 3],
}

#[derive(Clone, Debug)]
pub struct Ray {
    pub origin: Vector3<f32>,
    pub dir: Vector3<f32>,
}

#[derive(Clone, Debug)]
pub struct PosInfo {
    pub mask_info: [Vector4<f32>; MAX_DEPTH], // Position in parent at depth

    pub local_pos: Vector4<f32>,   // Origin in CurNode
    pub pos_on_edge: Vector4<f32>, // Origin on first edge of CurNode

    pub index: u32,
    pub span: f32,
    pub depth: i32,
}

pub struct Octree {
    // RootIndex = 0
    pub data: Vec<TreeNode>, // Octree as List
    pub light_data: Vec<Light>,
}

impl TreeNode {
    pub fn new(parent: usize) -> TreeNode {
        TreeNode {
            node_type: 0,
            parent: parent as u32,
            ..Default::default()
        }
    }

    pub fn set(&mut self, base_color: Vector4<f32>, node_type: u32) {
        self.node_type = node_type;
        self.base_color = base_color;
    }

    pub fn get_child_mask(cur_span: f32, local_origin: Vector3<f32>) -> Vector3<f32> {
        local_origin.step(Vector3::from([cur_span; 3]))
    }
}

impl Octree {
    /// Create children for an existing node.
    /// First add the material effect stuff divided by span to the
    /// material of the node to take LOD effect into
    /// account. If node has no children, create them.
    
    pub fn create_children(data: &mut Vec<TreeNode>, pos_info: &PosInfo, base_color: Vector4<f32>) {
        let mut node = data[pos_info.index()];

        if node.node_type == 1 {
            node.base_color += base_color.clone() / pos_info.span;
        } else {
            node.set(base_color.clone() / pos_info.span, 1);

            for index in 0..8 {
                node.children[index] = data.len() as u32;
                data.push(TreeNode::new(pos_info.index()));
            }
        }

        data[pos_info.index()] = node;
    }

    pub fn node_at_pos(&mut self, pos: Vector3<f32>) -> PosInfo {
        let mut pos_info = PosInfo {
            span: ROOT_SPAN,

            local_pos: pos % ROOT_SPAN,
            pos_on_edge: pos - (pos % ROOT_SPAN),

            ..Default::default()
        };

        for _ in 1..MAX_DEPTH {
            if pos_info.node(&self.data).node_type == 1 {
                pos_info.move_into_child(&self.data);
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
            span: ROOT_SPAN,

            local_pos: (insert_pos % ROOT_SPAN).extend(0.0),
            pos_on_edge: (insert_pos - (insert_pos % ROOT_SPAN)).extend(0.0),

            ..Default::default()
        };

        for _ in 1..MAX_DEPTH {
            Self::create_children(&mut self.data, &pos_info, base_color.clone());
            pos_info.move_into_child(&self.data);
        }

        self.data[pos_info.index()].set(base_color.clone(), node_type);

        pos_info
    }

    pub fn test_scene(&mut self) {
        let fbm = Fbm::<Perlin>::new(0);

        let mut rng = rand::thread_rng();
        for x in 0..1000 {
            for z in 0..1000 {
                let y = (fbm.get([x as f64, z as f64]) + 1.0) * 20.0;
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
    }
}

impl PosInfo {
    pub fn index(&self) -> usize {
        self.index as usize
    }
    pub fn node(&self, data: &Vec<TreeNode>) -> TreeNode {
        data[self.index()].clone()
    }

    pub fn move_into_child(&mut self, data: &Vec<TreeNode>) {
        self.span *= 0.5;
        self.depth += 1;

        let child_mask = TreeNode::get_child_mask(self.span, self.local_pos.truncate());

        self.pos_on_edge += (child_mask * self.span).extend(0.0);
        self.local_pos -= (child_mask * self.span).extend(0.0);

        self.mask_info[self.depth as usize] = child_mask.clone().extend(0.0);
        let space_index = child_mask.to_index(2.0);

        self.index = data[self.index()].children[space_index] as u32;
    }
}

impl Default for TreeNode {
    fn default() -> Self {
        Self {
            base_color: Vector4::default(),
            children: [0; 8],
            node_type: 0,
            parent: 0,
            padding: [0; 2],
        }
    }
}

impl Default for Light {
    fn default() -> Self {
        Self {
            pos: Vector4::default(),
            index: 0,
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
            mask_info: [Vector4::default(); MAX_DEPTH],

            local_pos: Vector4::default(),
            pos_on_edge: Vector4::default(),

            index: 0,
            span: ROOT_SPAN,
            depth: 0,
        }
    }
}

impl Default for Octree {
    fn default() -> Self {
        log::info!(
            "Creating Octree with RootSpan [ {} ] -> VoxSpan is 1.0 ...",
            ROOT_SPAN
        );
        Self {
            data: vec![TreeNode::default()],
            light_data: vec![],
        }
    }
}
