use cgmath::Vector3;
use rand::Rng;

use crate::{service::Service, uniform::{VecThree, VecFour}};

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct TreeNode {
    pub color: VecFour,

    // 0 = empty | 1 = subdivide | 2 = full
    pub node_type: u32,
    pub parent: u32,

    // Width
    pub span: f32,
    // See Hashing
    pub space_index: u32,
    
    pub children: [u32; 8],

    // Position of Center
    pub center: VecThree,
}

pub struct Octree {
    pub octree_root: u32,
    pub data: Vec<TreeNode>,
}

impl TreeNode {
    pub fn empty() -> TreeNode {
        TreeNode { color: VecFour::from_float(0.0), node_type: 0, parent: 0, span: 0.0, space_index: 0, children: [0; 8], center: VecThree::new(0.0, 0.0, 0.0, ) }
    }

    pub fn new(color: VecFour, node_type: u32, parent: u32, span: f32, space_index: u32, pos: Vector3<f32>, ) -> TreeNode {
        TreeNode { color, node_type, parent, span, space_index, children: [0; 8], center: VecThree::from_vec(pos) }
    }

    pub fn get_outer_pos(&self, sign: &Vector3<f32>, diameter: f32, ) -> Vector3<f32> {
        self.center.to_vec() + (sign * (diameter as f32))
    }

    pub fn get_bottom_left(&self) -> Vector3<f32> {
        self.center.to_vec() - VecThree::from_float(self.span / 2.0).to_vec()
    }

    pub fn get_top_right(&self) -> Vector3<f32> {
        self.center.to_vec() + VecThree::from_float(self.span / 2.0).to_vec()
    }

    pub fn create_child(&self, space_index: usize, parent: u32, ) -> TreeNode {
        let local_pos = Service::convert_index_to_pos(space_index as u32, 2) * self.span / 2.0;
        let global_pos = self.get_bottom_left() + VecThree::from_float(self.span / 4.0).to_vec() + local_pos;
        
        TreeNode::new(VecFour::from_float(0.0), 2, parent, self.span / 2.0, space_index as u32, global_pos, )
    }

    // Return SpaceIndex
    pub fn choose_child_node(&self, cur_pos: &Vector3<f32>, ) -> u32 {
        let center = self.center.to_vec();
        let top_right = self.get_top_right();

        // Generate LocalPosition
        let x = Service::check_number_in_range([center.x, top_right.x], cur_pos.x, ) as u32 as f32;
        let y = Service::check_number_in_range([center.y, top_right.y], cur_pos.y, ) as u32 as f32;
        let z = Service::check_number_in_range([center.z, top_right.z], cur_pos.z, ) as u32 as f32;
        
        // Convert to SpaceIndex
        Service::pos_to_index(&Vector3::new(x, y, z, ), 2)
    }
}

impl Octree {
    pub fn insert_node(root_index: usize, data: &mut Vec<TreeNode>, pos_to_insert: Vector3<f32>, ) -> usize {
        let mut cur_index = root_index;
        while data[cur_index].span > 1.0 {
            if data[cur_index].children == [0; 8] {
                data[cur_index].node_type = 1;
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
        data[root_index] = TreeNode::new(VecFour::from_float(0.0), 0, root_index as u32, span, 0, VecThree::from_float(0.0).to_vec(), );

        // Remove Later
        for _ in 0 .. vox_amount {
            let pos_to_insert = Vector3 { 
                x: rnd.gen_range((- span / 2.0) .. (span / 2.0)), 
                y: rnd.gen_range((- span / 2.0) .. (span / 2.0)), 
                z: rnd.gen_range((- span / 2.0) .. (span / 2.0)),
            };

            Self::insert_node(0, &mut data, pos_to_insert, ); 
        }

        Octree { octree_root: 0, data }
    }
}