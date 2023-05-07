use cgmath::{Vector3, Vector4};
use noise::{Fbm, NoiseFn, Perlin};
use rand::Rng;

use crate::service::{Mask, Vector};

pub const MAX_DEPTH: usize = 8;
pub const ROOT_SPAN: f32 = (1 << MAX_DEPTH) as f32;
pub const MICRO_GROUP_LEN: usize = 16;
pub const MICRO_GROUP_SIZE: usize = MICRO_GROUP_LEN * MICRO_GROUP_LEN * MICRO_GROUP_LEN;

// In struct, Vector four is used because of memory alignment in vulkan.
// Vector three is aligned as vec four in vulkan but as vec three in rust.
// This is problematic therefore we use vec four.

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct TreeNode {
    pub children: [u32; 8],

    // 0 = empty | 1 = subdivide | 2 = MicroGroup
    pub node_type: u32,
    pub parent: u32,

    pub padding: [u32; 2],

    pub base_color: Vector4<f32>,
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct MicroGroup {
    // Store material info in each cell
    pub data: [u32; MICRO_GROUP_SIZE],

    // First three comp. offset from 0,0
    // Last comp. is parent in octree
    pub loc_data: Vector4<f32>,
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Light {
    pub pos: Vector3<f32>,
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
    pub node_data: Vec<TreeNode>, // Octree as List
    pub micro_group_data: Vec<MicroGroup>,
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
        self.base_color = base_color / MICRO_GROUP_LEN as f32;
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

    pub fn create_children(
        node_data: &mut Vec<TreeNode>,
        pos_info: &PosInfo,
        base_color: Vector4<f32>,
    ) {
        let mut node = node_data[pos_info.index()];

        if node.node_type == 1 {
            node.base_color += base_color.clone() / pos_info.span / MICRO_GROUP_LEN as f32;
        } else {
            node.set(
                base_color.clone() / pos_info.span / MICRO_GROUP_LEN as f32,
                1,
            );

            for index in 0..8 {
                node.children[index] = node_data.len() as u32;
                node_data.push(TreeNode::new(pos_info.index()));
            }
        }

        node_data[pos_info.index()] = node;
    }

    pub fn insert_into_micro_group(
        node_data: &mut Vec<TreeNode>,
        micro_group_data: &mut Vec<MicroGroup>,
        pos_info: &PosInfo,
        base_color: Vector4<f32>,
    ) {
        let mut node = node_data[pos_info.index()];

        if node.node_type != 2 {
            node_data[pos_info.index()].set(base_color.clone(), 2);

            node.children[0] = micro_group_data.len() as u32;

            micro_group_data.push(MicroGroup {
                loc_data: pos_info
                    .pos_on_edge
                    .truncate()
                    .extend(pos_info.index() as f32),
                ..Default::default()
            });
        }

        micro_group_data[node.children[0] as usize].data[pos_info
            .local_pos
            .truncate()
            .to_index(MICRO_GROUP_LEN as f32)] = base_color.truncate().to_index(256.0) as u32;
    }

    pub fn node_at_pos(&mut self, pos: Vector3<f32>) -> PosInfo {
        let mut pos_info = PosInfo {
            span: ROOT_SPAN,

            local_pos: (pos % ROOT_SPAN).extend(0.0),
            pos_on_edge: (pos - (pos % ROOT_SPAN)).extend(0.0),

            ..Default::default()
        };

        for _ in 1..MAX_DEPTH {
            if pos_info.node(&self.node_data).node_type == 1 {
                pos_info.move_into_child(&self.node_data);
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
            Self::create_children(&mut self.node_data, &pos_info, base_color.clone());
            pos_info.move_into_child(&self.node_data);
        }

        Self::insert_into_micro_group(&mut self.node_data, &mut self.micro_group_data, &pos_info, base_color);

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
    }
}

impl PosInfo {
    pub fn index(&self) -> usize {
        self.index as usize
    }
    pub fn node(&self, node_data: &Vec<TreeNode>) -> TreeNode {
        node_data[self.index()].clone()
    }

    pub fn move_into_child(&mut self, node_data: &Vec<TreeNode>) {
        self.span *= 0.5;
        self.depth += 1;

        let child_mask = TreeNode::get_child_mask(self.span, self.local_pos.truncate());

        self.pos_on_edge += (child_mask * self.span).extend(0.0);
        self.local_pos -= (child_mask * self.span).extend(0.0);

        self.mask_info[self.depth as usize] = (child_mask.clone()).extend(0.0);
        let space_index = child_mask.to_index(2.0);

        self.index = node_data[self.index()].children[space_index] as u32;
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

impl Default for MicroGroup {
    fn default() -> Self {
        Self {
            data: [0; MICRO_GROUP_SIZE],
            loc_data: Vector4::default(),
        }
    }
}

impl Default for Light {
    fn default() -> Self {
        Self {
            pos: Vector3::default(),
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
            node_data: vec![TreeNode::default()],
            micro_group_data: vec![],
            light_data: vec![],
        }
    }
}
