use cgmath::{Vector3, Vector4};

use crate::service::Vector;

// In struct, Vector four is used because of memory alignment in vulkan.
// Vector three is aligned as vec four in vulkan but as vec three in rust.
// This is problematic therefore we use vec four.

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Material {
    pub base_color: Vector4<f32>,
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Octant {
    // Store index, 0 = empty | > 0 = full -> subdiv or leaf
    pub children: [u32; 8],
    // Store index, 0 = empty | 1 = full, store compact with bitshifting
    pub basic_children: u32,

    pub parent: u32,
    // 0 = Empty | 1 = Subdivide | 2 = Full
    pub node_type: u32,
    pub padding: [u32; 1],

    pub mat: Material,
}

impl Octant {
    pub fn new(parent: usize, node_type: u32) -> Octant {
        Octant {
            parent: parent as u32,
            node_type,
            ..Default::default()
        }
    }

    pub fn set(&mut self, mat: &Material) {
        self.mat = mat.clone();
    }

    pub fn update_basic_children(parent: &mut Octant) {
        for (idx, child_idx) in parent.children.iter().enumerate() {
            if child_idx.clone() > 0 {
                parent.basic_children |= 1 << 7 - idx;
            } else {
                parent.basic_children |= 0 << 7 - idx;
            }
        }
    }

    pub fn has_children(&self) -> bool {
        self.children[0] > 0
    }

    pub fn is_subdiv(&self) -> bool {
        self.node_type == 1
    }

    pub fn get_child_mask(cur_span: f32, local_origin: Vector3<f32>) -> Vector3<f32> {
        local_origin.step(Vector3::from([cur_span; 3]))
    }
}

impl Default for Material {
    fn default() -> Self {
        Self {
            base_color: Vector4::default(),
        }
    }
}

impl Default for Octant {
    fn default() -> Self {
        Self {
            children: [0; 8],
            basic_children: 00000000,
            parent: 0,
            node_type: 0,
            padding: [0; 1],
            mat: Material::default(),
        }
    }
}
