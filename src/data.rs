use cgmath::InnerSpace;
use cgmath::Vector3;
use cgmath::VectorSpace;

use crate::{OCTREE_MAX_NODE, service::Service};

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct TreeNode {
    pub mat: u32,
    pub parent: u32,
    pub children: [u32; 8],
    pub space_index: u32,
    pub span: f32,

    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub struct WorldData {
    pub octree_root: u32,
    pub data: [TreeNode; OCTREE_MAX_NODE],
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Uniform {
    pub time: u32,

    pub raw_field_of_view: f32,
    pub max_ray_length: u32,

    pub rot_horizontal: f32,
    pub rot_vertical: f32,

    pub octree_root_index: u32,
    
    pub node_at_pos: u32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct GraphicPref {
    pub empty: u32,
}

impl TreeNode {
    pub fn empty() -> TreeNode {
        TreeNode { mat: 0, parent: 0, span: 0.0, space_index: 0, children: [0; 8], x: 0.0, y: 0.0, z: 0.0 }
    }

    pub fn new(parent: u32, span: f32, space_index: u32, pos: Vector3<f32>, ) -> TreeNode {
        TreeNode { mat: 0, parent, span, children: [0; 8], space_index, x: pos.x, y: pos.y, z: pos.z }
    }

    pub fn get_outer_pos(&self, sign: &Vector3<i32>, diameter: u32, ) -> Vector3<f32> {
        Vector3::new(self.x + (sign.x as f32 * diameter as f32), self.y + (sign.y as f32 * diameter as f32), self.z + (sign.z as f32 * diameter as f32))
    }

    pub fn get_start_pos(&self) -> Vector3<f32> {
        Vector3::new(self.x - self.span / 2.0, self.y - self.span / 2.0, self.z - self.span / 2.0)
    }

    pub fn create_child(&self, space_index: usize, parent: u32, ) -> TreeNode {
        let space_index_as_pos = Service::convert_index_to_pos(space_index as u32, 2) * self.span / 2.0;
        let child_pos = self.get_start_pos() + Vector3::new(self.span / 2.0, self.span / 2.0, self.span / 2.0) + space_index_as_pos;
        TreeNode::new(parent, self.span / 2.0, space_index as u32, child_pos, )
    }

    pub fn choose_child_node(&self, cur_pos: Vector3<f32>, ) -> u32 {
        let parent_pos = Vector3::new(self.x, self.y, self.z);

        let horizontal = Service::check_number_in_range([parent_pos.x, parent_pos.x + self.span / 2.0], cur_pos.x, ) as u32 as f32;
        let vertical = Service::check_number_in_range([parent_pos.y, parent_pos.y + self.span / 2.0], cur_pos.y, ) as u32 as f32;
        let depth = Service::check_number_in_range([parent_pos.z, parent_pos.z + self.span / 2.0], cur_pos.z, ) as u32 as f32;

        Service::pos_to_index(&Vector3::new(horizontal, vertical, depth), 2.0)
    }

    pub fn create_children(&self, parent_index: u32, ) -> [TreeNode; 8] {
        let mut children: Vec<TreeNode> = vec![];

        for index in 0 .. self.children.len() {
            children.push(self.create_child(index, parent_index, ))
        }

        Service::vec_to_array(children)
    }
}

impl WorldData {
    

    pub fn insert_node(root_index: u32, min_span: f32, data: &mut Vec<TreeNode>, pos_to_insert: Vector3<f32>, ) {
        let mut cur_index = root_index;
        loop {
            if data[cur_index as usize].span <= min_span { break; }
            if data[cur_index as usize].children == [0; 8] {
                let children = &data[cur_index as usize].create_children(cur_index);

                for (index, child, ) in children.iter().enumerate() 
                { data[cur_index as usize].children[index] = data.len() as u32; data.push(child.to_owned()); }
            }

            let child_node_index = &data[cur_index as usize].choose_child_node(pos_to_insert.clone());
            
            cur_index = child_node_index.clone();
        }
    }

    pub fn format_octree(editable_data: &mut Vec<TreeNode>) -> [TreeNode; OCTREE_MAX_NODE] {
        for _ in 0 .. (OCTREE_MAX_NODE - editable_data.len()) { editable_data.push(TreeNode::empty()); } 
        Service::vec_to_array(editable_data.clone())
    }

    pub fn collect() -> WorldData { 
        let mut editable_data: Vec<TreeNode> = vec![TreeNode::empty()];
        editable_data[0] = TreeNode::new(0, 64.0, 0, Vector3::new(0.0, 0.0, 0.0, ), );
        WorldData::insert_node(0, 1.0,&mut editable_data, Vector3::new(5.0, 6.0, 4.0), );
        WorldData::insert_node(0, 1.0,&mut editable_data, Vector3::new(5.0, 6.0, 5.0), );

        let octree = Self::format_octree(&mut editable_data.clone());
        WorldData { octree_root: 0, data: octree }
    }
}