use cgmath::Vector3;

use crate::{uniform::{VecThree, Uniform}, service::{pos_to_index}};

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
    pub root_center: Vector3<f32>,
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

    pub fn get_bottom_left(center: Vector3<f32>, span: f32, ) -> Vector3<f32> {
        center - VecThree::from_float(span / 2.0).to_vec()
    }

    pub fn get_top_right(center: Vector3<f32>, span: f32, ) -> Vector3<f32> {
        center + VecThree::from_float(span / 2.0).to_vec()
    }

    pub fn get_child_mask(parent_center: Vector3<f32>, parent_span: f32, cur_pos: Vector3<f32>,  ) -> Vector3<u32> {
        let top_right = Self::get_top_right(parent_center, parent_span, );
        Vector3 {
            x: (parent_center.x .. top_right.x).contains(&cur_pos.x) as u32,
                y: (parent_center.y .. top_right.y).contains(&cur_pos.y) as u32,
                    z: (parent_center.z .. top_right.z).contains(&cur_pos.z) as u32
        }
    }

    pub fn get_child_pos(child_mask: Vector3<f32>, parent_center: Vector3<f32>, parent_span: f32, ) -> Vector3<f32> {
        let bottom_left = Self::get_bottom_left(parent_center, parent_span, );
        bottom_left + Vector3::from([parent_span / 4.0; 3]) + child_mask * parent_span / 2.0
    }

    pub fn choose_child_node(&self, parent_center: Vector3<f32>, parent_span: f32, cur_pos: Vector3<f32>, ) -> (u32, Vector3<f32>, f32) {
        let child_mask: Vector3<f32> = Self::get_child_mask(parent_center, parent_span, cur_pos, ).cast::<f32>().unwrap();

        let child_index = self.children[pos_to_index(child_mask, 2, ) as usize];
        let child_pos = Self::get_child_pos(child_mask, parent_center, parent_span, );

        (child_index, child_pos, parent_span * 0.5, )
    }
}

impl Octree {
    pub fn empty(uniform: &Uniform) -> Octree {
        let data: Vec<TreeNode> = vec![TreeNode::empty()];
        Octree { 
            root_span: uniform.root_span,
            root_center: uniform.root_center.to_vec(),
            max_recursion: uniform.max_recursion,
            data
        }
    }

    pub fn node_at_pos(&mut self, pos_to_find: Vector3<f32>, ) -> (u32, f32, u32, ) {
        let mut cur_index = 0;
        let mut cur_node_center = self.root_center;
        let mut cur_span = self.root_span;

        for cur_recursion in 0 .. self.max_recursion {
            if self.data[cur_index as usize].node_type == 1 {
                (cur_index, cur_node_center, cur_span, ) = self.data[cur_index as usize]
                    .choose_child_node(cur_node_center, cur_span, pos_to_find, );
            } else {
                return (cur_index, cur_span, cur_recursion, )
            }
        }

        (cur_index, cur_span, self.max_recursion, )
    }

    pub fn insert_node(&mut self, insert_pos: Vector3<f32>, ) {
        let mut cur_index = 0;
        let mut cur_node_center = self.root_center;
        let mut cur_span = self.root_span;

        for _  in 0 .. self.max_recursion {
            // If not Subdivide -> Create children
            if self.data[cur_index as usize].node_type == 0 {
                self.data[cur_index as usize].node_type = 1;

                for index in 0 .. 8 {
                    // Child Index = Next Index of OctreeData + CurChildIndex
                    self.data[cur_index as usize].children[index] = self.data.len() as u32;
                    self.data.push(TreeNode::new(0, cur_index, ));
                }
            }

            (cur_index, cur_node_center, cur_span, ) = self.data[cur_index as usize]
                .choose_child_node(cur_node_center, cur_span, insert_pos, );
        }

        if cur_index == 472 {
              log::info!("");
        }

        // Set CurNode to full
        self.data[cur_index as usize].node_type = 2;
    }

    pub fn collect_random(&mut self) {
        self.insert_node(Vector3::new(12.3, 54.2, 50.1));
        self.insert_node(Vector3::new(1.3, -23.4, 60.0));
        self.insert_node(Vector3::new(-8.1, -21.0, 46.7));
    }
}