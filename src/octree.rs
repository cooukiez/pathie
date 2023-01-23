use cgmath::Vector3;
use rand::Rng;

use crate::{service::Service, uniform::{VecThree, VecFour, Uniform}};

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct TreeNode {
    pub base_color: VecFour,

    // 0 = empty | 1 = subdivide | 2 = full
    pub node_type: u32,
    pub parent: u32,
    
    pub children: [u32; 8],

    // Position of Center
    pub center: VecThree,
}

pub struct Octree {
    pub root_index: u32,
    pub root_span: f32,

    pub data: Vec<TreeNode>,
}

impl TreeNode {
    pub fn empty() -> TreeNode {
        TreeNode { base_color: VecFour::from_float(0.0), node_type: 0, parent: 0, children: [0; 8], center: VecThree::new(0.0, 0.0, 0.0, ) }
    }

    pub fn new(base_color: VecFour, node_type: u32, parent: u32, pos: Vector3<f32>, ) -> TreeNode {
        TreeNode { base_color, node_type, parent, children: [0; 8], center: VecThree::from_vec(pos) }
    }

    pub fn get_outer_pos(&self, sign: &Vector3<f32>, diameter: f32, ) -> Vector3<f32> {
        self.center.to_vec() + (sign * (diameter as f32))
    }

    pub fn get_bottom_left(&self, span: f32, ) -> Vector3<f32> {
        self.center.to_vec() - VecThree::from_float(span / 2.0).to_vec()
    }

    pub fn get_top_right(&self, span: f32, ) -> Vector3<f32> {
        self.center.to_vec() + VecThree::from_float(span / 2.0).to_vec()
    }

    pub fn create_child(&self, child_index: u32, parent_index: u32, parent_span: f32, ) -> TreeNode {
        // LocalPos -> Par Example -> [ 0 0 1 ] or [ 1 0 1 ] * ChildSize
        let local_pos = Service::convert_index_to_pos(child_index, 2) * parent_span / 2.0;
        // BottomCorner + Offset [ -> Required for Center ] + LocalPos in Parent
        let global_pos = self.get_bottom_left(parent_span) + VecThree::from_float(parent_span / 4.0).to_vec() + local_pos;
        
        TreeNode::new(VecFour::from_float(0.0), 0, parent_index, global_pos, )
    }

    pub fn create_children(&mut self, parent_index: u32, parent_span: f32, data: &mut Vec<TreeNode>, ) {
        for index in 0 .. 8 {
            self.children[index] = data.len() as u32;
            data.push(self.create_child(index as u32, parent_index, parent_span, ))
        }
    }

    // Return SpaceIndex
    pub fn choose_child_node(&self, cur_pos: &Vector3<f32>, parent_span: f32, ) -> u32 {
        let center = self.center.to_vec();
        let top_right = self.get_top_right(parent_span);

        // Generate LocalPosition
        let x = Service::check_number_in_range([center.x, top_right.x], cur_pos.x, ) as u32 as f32;
        let y = Service::check_number_in_range([center.y, top_right.y], cur_pos.y, ) as u32 as f32;
        let z = Service::check_number_in_range([center.z, top_right.z], cur_pos.z, ) as u32 as f32;
        
        // Convert to SpaceIndex
        Service::pos_to_index(&Vector3::new(x, y, z, ), 2)
    }
}

impl Octree {
    pub fn empty(uniform: &Uniform) {
        let mut data: Vec<TreeNode> = vec![TreeNode::empty()];

        let root_pos = VecThree::from_float(0.0).to_vec();
        data[root_index] = TreeNode::new(VecFour::from_float(0.0), 0, root_index as u32, root_pos, );

        Octree { root_index, root_span, data }
    }

    pub fn node_at_pos(root_index: usize, root_span: f32, data: &mut Vec<TreeNode>, insert_pos: Vector3<f32>, ) -> usize {
        let mut cur_index = root_index;
        let mut cur_span = root_span;

        while cur_span >= 1.0 {
            if data[cur_index].node_type == 0 {
                data[cur_index].node_type = 1;
                data[cur_index].create_children(cur_index, cur_span, &mut data, )
            }

            let child_node_index = data[cur_index].choose_child_node(&insert_pos, cur_span, );
            cur_index = data[cur_index].children[child_node_index as usize] as usize;
        } cur_index
    }

    pub fn insert_node(&mut self, insert_pos: Vector3<f32>, ) -> usize {
        let mut cur_index = self.root_index as usize;
        let mut cur_span = self.root_span;

        while cur_span >= 1.0 {
            if self.data[cur_index].node_type == 0 {
                self.data[cur_index].node_type = 1;
                self.data[cur_index].create_children(cur_index, cur_span, &mut data, )
            }

            let child_node_index = self.data[cur_index].choose_child_node(&insert_pos, cur_span, );
            cur_index = self.data[cur_index].children[child_node_index as usize] as usize;
        } cur_index
    }

    pub fn collect_random(&mut self, vox_amount: u32, ) {
        let mut rnd = rand::thread_rng();
        for _ in 0 .. vox_amount { 
            octree.insert_node(VecThree::from_float(rnd.gen_range((- span / 2.0) .. (span / 2.0))).to_vec()); 
        }
    }
}