use cgmath::Vector3;

use crate::{service::Service, uniform::{VecThree, Uniform}};

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
    pub root_center: VecThree,
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

    pub fn get_outer_pos(center: &VecThree, sign: &Vector3<f32>, diameter: f32, ) -> Vector3<f32> {
        center.to_vec() + (sign * (diameter as f32))
    }

    pub fn get_bottom_left(center: &VecThree, span: f32, ) -> Vector3<f32> {
        center.to_vec() - VecThree::from_float(span / 2.0).to_vec()
    }

    pub fn get_top_right(center: &VecThree, span: f32, ) -> Vector3<f32> {
        center.to_vec() + VecThree::from_float(span / 2.0).to_vec()
    }

    pub fn get_child_mask(center: &VecThree, top_right: Vector3<f32>, cur_pos: Vector3<f32>,  ) -> VecThree {
        // log::info!("{:?} {:?} {:?}", center, top_right, cur_pos);
        VecThree { 
            x: Service::check_number_in_range([center.x, top_right.x], cur_pos.x, ) as u32 as f32,
                y: Service::check_number_in_range([center.y, top_right.y], cur_pos.y, ) as u32 as f32,
                    z: Service::check_number_in_range([center.z, top_right.z], cur_pos.z, ) as u32 as f32
        }
    }

    pub fn get_child_pos(child_mask: &VecThree, parent_center: &VecThree, parent_span: f32, ) -> VecThree {
        let parent_bottom_left = Self::get_bottom_left(parent_center, parent_span, );
        let local_pos = child_mask.to_vec() * parent_span / 2.0;

        VecThree::from_vec(parent_bottom_left + VecThree::from_float(parent_span / 4.0).to_vec() + local_pos)
    }

    pub fn choose_child_node(&self, parent_center: &VecThree, parent_span: f32, cur_pos: Vector3<f32>, ) -> (u32, VecThree, f32) {
        let top_right = Self::get_top_right(parent_center, parent_span, );

        let child_mask = Self::get_child_mask(parent_center, top_right, cur_pos, );

        // log::info!("{:?}", Self::get_child_mask(parent_center, top_right, cur_pos, ));
        let child_index = self.children[Service::pos_to_index(&child_mask.to_vec(), 2, ) as usize];
        let child_pos = Self::get_child_pos(&child_mask, parent_center, parent_span, );
        let child_span = parent_span / 0.5;

        (child_index, child_pos, child_span, )
    }
}

impl Octree {
    pub fn empty(uniform: &Uniform) -> Octree {
        let data: Vec<TreeNode> = vec![TreeNode::empty()];
        Octree { 
            root_span: uniform.root_span,
            root_center: uniform.root_center,
            max_recursion: uniform.max_recursion,
            data
        }
    }

    pub fn node_at_pos(&mut self, pos_to_find: Vector3<f32>, ) -> (u32, f32, ) {
        let mut cur_index = 0;
        let mut cur_node_center = self.root_center;
        let mut cur_span = self.root_span;

        log::info!("");

        for _  in 0 .. self.max_recursion {
            if self.data[cur_index as usize].node_type == 1 {
                (cur_index, cur_node_center, cur_span, ) = self.data[cur_index as usize]
                    .choose_child_node(&mut cur_node_center, cur_span, pos_to_find, );
            } else {
                // Case -> At Leaf -> Return current LeafNode
                return (cur_index, cur_span, )
            }
        }

        log::info!("{}", self.data[cur_index as usize].node_type);

        (cur_index, cur_span, )
    }

    pub fn insert_node(&mut self, insert_pos: Vector3<f32>, ) {
        let mut cur_index = 0;
        let mut cur_node_center = self.root_center;
        let mut cur_span = self.root_span;

        for _  in 0 .. self.max_recursion {
            // If not Subdivide then change into -> Create children
            if self.data[cur_index as usize].node_type == 0 {
                self.data[cur_index as usize].node_type = 1;

                for index in 0 .. 8 {
                    // Child Index = Next Index of OctreeData + CurChildIndex
                    self.data[cur_index as usize].children[index] = self.data.len() as u32;
                    self.data.push(TreeNode::new(0, cur_index, ));
                }
            }

            (cur_index, cur_node_center, cur_span, ) = self.data[cur_index as usize]
                    .choose_child_node(&mut cur_node_center, cur_span, insert_pos, );
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