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
    // RootIndex = 0
    pub root_span: f32,
    // MinVoxSpan
    pub max_detail: f32,
    // Octree as List
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

    // Return SpaceIndex
    pub fn choose_child_node(&self, cur_pos: &Vector3<f32>, parent_span: f32, ) -> u32 {
        let center = self.center.to_vec();
        let top_right = self.get_top_right(parent_span);

        // Generate LocalPosition
        let x = Service::check_number_in_range([center.x, top_right.x], cur_pos.x, ) as u32 as f32;
        let y = Service::check_number_in_range([center.y, top_right.y], cur_pos.y, ) as u32 as f32;
        let z = Service::check_number_in_range([center.z, top_right.z], cur_pos.z, ) as u32 as f32;
        
        // Convert to SpaceIndex -> Index inside Parent
        Service::pos_to_index(&Vector3::new(x, y, z, ), 2)
    }
}

impl Octree {
    pub fn empty(uniform: &Uniform) -> Octree {
        let data: Vec<TreeNode> = vec![TreeNode::empty()];
        Octree { 
            root_span: uniform.root_span,
            max_detail: uniform.max_detail,
            data
        }
    }

    pub fn node_at_pos(&mut self, pos_to_find: Vector3<f32>, ) -> usize {
        // Start at Root
        let mut cur_index = 0;
        let mut cur_span = self.root_span;

        // Stop at MaxDetail
        while cur_span >= self.max_detail {
            if self.data[cur_index].node_type == 1 {
                // Select ChildNode base on the Position which is searched
                let child_node_index = self.data[cur_index].choose_child_node(&pos_to_find, cur_span, );
                // Update CurIndex with certain ChildNode of current Index
                cur_index = self.data[cur_index].children[child_node_index as usize] as usize;
                // Next Node is half the Span
                cur_span /= 2.0;
            } else {
                // Case -> At Leaf -> Return current LeafNode
                return cur_index
            }
        } cur_index
    }

    pub fn insert_node(&mut self, insert_pos: Vector3<f32>, ) -> usize {
        // Start at Root
        let mut cur_index = 0;
        let mut cur_span = self.root_span;

        // Stop at MaxDetail
        while cur_span >= self.max_detail {
            // If not Subdivide then change into
            if self.data[cur_index].node_type == 0 {
                self.data[cur_index].node_type = 1;

                for index in 0 .. 8 {
                    // Child Index = Next Index of OctreeData + CurChildIndex
                    self.data[cur_index as usize].children[index] = self.data.len() as u32;
                    // Add Child to Octree
                    self.data.push(self.data[cur_index as usize].create_child(index as u32, cur_index as u32, cur_span, ))
                }
            }

            // Select ChildNode base on the InsertPos
            let child_node_index = self.data[cur_index].choose_child_node(&insert_pos, cur_span, );
            // Update CurIndex with certain ChildNode of current Index
            cur_index = self.data[cur_index].children[child_node_index as usize] as usize;
            // Next Node is half the Span
            cur_span /= 2.0;
        } cur_index
    }

    pub fn collect_random(&mut self, vox_amount: u32, ) {
        let mut rnd = rand::thread_rng();
        for _ in 0 .. vox_amount {
            // Insert Node at random Position
            let rnd_range = - self.root_span / 2.0 .. self.root_span / 2.0;
            self.insert_node(VecThree::from_float(rnd.gen_range(rnd_range)).to_vec()); 
        }
    }
}