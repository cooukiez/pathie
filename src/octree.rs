use cgmath::{Vector3};
use rand::Rng;

use crate::{service::{pos_to_index, step_vec_three}};

pub const MAX_RECURSION: usize = 15;
pub const ROOT_SPAN: f32 = (1 << MAX_RECURSION) as f32;

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct TreeNode {
    // 0 = empty | 1 = subdivide | 2 = full
    pub node_type: u32,
    pub parent: u32,
    
    pub children: [u32; 8],
    pub base_color: Vector3<f32>,
}

// Store Info about OctreeTraverse
pub struct Traverse {
    pub cur_index: usize,
    pub parent: usize,
    pub cur_span: f32,
    pub cur_recursion: usize,

    pub ray_origin: Vector3<f32>,
    pub ray_dir: Vector3<f32>,

    // Origin in CurNode
    pub local_origin: Vector3<f32>,
    // Origin on first edge of CurNode
    pub origin_on_edge: Vector3<f32>,

    pub mask_in_parent: [Vector3<f32>; MAX_RECURSION],
}

pub struct Octree {
    // Octree as List
    pub data: Vec<TreeNode>,
    // RootIndex = 0
}

impl TreeNode {
    pub fn new(node_type: u32, parent: usize, ) -> TreeNode {
        TreeNode { node_type, parent: parent as u32, .. Default::default() }
    }

    pub fn set_full(&mut self, base_color: Vector3<f32>) {
        self.node_type = 2;
        self.base_color = base_color;
    }

    pub fn get_center(node_pos: Vector3<f32>, span: f32, ) -> Vector3<f32> {
        node_pos + Vector3::from([span * 0.5; 3])
    }

    pub fn get_top_right(node_pos: Vector3<f32>, span: f32, ) -> Vector3<f32> {
        node_pos + Vector3::from([span; 3])
    }

    pub fn get_child_mask(cur_span: f32, local_origin: Vector3<f32>, ) -> Vector3<f32> {
        step_vec_three(Vector3::from([cur_span; 3]), local_origin, )
            .cast::<f32>()
            .unwrap()
    }
}

impl Octree {
    // If not subdivide -> Create children
    pub fn try_child_creation(data: &mut Vec<TreeNode>, parent_index: usize, ) {
        if data[parent_index].node_type == 0 {
            data[parent_index].node_type = 1;
            data[parent_index].base_color = Vector3::from([0.5; 3]);

            for index in 0 .. 8 {
                data[parent_index as usize].children[index] = data.len() as u32;
                data.push(TreeNode::new(0, parent_index, ));
            }
        }
    }

    pub fn insert_node(&mut self, insert_pos: Vector3<f32>, base_color: Vector3<f32>, ) -> Traverse {
        let mut traverse = Traverse {
            ray_origin: insert_pos,

            local_origin: insert_pos % ROOT_SPAN,
            origin_on_edge: insert_pos - (insert_pos % ROOT_SPAN),

            .. Default::default()
        };

        for _  in 1 .. MAX_RECURSION {
            Self::try_child_creation(&mut self.data, traverse.cur_index);
            traverse.move_into_child(&self.data);
        }

        self.data[traverse.cur_index].set_full(base_color);

        let backward = Traverse {
            cur_index: traverse.cur_index,
            parent: traverse.parent,

            .. Default::default()
        };

        for _  in 1 .. MAX_RECURSION {
            let cur_color = self.data[backward.cur_index].base_color;
            self.data[backward.parent].base_color += cur_color;
            self.data[backward.parent].base_color *= 0.5;
            traverse.cur_index = traverse.parent;
            traverse.parent = self.data[traverse.cur_index].parent as usize;
        }

        traverse
    }

    pub fn test_scene(&mut self) {
        let mut rng = rand::thread_rng();
        for x in 0 .. 1000 {
            for z in 0 .. 1000 {
                let base_color = Vector3::new(rng.gen_range(0.0 .. 1.0), rng.gen_range(0.0 .. 1.0), rng.gen_range(0.0 .. 1.0));
                self.insert_node(Vector3::new(100.0 + x as f32, 100.0, 100.0 + z as f32, ), base_color);
            }
        }

        for _ in 0 .. 5000 {
            let base_color = Vector3::new(rng.gen_range(0.0 .. 1.0), rng.gen_range(0.0 .. 1.0), rng.gen_range(0.0 .. 1.0));
            let pos = Vector3::new(rng.gen_range(0.0 .. ROOT_SPAN), rng.gen_range(0.0 .. ROOT_SPAN), rng.gen_range(0.0 .. ROOT_SPAN));
            // self.insert_node(pos, base_color, );
        }
    }
}

impl Traverse {
    // Return SpaceIndex
    pub fn move_into_child(&mut self, data: &Vec<TreeNode>, ) {
        self.cur_recursion += 1;
        self.cur_span *= 0.5;

        let child_mask = TreeNode::get_child_mask(self.cur_span, self.local_origin, );

        self.origin_on_edge += child_mask * self.cur_span;
        self.local_origin -= child_mask * self.cur_span;

        // Save position in parent for later use
        self.mask_in_parent[self.cur_recursion] = child_mask.clone();
        
        let space_index = pos_to_index(child_mask, 2, );

        self.parent = self.cur_index;
        self.cur_index = data[self.parent].children[space_index] as usize;
    }
}

impl Default for TreeNode {
    fn default() -> Self {
        Self { 
            node_type: 0,
            parent: 0,
            children: [0; 8],
            base_color: Vector3::from([0.0; 3])
        }
    }
}

impl Default for Traverse {
    fn default() -> Self {
        Self { 
            cur_index: 0,
            parent: 0,
            cur_span: ROOT_SPAN,
            cur_recursion: 0,

            ray_origin: Vector3::from([0.0; 3]),
            ray_dir: Vector3::from([0.0; 3]),

            local_origin: Vector3::from([0.0; 3]),
            origin_on_edge: Vector3::from([0.0; 3]),

            mask_in_parent: [Vector3::from([0.0; 3]); MAX_RECURSION]
        }
    }
}

impl Default for Octree {
    fn default() -> Self {
        Self { data: vec![TreeNode::default()] }
    }
}