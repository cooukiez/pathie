use cgmath::{Vector4, Vector3};
use rand::Rng;

use crate::{service::{Vector, Mask}};

pub const MAX_DEPTH: usize = 17;
pub const ROOT_SPAN: f32 = ((1 << MAX_DEPTH)) as f32;

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
    pub mask_in_parent: [Vector3<f32>; MAX_DEPTH], // Position in parent at depth

    pub local_pos: Vector3<f32>, // Origin in CurNode
    pub pos_on_edge: Vector3<f32>, // Origin on first edge of CurNode

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
    pub fn new(parent: usize, ) -> TreeNode {
        TreeNode { node_type: 0, parent: parent as u32, .. Default::default() }
    }

    pub fn set(&mut self, base_color: Vector4<f32>, node_type: u32, ) {
        self.node_type = node_type;
        self.base_color = base_color;
    }

    pub fn get_child_mask(cur_span: f32, local_origin: Vector3<f32>, ) -> Vector3<f32> {
        local_origin
            .step(Vector3::from([cur_span; 3]))
    }
}

impl Octree {
    pub fn node_at_pos(&mut self, pos: Vector3<f32>, ) -> PosInfo {
        let mut pos_info = PosInfo {
            span: ROOT_SPAN,

            local_pos: pos % ROOT_SPAN,
            pos_on_edge: pos - (pos % ROOT_SPAN),

            .. Default::default()
        };

        for _  in 1 .. MAX_DEPTH {
            if pos_info.node_type(&self.data) == 1 {
                pos_info.move_into_child(&self.data);
            } else {
                break;
            }
        }

        pos_info
    }

    pub fn insert_node(&mut self, insert_pos: Vector3<f32>, base_color: Vector4<f32>, node_type: u32, max_depth: usize, ) -> PosInfo {
        let mut pos_info = PosInfo {
            span: ROOT_SPAN,

            local_pos: insert_pos % ROOT_SPAN,
            pos_on_edge: insert_pos - (insert_pos % ROOT_SPAN),

            .. Default::default()
        };

        for _  in 1 .. max_depth {
            pos_info.try_child_creation(&mut self.data, base_color, );
            pos_info.move_into_child(&self.data);
        }

        self.data[pos_info.index()].set(base_color.clone(), node_type, );

        pos_info
    }

    pub fn insert_light(&mut self, insert_pos: Vector3<f32>, light_color: Vector4<f32>, ) {
        let pos_info = self.insert_node(insert_pos, light_color, 3, MAX_DEPTH - 2, );
        self.light_data.push(Light { pos: insert_pos.extend(0.0), index: pos_info.index, .. Default::default() });
    }

    pub fn test_scene(&mut self) {
        // Ground
        for x in 0 .. 100 {
            for z in 0 .. 100 {
                let base_color = Vector4::new(1.0, 1.0, 1.0, 0.0, );
                self.insert_node(Vector3::new(100.0 + x as f32, 100.0, 100.0 + z as f32, ), base_color, 2, MAX_DEPTH);
            }
        }

        // GreenWall
        for z in 0 .. 100 {
            for y in 0 .. 100 {
                let base_color = Vector4::new(0.0, 1.0, 0.0, 0.0, );
                self.insert_node(Vector3::new(100.0, 100.0 + y as f32, 100.0 + z as f32, ), base_color, 2, MAX_DEPTH);
            }
        }

        // RedWall
        for z in 0 .. 100 {
            for y in 0 .. 100 {
                let base_color = Vector4::new(1.0, 0.0, 0.0, 0.0, );
                self.insert_node(Vector3::new(200.0, 100.0 + y as f32, 100.0 + z as f32, ), base_color, 2, MAX_DEPTH);
            }
        }

        // BackWall
        for x in 0 .. 100 {
            for y in 0 .. 100 {
                let base_color = Vector4::new(1.0, 1.0, 1.0, 0.0, );
                self.insert_node(Vector3::new(100.0 + x as f32, 100.0 + y as f32, 200.0, ), base_color, 2, MAX_DEPTH);
            }
        }

        // Ceilling
        for x in 0 .. 100 {
            for z in 0 .. 100 {
                let base_color = Vector4::new(1.0, 1.0, 1.0, 0.0, );
                self.insert_node(Vector3::new(100.0 + x as f32, 200.0, 100.0 + z as f32, ), base_color, 2, MAX_DEPTH);
            }
        }

        // Box
        for x in 0 .. 20 {
            for z in 0 .. 20 {
                for y in 0 .. 20 {
                    let base_color = Vector4::new(0.0, 0.0, 1.0, 0.0, );
                    self.insert_node(Vector3::new(140.0 + x as f32, 100.0 + y as f32, 140.0 + z as f32, ), base_color, 2, MAX_DEPTH);
                }
            }
        }

        for _ in 0 .. 10000 {
            let mut rng = rand::thread_rng();
            let base_color = Vector4::new(0.0, 0.0, 0.5, 0.0, );
            // self.insert_node(Vector3::new(rng.gen_range(0.0 .. 500.0), rng.gen_range(0.0 .. 500.0), rng.gen_range(0.0 .. 500.0), ), base_color, 2, MAX_DEPTH);
        }

        self.insert_node(Vector3::new(150.0, 190.0, 150.0, ), Vector4::new(1.0, 0.0, 0.0, 0.0, ), 2, MAX_DEPTH);

        // Light
        let light_color = Vector4::new(1.0, 1.0, 0.0, 0.0, );
        self.insert_light(Vector3::new(150.0, 180.0, 150.0, ), light_color, );
    }
}

impl PosInfo {
    pub fn index(&self) -> usize { self.index as usize }
    pub fn parent(&self, data: &Vec<TreeNode>, ) -> usize { data[self.index()].parent as usize }
    pub fn node_type(&self, data: &Vec<TreeNode>, ) -> u32 { data[self.index()].node_type } 

    pub fn try_child_creation(&mut self, data: &mut Vec<TreeNode>, base_color: Vector4<f32>, ) {
        let mut node = data[self.index()];
        if node.node_type == 1 {
            node.base_color += base_color.clone() / self.span;
        } else {
            node.set(base_color.clone() / self.span, 1, );

            for index in 0 .. 8 {
                node.children[index] = data.len() as u32;
                data.push(TreeNode::new(self.index()));
            }
        }

        data[self.index()] = node;
    }

    pub fn move_into_child(&mut self, data: &Vec<TreeNode>, ) {
        self.span *= 0.5;
        self.depth += 1;

        let child_mask =
            TreeNode::get_child_mask(self.span, self.local_pos, );

        self.pos_on_edge += child_mask * self.span;
        self.local_pos -= child_mask * self.span;

        self.mask_in_parent[self.depth as usize] = child_mask.clone();
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
            padding: [0; 2]
        }
    }
}

impl Default for Light {
    fn default() -> Self {
        Self {
            pos: Vector4::default(),
            index: 0,
            padding: [0; 3]
        }
    }
}

impl Default for Ray {
    fn default() -> Self {
        Self {
            origin: Vector3::default(),
            dir: Vector3::default()
        }
    }
}

impl Default for PosInfo {
    fn default() -> Self {
        Self {
            mask_in_parent: [Vector3::from([0.0; 3]); MAX_DEPTH],

            local_pos: Vector3::default(),
            pos_on_edge: Vector3::default(),

            index: 0,
            span: ROOT_SPAN,
            depth: 0,
        }
    }
}

impl Default for Octree {
    fn default() -> Self {
        log::info!("Creating Octree with RootSpan [ {} ] -> VoxSpan is 1.0 ...", ROOT_SPAN);
        Self {
            data: vec![TreeNode::default()],
            light_data: vec![]
        }
    }
}