use cgmath::Vector3;
use rand::Rng;

use crate::{service::Service};

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct TreeNode {
    pub mat: u32,
    pub parent: u32,

    pub span: f32,
    pub space_index: u32,
    
    pub children: [u32; 8],

    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub struct Octree {
    pub octree_root: u32,
    pub data: Vec<TreeNode>,
}

impl TreeNode {
    pub fn empty() -> TreeNode {
        TreeNode { mat: 0, parent: 0, span: 0.0, space_index: 0, children: [0; 8], x: 0.0, y: 0.0, z: 0.0 }
    }

    pub fn new(parent: u32, span: f32, space_index: u32, pos: Vector3<f32>, ) -> TreeNode {
        TreeNode { mat: 0, parent, span, space_index, children: [0; 8], x: pos.x, y: pos.y, z: pos.z }
    }

    pub fn get_outer_pos(&self, sign: &Vector3<f32>, diameter: f32, ) -> Vector3<f32> {
        Vector3::new(self.x, self.y, self.z, ) + (sign * (diameter as f32))
    }

    pub fn get_bottom_left(&self) -> Vector3<f32> {
        Vector3::new(self.x - self.span / 2.0, self.y - self.span / 2.0, self.z - self.span / 2.0)
    }

    pub fn get_top_right(&self) -> Vector3<f32> {
        Vector3::new(self.x + self.span / 2.0, self.y + self.span / 2.0, self.z + self.span / 2.0)
    }

    pub fn create_child(&self, space_index: usize, parent: u32, ) -> TreeNode {
        let local_pos = Service::convert_index_to_pos(space_index as u32, 2) * self.span / 2.0;
        let global_pos = self.get_bottom_left() + Vector3::new(self.span / 4.0, self.span / 4.0, self.span / 4.0) + local_pos;
        
        TreeNode::new(parent, self.span / 2.0, space_index as u32, global_pos, )
    }

    pub fn choose_child_node(&self, cur_pos: &Vector3<f32>, ) -> u32 {
        let parent_pos = Vector3::new(self.x, self.y, self.z);

        let horizontal = Service::check_number_in_range([parent_pos.x, parent_pos.x + self.span / 2.0], cur_pos.x, ) as u32 as f32;
        let vertical = Service::check_number_in_range([parent_pos.y, parent_pos.y + self.span / 2.0], cur_pos.y, ) as u32 as f32;
        let depth = Service::check_number_in_range([parent_pos.z, parent_pos.z + self.span / 2.0], cur_pos.z, ) as u32 as f32;
        
        Service::pos_to_index(&Vector3::new(horizontal, vertical, depth), 2)
    }
}

impl Octree {
    pub fn insert_node(root_index: usize, data: &mut Vec<TreeNode>, pos_to_insert: Vector3<f32>, ) -> usize {
        let mut cur_index = root_index;
        while data[cur_index].span > 1.0 {
            if data[cur_index].children == [0; 8] {
                for index in 0 .. data[cur_index].children.len() {
                    data[cur_index].children[index] = data.len() as u32;

                    data.push(data[cur_index]
                        .create_child(index, cur_index as u32, ))
                }
            }

            let child_node_index = (&data[cur_index].choose_child_node(&pos_to_insert)).clone();
            cur_index = (&data[cur_index].children[child_node_index as usize]).clone() as usize;
        } cur_index
    }

    pub fn collect(root_index: usize, vox_amount: u32, span: f32, ) -> Octree {
        let mut data: Vec<TreeNode> = vec![TreeNode::empty()]; 
        let mut rnd = rand::thread_rng();
        data[root_index] = TreeNode::new(root_index as u32, span, 0, Vector3::new(0.0, 0.0, 0.0, ), );

        // Remove Later
        for _ in 0 .. vox_amount {
            let pos_to_insert = Vector3 { 
                x: rnd.gen_range((- span / 2.0) .. (span / 2.0)), y: rnd.gen_range((- span / 2.0) .. (span / 2.0)), z: rnd.gen_range((- span / 2.0) .. (span / 2.0)),
            };

            Self::insert_node(0, &mut data, pos_to_insert, ); 
        }

        Octree { octree_root: 0, data }
    }
}