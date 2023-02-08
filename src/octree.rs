use cgmath::Vector3;
use rand::Rng;

use crate::{uniform::{VecThree, Uniform}, service::{pos_to_index, step_vec_three}};

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct TreeNode {
    // 0 = empty | 1 = subdivide | 2 = full
    pub node_type: u32,
    pub parent: u32,
    
    pub children: [u32; 8],
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

    pub fn new(node_type: u32, parent: u32, ) -> TreeNode {
        TreeNode { node_type, parent, children: [0; 8]}
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

    pub fn create_children(data: &mut Vec<TreeNode>, parent_index: u32, ) {
        data[parent_index as usize].node_type = 1;
        for index in 0 .. 8 {
            // Child Index = Next Index of OctreeData + CurChildIndex
            data[parent_index as usize].children[index] = data.len() as u32;
            data.push(TreeNode::new(0, parent_index, ));
        }
    }

    pub fn insert_node(&mut self, insert_pos: Vector3<f32>, ) {
        let mut cur_index = 0;
        let mut cur_span = self.root_span;

        let mut local_origin  = insert_pos % cur_span;
        let mut origin_on_edge = insert_pos - local_origin;

        let mut child_mask;
        
        for _  in 0 .. self.max_recursion {
            if self.data[cur_index as usize].node_type == 0 {
                Self::create_children(&mut self.data, cur_index, );
            }

            // Moving into Child now
            cur_span *= 0.5;
            child_mask = TreeNode::get_child_mask(cur_span, local_origin, );

            origin_on_edge += child_mask * cur_span;
            local_origin -= child_mask * cur_span;
            
            cur_index = self.data[cur_index as usize]
                .children[pos_to_index(child_mask, 2, ) as usize];
        }

        // Set CurNode to full
        self.data[cur_index as usize].node_type = 2;
    }

    pub fn collect_random(&mut self) {
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