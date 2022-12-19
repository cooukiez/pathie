use cgmath::Vector3;

use crate::OCTREE_MAX_NODE;

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
    pub fn vec_to_array<Type, const Length: usize>(vec: Vec<Type>) -> [Type; Length] { vec.try_into().unwrap_or_else(| vec: Vec<Type> | panic!("ERR_INVALI_LEN -> Expected {} | Got {}", Length, vec.len())) }

    pub fn create_children(parent: &TreeNode, parent_index: u32, child_size: f32, ) -> [TreeNode; 8] {
        let child_pos = | parent_pos: Vector3<f32>, sign: Vector3<i32>, size: f32 | -> Vector3<f32> 
        { Vector3::new(parent_pos.x + (sign.x as f32* size), parent_pos.y + (sign.y as f32* size), parent_pos.z + (sign.z as f32* size)) };

        let children: Vec<TreeNode> = vec![];
        for sign in CHILD_SIGN {
            let cur_child_pos = child_pos(Vector3::new(parent.X, parent.Y, parent.Z, ), Vector3::new(sign[0], sign[1], sign[2], ), child_size, );
            children.push(TreeNode { parent: parent_index, children: [0; 8], X: cur_child_pos.x, Y: cur_child_pos.y, Z: cur_child_pos.z })
        }

        WorldData::vec_to_array(children)
    }

    pub fn get_child_index() {

    }

    pub fn insert_node(root: &TreeNode, root_index: u32, root_size: f32, depth: u32, ) {
        let cur_node = root; let cur_index = root_index; let cur_size = root_size;
        for cur_depth in 0 .. depth {
            if cur_node.children == [0; 8] {
                
            }
            else {
                let children = WorldData::create_children(cur_node, cur_index, cur_size, );
                for child_index in 0 .. children.len() {
                    
                }
            }
        }
    }

    pub fn collect() -> WorldData {
        let mut data: [TreeNode; OCTREE_MAX_NODE] = [TreeNode::new(); OCTREE_MAX_NODE];

        WorldData { octree_root: 0, data }
    }

    
}