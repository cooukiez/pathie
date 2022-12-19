use cgmath::Vector3;

use crate::{OCTREE_MAX_NODE, service::Service};

const CHILD_SIGN: [[i32; 3]; 8] = [[-1, -1, -1], [1, -1, -1], [1, -1, 1], [-1, -1, 1], [-1, 1, -1], [1, 1, -1], [1, 1, 1], [-1, 1, 1]];

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct TreeNode {
    pub parent: u32,
    pub children: [u32; 8],

    pub X: f32,
    pub Y: f32,
    pub Z: f32,
}

pub struct WorldData {
    pub octree_root: u32,
    pub data: [TreeNode; OCTREE_MAX_NODE],
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Uniform {
    pub time: u32,

    pub field_of_view: f32,
    pub max_ray_length: u32,

    pub rot_horizontal: f32,
    pub rot_vertical: f32,
    
    pub X: f32,
    pub Y: f32,
    pub Z: f32,
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct GraphicPref {
    pub empty: u32,
}

impl TreeNode {
    pub fn empty() -> TreeNode {
        TreeNode { parent: 0, children: [0; 8], X: 0.0, Y: 0.0, Z: 0.0 }
    }

    pub fn new(parent: u32, pos: Vector3<f32>, ) -> TreeNode {
        TreeNode { parent, children: [0; 8], X: pos.x, Y: pos.y, Z: pos.z }
    }

    pub fn child_boundary(&self, sign: &Vector3<i32>, size: f32, ) -> Vector3<f32> {
        Vector3::new(self.X + (sign.x as f32 * size), self.Y + (sign.y as f32 * size), self.Z + (sign.z as f32 * size))
    }

    pub fn create_child(&self, sign: &Vector3<i32>, cur_size: f32, parent: u32, ) -> TreeNode {
        let child_pos = self.child_boundary(sign, cur_size / 4.0);
        TreeNode::new(parent, child_pos, )
    }

    pub fn check_pos_in_child(&self, sign: &Vector3<i32>, cur_size: f32, cur_pos: Vector3<f32>, ) -> bool {
        Service::check_in_volume(&Vector3::new(self.X, self.Y, self.Z), &self.child_boundary(sign, cur_size / 2.0), &cur_pos, )
    }
}

impl WorldData {
    pub fn create_children(parent: &TreeNode, cur_size: f32, parent_index: u32, ) -> [TreeNode; 8] {
        let mut children: Vec<TreeNode> = vec![];
        for sign in CHILD_SIGN { children.push(parent.create_child(&Vector3::new(sign[0], sign[1], sign[2]), cur_size, parent_index)) }
        Service::vec_to_array(children)
    }

    pub fn choose_child_node(parent: &TreeNode, cur_size: f32, cur_pos: Vector3<f32>, ) -> Option<&u32> {
        for (index, child, ) in parent.children.iter().enumerate() { if parent.check_pos_in_child(&Vector3::new(CHILD_SIGN[index][0], CHILD_SIGN[index][1], CHILD_SIGN[index][2]), cur_size, cur_pos) { return Some(child) } } None
    }

    pub fn insert_node(root_index: u32, root_size: f32, data: &mut Vec<TreeNode>, pos_to_insert: Vector3<f32>, ) {
        let mut cur_index = root_index;
        let mut cur_size = root_size;

        while (cur_size * 1000.0).fract() == 0.0 {
            let mut cur_node = data[cur_index as usize];

            if cur_node.children == [0; 8] {
                let children = Self::create_children(&cur_node, cur_size, cur_index, );
                for (index, child, ) in children.iter().enumerate() { cur_node.children[index] = data.len() as u32; data.push(child.to_owned()); }
            }

            let child_node_index = Self::choose_child_node(&cur_node, cur_size, pos_to_insert.clone());
            
            cur_index = child_node_index.unwrap().to_owned();
            cur_size = cur_size / 2.0;
        }
    }

    pub fn collect() -> WorldData {
        let mut data: [TreeNode; OCTREE_MAX_NODE] = [TreeNode::empty(); OCTREE_MAX_NODE];
        let mut editable_data: Vec<TreeNode> = vec![TreeNode::empty()];

        WorldData::insert_node(0, 100.0, &mut editable_data, Vector3::new(5.0, 6.0, 4.0));

        WorldData { octree_root: 0, data }
    }
}