use cgmath::Vector3;
use rand::Rng;

use crate::{uniform::Uniform, service::{pos_to_index, step_vec_three}};

const MAX_RECURSION: usize = 10;

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct TreeNode {
    // 0 = empty | 1 = subdivide | 2 = full
    pub node_type: u32,
    pub parent: u32,
    
    pub children: [u32; 8],
}

// Store Info about OctreeTraverse
pub struct Traverse {
    pub cur_index: usize,
    pub cur_span: f32,
    pub cur_recursion: usize,

    // Origin in CurNode
    pub local_origin: Vector3<f32>,
    // Origin on first edge of CurNode
    pub origin_on_edge: Vector3<f32>,

    pub mask_in_parent: [Vector3<f32>; MAX_RECURSION],
}

pub struct Octree {
    // RootIndex = 0
    pub root_span: f32,
    pub max_recursion: u32,
    // Octree as List
    pub data: Vec<TreeNode>,
}

impl TreeNode {
    pub fn empty() -> TreeNode {
        TreeNode { node_type: 0, parent: 0, children: [0; 8] }
    }

    pub fn new(node_type: u32, parent: usize, ) -> TreeNode {
        TreeNode { node_type, parent: parent as u32, children: [0; 8]}
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
    pub fn empty(uniform: &Uniform) -> Octree {
        let data: Vec<TreeNode> = vec![TreeNode::empty()];
        Octree { 
            root_span: uniform.root_span,
            max_recursion: uniform.max_recursion,
            data
        }
    }

    // If not subdivide -> Create children
    pub fn try_child_creation(data: &mut Vec<TreeNode>, parent_index: usize, ) {
        if data[parent_index].node_type == 0 {
            data[parent_index].node_type = 1;

            for index in 0 .. 8 {
                data[parent_index as usize].children[index] = data.len() as u32;
                data.push(TreeNode::new(0, parent_index, ));
            }
        }
    }

    pub fn insert_node(&mut self, insert_pos: Vector3<f32>, ) {
        let mut traverse = Traverse {
            cur_index: 0,
            cur_span: self.root_span,
            cur_recursion: 0,

            local_origin: insert_pos % self.root_span,
            origin_on_edge: insert_pos - (insert_pos % self.root_span),

            mask_in_parent: [Vector3::from([0.0; 3]); MAX_RECURSION],
        };
        
        // 1. Create children if not present
        // 2. Select child based on insert position
        // 3. Repeat
        // Finally -> Set current Node to full
        for _  in 0 .. self.max_recursion {
            Self::try_child_creation(&mut self.data, traverse.cur_index);
            // Select next Child
            traverse.move_into_child(&self.data);
        }

        // CurNode = full
        self.data[traverse.cur_index].node_type = 2;
    }

    // Not diagonal
    pub fn insert_neighbour(&mut self, traverse: &mut Traverse, dir: Vector3<f32>) {
        let exitOctree = false;
        for _ in 0 .. self.max_recursion -  {
            if exitOctree {

            } else {
                // Move forward
                let new_origin_on_edge = 
                    traverse.origin_on_edge + dir * traverse.cur_span;
            }
        }
    }

    pub fn test_scene(&mut self) {
        self.insert_node(Vector3::new(120.3, 321.2, 213.1));
        self.insert_node(Vector3::new(10.3, 230.4, 60.0));
        self.insert_node(Vector3::new(10.1, 210.0, 46.7));
        self.insert_node(Vector3::new(400.1, 10.0, 100.7));
        // self.insert_node(Vector3::new(255.1, 255.0, 2.7));

        let mut rng = rand::thread_rng();
        for _ in 0 .. 2000 {
            self.insert_node(Vector3::new(rng.gen_range(0.0 .. 4000.0), rng.gen_range(0.0 .. 4000.0), rng.gen_range(0.0 .. 4000.0)));
        }
    }
}

impl Traverse {
    // Return SpaceIndex
    pub fn move_into_child(&mut self, data: &Vec<TreeNode>, ) {
        self.cur_span *= 0.5;
        self.cur_recursion += 1;

        let child_mask = TreeNode::get_child_mask(self.cur_span, self.local_origin, );

        self.origin_on_edge += child_mask * self.cur_span;
        self.local_origin -= child_mask * self.cur_span;

        // Save position in parent for later use
        self.mask_in_parent[self.cur_recursion] = child_mask.clone();
        
        let space_index = pos_to_index(child_mask, 2, );
        // Get global index of selected child
        self.cur_index = data[self.cur_index].children[space_index] as usize;
    }
}