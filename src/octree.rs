use cgmath::Vector3;
use rand::Rng;

use crate::{service::{pos_to_index, step_vec_three, floor_vec_three, add_dir_to_mask}};

const MAX_RECURSION: usize = 10;
const MAX_SEARCH_DEPTH: usize = 4096;

const ROOT_SPAN: f32 = 4096.0;

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct TreeNode {
    // 0 = empty | 1 = subdivide | 2 = full
    pub node_type: u32,
    pub parent: u32,
    
    pub children: [u32; 8],
}

// Store Info about OctreeTraverse
pub struct Traverse {
    pub cur_index: usize,
    pub cur_span: f32,
    pub cur_recursion: usize,

    pub ray_origin: Vector3<f32>,
    pub ray_dir: Vector3<f32>,

    // Origin in CurNode
    pub local_origin: Vector3<f32>,
    // Origin on first edge of CurNode
    pub origin_on_edge: Vector3<f32>,

    pub mask_in_parent: [Vector3<f32>; MAX_RECURSION],
}

pub struct Octree {
    // Octree as List
    pub data: Vec<TreeNode>,
    // RootIndex = 0
}

impl TreeNode {
    pub fn new(node_type: u32, parent: usize, ) -> TreeNode {
        TreeNode { node_type, parent: parent as u32, .. Default::default() }
    }

    pub fn get_center(node_pos: Vector3<f32>, span: f32, ) -> Vector3<f32> {
        node_pos + Vector3::from([span * 0.5; 3])
    }

    pub fn get_top_right(node_pos: Vector3<f32>, span: f32, ) -> Vector3<f32> {
        node_pos + Vector3::from([span; 3])
    }

    pub fn get_child_mask(cur_span: f32, local_origin: Vector3<f32>, ) -> Vector3<f32> {
        step_vec_three(Vector3::from([cur_span; 3]), local_origin, )
            .cast::<f32>()
            .unwrap()
    }
}

impl Octree {
    // If not subdivide -> Create children
    pub fn try_child_creation(data: &mut Vec<TreeNode>, parent_index: usize, ) {
        if data[parent_index].node_type == 0 {
            data[parent_index].node_type = 1;

            for index in 0 .. 8 {
                data[parent_index as usize].children[index] = data.len() as u32;
                data.push(TreeNode::new(0, parent_index, ));
            }
        }
    }

    pub fn insert_node(&mut self, insert_pos: Vector3<f32>, ) -> Traverse {
        let mut traverse = Traverse {
            ray_origin: insert_pos,

            local_origin: insert_pos % ROOT_SPAN,
            origin_on_edge: insert_pos - (insert_pos % ROOT_SPAN),

            .. Default::default()
        };
        
        // 1. Create children if not present
        // 2. Select child based on insert position
        // 3. Repeat
        // Finally -> Set current Node to full
        for _  in 0 .. MAX_RECURSION {
            Self::try_child_creation(&mut self.data, traverse.cur_index);
            // Select next Child
            traverse.move_into_child(&self.data);
        }

        // CurNode = full
        self.data[traverse.cur_index].node_type = 2;

        traverse
    }

    pub fn insert_neighbour(&mut self, origin: Traverse, dir: Vector3<f32>, ) -> Traverse {
        // Select new Pos based on old OriginOnEdge
        // Has to be 1.5 x the span because directly in center of neighbour
        let insert_pos = 
            origin.origin_on_edge + dir * origin.cur_span * 1.5;

        let mut traverse = Traverse {
            ray_origin: insert_pos,

            local_origin: insert_pos % ROOT_SPAN,
            origin_on_edge: insert_pos - (insert_pos % ROOT_SPAN),

            .. Default::default()
        };
        
        // Process described above
        for _  in 0 .. MAX_RECURSION {
            Self::try_child_creation(&mut self.data, traverse.cur_index);
            traverse.move_into_child(&self.data);
        }

        self.data[traverse.cur_index].node_type = 2;

        traverse
    }

    pub fn test_scene(&mut self) {
        self.insert_node(Vector3::new(120.3, 321.2, 213.1));
        self.insert_node(Vector3::new(10.3, 230.4, 60.0));
        self.insert_node(Vector3::new(10.1, 210.0, 46.7));
        self.insert_node(Vector3::new(400.1, 10.0, 100.7));
        // self.insert_node(Vector3::new(255.1, 255.0, 2.7));

        let mut rng = rand::thread_rng();
        for _ in 0 .. 2000 {
            self.insert_node(Vector3::new(rng.gen_range(0.0 .. 4000.0), rng.gen_range(0.0 .. 4000.0), rng.gen_range(0.0 .. 4000.0)));
        }
    }
}

impl Traverse {
    // Return SpaceIndex
    pub fn move_into_child(&mut self, data: &Vec<TreeNode>, ) {
        self.cur_recursion += 1;
        self.cur_span *= 0.5;

        let child_mask = TreeNode::get_child_mask(self.cur_span, self.local_origin, );

        self.origin_on_edge += child_mask * self.cur_span;
        self.local_origin -= child_mask * self.cur_span;

        // Save position in parent for later use
        self.mask_in_parent[self.cur_recursion] = child_mask.clone();
        
        let space_index = pos_to_index(child_mask, 2, );
        // Get global index of selected child
        self.cur_index = data[self.cur_index].children[space_index] as usize;
    }

    pub fn move_layer_up(&mut self, data: &Vec<TreeNode>, dir_mask: Vector3<f32>, ) {
        // Compute the rest ( because of moving up ) into LocalOrigin
        let new_origin_on_edge = 
            floor_vec_three(self.origin_on_edge / (self.cur_span / 2.0)) * (self.cur_span * 2.0);
        
        self.local_origin += self.origin_on_edge - new_origin_on_edge;
        self.origin_on_edge = new_origin_on_edge;

        self.cur_recursion -= 1;
        self.cur_span *= 2.0;

        // Use earlier saved mask and move in dir
        self.mask_in_parent[self.cur_recursion] = 
            add_dir_to_mask(self.mask_in_parent[self.cur_recursion], dir_mask, );

        // Temp save parent of parent of CurNode
        let parent_of_parent = 
            data[data[self.cur_index].parent as usize];
        
        // Moved mask into SpaceIndex and get global index of Child
        let next_space_index = 
            pos_to_index(self.mask_in_parent[self.cur_recursion], 2, );
        self.cur_index = parent_of_parent
            .children[next_space_index] as usize;
    }

    // Add move forward function
}

impl Default for TreeNode {
    fn default() -> Self {
        Self { node_type: 0, parent: 0, children: [0; 8] }
    }
}

impl Default for Traverse {
    fn default() -> Self {
        Self { 
            cur_index: 0,
            cur_span: ROOT_SPAN,
            cur_recursion: 0,

            ray_origin: Vector3::from([0.0; 3]),
            ray_dir: Vector3::from([0.0; 3]),

            local_origin: Vector3::from([0.0; 3]),
            origin_on_edge: Vector3::from([0.0; 3]),

            mask_in_parent: [Vector3::from([0.0; 3]); MAX_RECURSION]
        }
    }
}

impl Default for Octree {
    fn default() -> Self {
        Self { data: vec![TreeNode::default()] }
    }
}