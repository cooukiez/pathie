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
pub struct Subdivide {
    // Store index, 0 = empty | > 0 = full -> subdiv or leaf
    pub children: [i32; 8],
    // Store index, 0 = empty | 1 = full, store compact with bitshifting
    pub basic_children: u32,

    pub parent: i32,
    pub padding: [u32; 2],
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Leaf {
    // Store material here later on
    pub mat: Material,

    pub parent: i32,
    pub padding: [u32; 3],
}

impl Subdivide {
    pub fn new(parent: usize) -> Subdivide {
        Subdivide {
            parent: parent as i32,
            ..Default::default()
        }
    }

    pub fn has_children(&self) -> bool {
        self.children[0] > 0
    }

    pub fn get_child_mask(cur_span: f32, local_origin: Vector3<f32>) -> Vector3<f32> {
        local_origin.step(Vector3::from([cur_span; 3]))
    }
}

impl Leaf {
    pub fn new(parent: usize) -> Leaf {
        Self {
            parent: parent as i32,
            ..Default::default()
        }
    }

    pub fn set(&mut self, mat: &Material) {
        self.mat = mat.clone();
    }
}

impl Default for Material {
    fn default() -> Self {
        Self {
            base_color: Vector4::default(),
        }
    }
}

impl Default for Subdivide {
    fn default() -> Self {
        Self {
            children: [0; 8],
            basic_children: 0,
            parent: 0,
            padding: [0; 2],
        }
    }
}

impl Default for Leaf {
    fn default() -> Self {
        Self {
            mat: Material::default(),
            parent: 0,
            padding: [0; 3],
        }
    }
}
