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
    pub fn new() -> TreeNode {
        TreeNode { parent: 0, children: [0; 8], X: 0.0, Y: 0.0, Z: 0.0 }
    }
}

impl WorldData {
    pub fn create_children(parent: &TreeNode, parent_index: u32, cur_size: f32, ) -> [TreeNode; 8] {
        let child_pos = | parent_pos: Vector3<f32>, sign: Vector3<i32>, size: f32 | -> Vector3<f32> 
        { Vector3::new(parent_pos.x + (sign.x as f32 * size), parent_pos.y + (sign.y as f32* size), parent_pos.z + (sign.z as f32* size)) };

        let mut children: Vec<TreeNode> = vec![];
        for sign in CHILD_SIGN {
            let cur_child_pos = child_pos(Vector3::new(parent.X, parent.Y, parent.Z, ), Vector3::new(sign[0], sign[1], sign[2], ), cur_size / 4.0, );
            children.push(TreeNode { parent: parent_index, children: [0; 8], X: cur_child_pos.x, Y: cur_child_pos.y, Z: cur_child_pos.z })
        }

        Service::vec_to_array(children)
    }

    pub fn choose_child_node(parent: &TreeNode, cur_size: f32, cur_pos: &Vector3<f32>, ) -> usize {
        let mut selected = -1;
        for index in 0 .. CHILD_SIGN.len() {
            let child_boundary = Vector3::new(parent.X + (CHILD_SIGN[index][0] as f32 * cur_size / 2.0), parent.Y + (CHILD_SIGN[index][1] as f32 * cur_size / 2.0),parent.Z + (CHILD_SIGN[index][2] as f32 * cur_size / 2.0));
            let x = Service::check_boundary([parent.X, child_boundary.x], cur_pos.x, );
            let y = Service::check_boundary([parent.Y, child_boundary.y], cur_pos.y, );
            let z = Service::check_boundary([parent.Z, child_boundary.z], cur_pos.z, );
            if x && y && z { selected = index as i32; break; }
        }
        selected as usize
    }

    pub fn insert_node(root_index: u32, root_size: f32, size_limit: f32, data: &mut Vec<TreeNode>, pos_to_insert: Vector3<f32>, ) {
        let mut cur_node = data[root_index as usize]; let mut cur_index = root_index; let mut cur_size = root_size;
        while (cur_size * 1000.0).fract() == 0.0 {
            if cur_node.children == [0; 8] {
                let children = WorldData::create_children(&cur_node, cur_index, cur_size, );
                for child_index in 0 .. children.len() {
                    cur_node.children[child_index] = data.len() as u32;
                    data.push(children[child_index]);
                }
                log::info!("{} {} {}", pos_to_insert.x, pos_to_insert.y, pos_to_insert.z);
                let child_node_index = WorldData::choose_child_node(&cur_node, cur_size, &pos_to_insert);
                log::info!("{} {} {}", data[cur_node.children[child_node_index] as usize].X, data[cur_node.children[child_node_index] as usize].Y, data[cur_node.children[child_node_index] as usize].Z);
                cur_node = data[cur_node.children[child_node_index] as usize]; cur_index = cur_node.children[child_node_index]; cur_size /= 2.0;
            }
            else {
                let child_node_index = WorldData::choose_child_node(&cur_node, cur_size, &pos_to_insert);
                cur_node = data[cur_node.children[child_node_index] as usize]; cur_index = cur_node.children[child_node_index]; cur_size /= 2.0;
            }

            log::info!("{}", cur_index);
        }
    }

    pub fn collect() -> WorldData {
        let mut data: [TreeNode; OCTREE_MAX_NODE] = [TreeNode::new(); OCTREE_MAX_NODE];
        let mut editable_data: Vec<TreeNode> = vec![TreeNode::new()];

        WorldData::insert_node(0, 100.0, 1.0, &mut editable_data, Vector3::new(5.0, 6.0, 4.0));

        WorldData { octree_root: 0, data }
    }
}